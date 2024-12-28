// Copyright 2024 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use std::hash::{DefaultHasher, Hash, Hasher};
use std::sync::LazyLock;

use crate::client::{
    error::{GetError, PutError},
    payment::{PaymentOption, Receipt},
    utils::process_tasks_with_max_concurrency,
    ClientEvent, UploadSummary,
};
use crate::self_encryption::encrypt;
use crate::Client;
use ant_evm::Amount;
use ant_networking::GetRecordCfg;
use ant_protocol::storage::{Chunk, ChunkAddress};
use ant_protocol::NetworkAddress;
use bytes::Bytes;
use libp2p::kad::Quorum;
use serde::{Deserialize, Serialize};
use tracing::{debug, error, info};
use xor_name::XorName;

pub mod public;
pub mod streaming;

/// Number of chunks to upload in parallel.
///
/// Can be overridden by the `CHUNK_UPLOAD_BATCH_SIZE` environment variable.
pub(crate) static CHUNK_UPLOAD_BATCH_SIZE: LazyLock<usize> = LazyLock::new(|| {
    let batch_size = std::env::var("CHUNK_UPLOAD_BATCH_SIZE")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(
            std::thread::available_parallelism()
                .map(|n| n.get())
                .unwrap_or(1)
                * 8,
        );
    info!("Chunk upload batch size: {}", batch_size);
    batch_size
});

/// Number of chunks to download in parallel.
///
/// Can be overridden by the `CHUNK_DOWNLOAD_BATCH_SIZE` environment variable.
pub static CHUNK_DOWNLOAD_BATCH_SIZE: LazyLock<usize> = LazyLock::new(|| {
    let batch_size = std::env::var("CHUNK_DOWNLOAD_BATCH_SIZE")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(
            std::thread::available_parallelism()
                .map(|n| n.get())
                .unwrap_or(1)
                * 8,
        );
    info!("Chunk download batch size: {}", batch_size);
    batch_size
});

/// Number of retries to upload chunks.
pub(crate) const RETRY_ATTEMPTS: usize = 3;

/// Raw Data Address (points to a DataMap)
pub type DataAddr = XorName;
/// Raw Chunk Address (points to a [`Chunk`])
pub type ChunkAddr = XorName;

/// Private data on the network can be accessed with this
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct DataMapChunk(pub(crate) Chunk);

impl DataMapChunk {
    pub fn value(&self) -> &[u8] {
        self.0.value()
    }

    pub fn to_hex(&self) -> String {
        hex::encode(self.0.value())
    }

    pub fn from_hex(hex: &str) -> Result<Self, hex::FromHexError> {
        let data = hex::decode(hex)?;
        Ok(Self(Chunk::new(Bytes::from(data))))
    }

    /// Get a private address for [`DataMapChunk`]. Note that this is not a network address, it is only used for refering to private data client side.
    pub fn address(&self) -> String {
        hash_to_short_string(&self.to_hex())
    }
}

impl From<Chunk> for DataMapChunk {
    fn from(value: Chunk) -> Self {
        Self(value)
    }
}

fn hash_to_short_string(input: &str) -> String {
    let mut hasher = DefaultHasher::new();
    input.hash(&mut hasher);
    let hash_value = hasher.finish();
    hash_value.to_string()
}

impl Client {
    /// Fetch a blob of (private) data from the network
    ///
    /// # Example
    ///
    /// ```no_run
    /// use autonomi::{Client, Bytes};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let client = Client::init().await?;
    /// # let data_map = todo!();
    /// let data_fetched = client.data_get(data_map).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn data_get(&self, data_map: DataMapChunk) -> Result<Bytes, GetError> {
        info!(
            "Fetching private data from Data Map {:?}",
            data_map.0.address()
        );
        let data = self.fetch_from_data_map_chunk(data_map.0.value()).await?;

        debug!("Successfully fetched a blob of private data from the network");
        Ok(data)
    }

    /// Upload a piece of private data to the network. This data will be self-encrypted.
    /// The [`DataMapChunk`] is not uploaded to the network, keeping the data private.
    ///
    /// Returns the [`DataMapChunk`] containing the map to the encrypted chunks.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use autonomi::{Client, Bytes};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let client = Client::init().await?;
    /// # let wallet = todo!();
    /// let data = Bytes::from("Hello, World");
    /// let data_map = client.data_put(data, wallet).await?;
    /// let data_fetched = client.data_get(data_map).await?;
    /// assert_eq!(data, data_fetched);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn data_put(
        &self,
        data: Bytes,
        payment_option: PaymentOption,
    ) -> Result<DataMapChunk, PutError> {
        let now = ant_networking::target_arch::Instant::now();
        let (data_map_chunk, chunks) = encrypt(data)?;
        debug!("Encryption took: {:.2?}", now.elapsed());

        // Pay for all chunks
        let xor_names: Vec<_> = chunks.iter().map(|chunk| *chunk.name()).collect();
        info!("Paying for {} addresses", xor_names.len());
        let receipt = self
            .pay_for_content_addrs(xor_names.into_iter(), payment_option)
            .await
            .inspect_err(|err| error!("Error paying for data: {err:?}"))?;

        // Upload the chunks with the payments
        debug!("Uploading {} chunks", chunks.len());

        let mut failed_uploads = self
            .upload_chunks_with_retries(chunks.iter().collect(), &receipt)
            .await;

        // Return the last chunk upload error
        if let Some(last_chunk_fail) = failed_uploads.pop() {
            tracing::error!(
                "Error uploading chunk ({:?}): {:?}",
                last_chunk_fail.0.address(),
                last_chunk_fail.1
            );
            return Err(last_chunk_fail.1);
        }

        let record_count = chunks.len();

        // Reporting
        if let Some(channel) = self.client_event_sender.as_ref() {
            let tokens_spent = receipt
                .values()
                .map(|(_, cost)| cost.as_atto())
                .sum::<Amount>();

            let summary = UploadSummary {
                record_count,
                tokens_spent,
            };
            if let Err(err) = channel.send(ClientEvent::UploadComplete(summary)).await {
                error!("Failed to send client event: {err:?}");
            }
        }

        Ok(DataMapChunk(data_map_chunk))
    }

    // Upload chunks and retry failed uploads up to `RETRY_ATTEMPTS` times.
    pub(crate) async fn upload_chunks_with_retries<'a>(
        &self,
        mut chunks: Vec<&'a Chunk>,
        receipt: &Receipt,
    ) -> Vec<(&'a Chunk, PutError)> {
        let mut current_attempt: usize = 1;

        loop {
            let mut upload_tasks = vec![];
            for chunk in chunks {
                let self_clone = self.clone();
                let address = *chunk.address();

                let Some((proof, _)) = receipt.get(chunk.name()) else {
                    debug!("Chunk at {address:?} was already paid for so skipping");
                    continue;
                };

                upload_tasks.push(async move {
                    self_clone
                        .chunk_upload_with_payment(chunk, proof.clone())
                        .await
                        .inspect_err(|err| error!("Error uploading chunk {address:?} :{err:?}"))
                        // Return chunk reference too, to re-use it next attempt/iteration
                        .map_err(|err| (chunk, err))
                });
            }
            let uploads =
                process_tasks_with_max_concurrency(upload_tasks, *CHUNK_UPLOAD_BATCH_SIZE).await;

            // Check for errors.
            let total_uploads = uploads.len();
            let uploads_failed: Vec<_> = uploads.into_iter().filter_map(|up| up.err()).collect();
            info!(
                "Uploaded {} chunks out of {total_uploads}",
                total_uploads - uploads_failed.len()
            );

            // All uploads succeeded.
            if uploads_failed.is_empty() {
                return vec![];
            }

            // Max retries reached.
            if current_attempt > RETRY_ATTEMPTS {
                return uploads_failed;
            }

            tracing::info!(
                "Retrying putting {} failed chunks (attempt {current_attempt}/3)",
                uploads_failed.len()
            );

            // Re-iterate over the failed chunks
            chunks = uploads_failed.into_iter().map(|(chunk, _)| chunk).collect();
            current_attempt += 1;
        }
    }

    /// Get a chunk from the network by its XorName
    pub(crate) async fn chunk_get(&self, xor_name: XorName) -> Result<Chunk, GetError> {
        let chunk_address = ChunkAddress::new(xor_name);
        let network_address = NetworkAddress::from_chunk_address(chunk_address);
        let get_cfg = GetRecordCfg {
            get_quorum: Quorum::One,
            retry_strategy: None,
            target_record: None,
            expected_holders: Default::default(),
            is_register: false,
        };
        let record = self
            .network
            .get_record_from_network(network_address.to_record_key(), &get_cfg)
            .await?;
        let chunk = Chunk::new(record.value.to_vec().into());
        Ok(chunk)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hex() {
        let data_map = DataMapChunk(Chunk::new(Bytes::from_static(b"hello")));
        let hex = data_map.to_hex();
        let data_map2 = DataMapChunk::from_hex(&hex).expect("Failed to decode hex");
        assert_eq!(data_map, data_map2);
    }
}
