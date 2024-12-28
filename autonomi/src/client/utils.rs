// Copyright 2024 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use crate::client::payment::{receipt_from_store_quotes, Receipt};
use ant_evm::{EvmWallet, ProofOfPayment};
use ant_networking::{GetRecordCfg, PutRecordCfg, VerificationKind};
use ant_protocol::{
    messages::ChunkProof,
    storage::{try_serialize_record, Chunk, RecordKind, RetryStrategy},
};
use bytes::Bytes;
use futures::stream::{FuturesUnordered, StreamExt};
use libp2p::kad::{Quorum, Record};
use rand::{thread_rng, Rng};
use self_encryption::{streaming_decrypt_from_storage, DataMap, Error as SelfEncryptionError};
use std::{future::Future, num::NonZero};
use tempfile::NamedTempFile;
use tokio::fs;
use xor_name::XorName;

use super::{
    data::{GetError, PayError, PutError},
    Client,
};

impl Client {
    /// Fetch and decrypt all chunks in the data map.
    pub(crate) async fn fetch_from_data_map(&self, data_map: &DataMap) -> Result<Bytes, GetError> {
        debug!("Fetching encrypted data chunks from data map {data_map:?}");

        // Create a temporary file to store the decrypted data
        let temp_file = NamedTempFile::new().map_err(|e| {
            GetError::Decryption(crate::self_encryption::Error::SelfEncryption(e.into()))
        })?;
        let temp_path = temp_file.path().to_owned();

        // Create a closure to fetch chunks
        let client = self.clone();
        let get_chunks = move |xor_names: &[XorName]| -> Result<Vec<Bytes>, SelfEncryptionError> {
            let mut chunks = Vec::with_capacity(xor_names.len());
            for xor_name in xor_names {
                match futures::executor::block_on(client.chunk_get(*xor_name)) {
                    Ok(chunk) => chunks.push(chunk.value),
                    Err(err) => {
                        error!("Error fetching chunk {:?}: {err:?}", xor_name);
                        return Err(SelfEncryptionError::Generic(format!(
                            "Failed to fetch chunk: {}",
                            err
                        )));
                    }
                }
            }
            Ok(chunks)
        };

        // Decrypt the data using streaming decryption
        streaming_decrypt_from_storage(data_map, &temp_path, get_chunks)
            .map_err(|e| GetError::Decryption(crate::self_encryption::Error::SelfEncryption(e)))?;

        // Read the decrypted data
        let bytes = fs::read(&temp_path).await.map_err(|e| {
            GetError::Decryption(crate::self_encryption::Error::SelfEncryption(e.into()))
        })?;

        Ok(Bytes::from(bytes))
    }

    /// Unpack a data map and fetch all bytes using self-encryption.
    pub(crate) async fn fetch_from_data_map_chunk(
        &self,
        data_map_bytes: &Bytes,
    ) -> Result<Bytes, GetError> {
        let data_map_level: crate::self_encryption::DataMapLevel =
            rmp_serde::from_slice(data_map_bytes)
                .map_err(GetError::InvalidDataMap)
                .inspect_err(|err| error!("Error deserializing data map level: {err:?}"))?;

        let data_map = match data_map_level {
            crate::self_encryption::DataMapLevel::First(data_map) => data_map,
            crate::self_encryption::DataMapLevel::Additional(data_map) => data_map,
        };

        self.fetch_from_data_map(&data_map).await
    }

    pub(crate) async fn chunk_upload_with_payment(
        &self,
        chunk: &Chunk,
        payment: ProofOfPayment,
    ) -> Result<(), PutError> {
        let storing_nodes = payment.payees();

        if storing_nodes.is_empty() {
            return Err(PutError::PayeesMissing);
        }

        debug!("Storing chunk: {chunk:?} to {:?}", storing_nodes);

        let key = chunk.network_address().to_record_key();

        let record_kind = RecordKind::ChunkWithPayment;
        let record = Record {
            key: key.clone(),
            value: try_serialize_record(&(payment, chunk.clone()), record_kind)
                .map_err(|e| {
                    PutError::Serialization(format!(
                        "Failed to serialize chunk with payment: {e:?}"
                    ))
                })?
                .to_vec(),
            publisher: None,
            expires: None,
        };

        let verification = {
            let verification_cfg = GetRecordCfg {
                get_quorum: Quorum::N(NonZero::new(2).expect("2 is non-zero")),
                retry_strategy: Some(RetryStrategy::Balanced),
                target_record: None,
                expected_holders: Default::default(),
                is_register: false,
            };

            let stored_on_node = try_serialize_record(&chunk, RecordKind::Chunk)
                .map_err(|e| PutError::Serialization(format!("Failed to serialize chunk: {e:?}")))?
                .to_vec();
            let random_nonce = thread_rng().gen::<u64>();
            let expected_proof = ChunkProof::new(&stored_on_node, random_nonce);

            Some((
                VerificationKind::ChunkProof {
                    expected_proof,
                    nonce: random_nonce,
                },
                verification_cfg,
            ))
        };

        let put_cfg = PutRecordCfg {
            put_quorum: Quorum::One,
            retry_strategy: Some(RetryStrategy::Balanced),
            use_put_record_to: Some(storing_nodes.clone()),
            verification,
        };
        let payment_upload = Ok(self.network.put_record(record, &put_cfg).await?);
        debug!("Successfully stored chunk: {chunk:?} to {storing_nodes:?}");
        payment_upload
    }

    /// Pay for the chunks and get the proof of payment.
    pub(crate) async fn pay(
        &self,
        content_addrs: impl Iterator<Item = XorName> + Clone,
        wallet: &EvmWallet,
    ) -> Result<Receipt, PayError> {
        let number_of_content_addrs = content_addrs.clone().count();
        let quotes = self.get_store_quotes(content_addrs).await?;

        // Make sure nobody else can use the wallet while we are paying
        debug!("Waiting for wallet lock");
        let lock_guard = wallet.lock().await;
        debug!("Locked wallet");

        // TODO: the error might contain some succeeded quote payments as well. These should be returned on err, so that they can be skipped when retrying.
        // TODO: retry when it fails?
        // Execute chunk payments
        let _payments = wallet
            .pay_for_quotes(quotes.payments())
            .await
            .map_err(|err| PayError::from(err.0))?;

        // payment is done, unlock the wallet for other threads
        drop(lock_guard);
        debug!("Unlocked wallet");

        let skipped_chunks = number_of_content_addrs - quotes.len();
        trace!(
            "Chunk payments of {} chunks completed. {} chunks were free / already paid for",
            quotes.len(),
            skipped_chunks
        );

        let receipt = receipt_from_store_quotes(quotes);

        Ok(receipt)
    }
}

pub(crate) async fn process_tasks_with_max_concurrency<I, R>(tasks: I, batch_size: usize) -> Vec<R>
where
    I: IntoIterator,
    I::Item: Future<Output = R> + Send,
    R: Send,
{
    let mut futures = FuturesUnordered::new();
    let mut results = Vec::new();

    for task in tasks.into_iter() {
        futures.push(task);

        if futures.len() >= batch_size {
            if let Some(result) = futures.next().await {
                results.push(result);
            }
        }
    }

    // Process remaining tasks
    while let Some(result) = futures.next().await {
        results.push(result);
    }

    results
}
