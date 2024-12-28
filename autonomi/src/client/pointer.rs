use super::Client;
use crate::client::error::{CostError, GetError, PayError, PutError};
use crate::client::payment::{PaymentOption, Receipt};
use crate::client::utils::process_tasks_with_max_concurrency;
use crate::client::{ClientEvent, UploadSummary};
use crate::self_encryption::{self, encrypt};
use ant_evm::{Amount, AttoTokens, EvmWallet, EvmWalletError};
use ant_networking::{GetRecordCfg, NetworkError, PutRecordCfg, VerificationKind};
use ant_protocol::storage::{
    try_serialize_record, Chunk, ChunkAddress, Pointer, PointerAddress, RecordKind, RetryStrategy,
};
use ant_protocol::NetworkAddress;
use bls::SecretKey;
use bytes::Bytes;
use libp2p::kad::{Quorum, Record};
use serde::{Deserialize, Serialize};
use tracing::{debug, error, info, trace};
use xor_name::XorName;

#[derive(Debug, thiserror::Error)]
pub enum PointerError {
    #[error("Cost error: {0}")]
    Cost(#[from] CostError),
    #[error("Network error")]
    Network(#[from] NetworkError),
    #[error("Serialization error")]
    Serialization,
    #[error("Pointer could not be verified (corrupt)")]
    FailedVerification,
    #[error("Payment failure occurred during pointer creation.")]
    Pay(#[from] PayError),
    #[error("Failed to retrieve wallet payment")]
    Wallet(#[from] EvmWalletError),
    #[error("Received invalid quote from node, this node is possibly malfunctioning, try another node by trying another pointer name")]
    InvalidQuote,
    #[error("Pointer already exists at this address: {0:?}")]
    PointerAlreadyExists(PointerAddress),
}

impl Client {
    /// Get a pointer from the network
    pub async fn pointer_get(&self, address: PointerAddress) -> Result<Pointer, PointerError> {
        let key = NetworkAddress::from_pointer_address(address).to_record_key();
        let record = self.network.get_local_record(&key).await?;

        match record {
            Some(record) => {
                let (_, pointer): (Vec<u8>, Pointer) = rmp_serde::from_slice(&record.value)
                    .map_err(|_| PointerError::Serialization)?;
                Ok(pointer)
            }
            None => Err(PointerError::FailedVerification),
        }
    }

    /// Store a pointer on the network
    pub async fn pointer_put(
        &self,
        pointer: Pointer,
        wallet: &EvmWallet,
    ) -> Result<(), PointerError> {
        let address = pointer.network_address();

        // pay for the pointer storage
        let xor_name = *address.xorname();
        debug!("Paying for pointer at address: {address:?}");
        let payment_proofs = self
            .pay(std::iter::once(xor_name), wallet)
            .await
            .inspect_err(|err| {
                error!("Failed to pay for pointer at address: {address:?} : {err}")
            })?;

        // verify payment was successful
        let (proof, _price) = match payment_proofs.get(&xor_name) {
            Some((proof, price)) => (proof, price),
            None => {
                error!("Pointer at address: {address:?} was already paid for");
                return Err(PointerError::PointerAlreadyExists(address));
            }
        };

        let payees = proof.payees();

        let record = Record {
            key: NetworkAddress::from_pointer_address(address).to_record_key(),
            value: try_serialize_record(&(proof, &pointer), RecordKind::PointerWithPayment)
                .map_err(|_| PointerError::Serialization)?
                .to_vec(),
            publisher: None,
            expires: None,
        };

        let get_cfg = GetRecordCfg {
            get_quorum: Quorum::Majority,
            retry_strategy: Some(RetryStrategy::default()),
            target_record: None,
            expected_holders: Default::default(),
            is_register: false,
        };

        let put_cfg = PutRecordCfg {
            put_quorum: Quorum::All,
            retry_strategy: None,
            verification: Some((VerificationKind::Crdt, get_cfg)),
            use_put_record_to: Some(payees),
        };

        // store the pointer on the network
        debug!("Storing pointer at address {address:?} to the network");
        self.network
            .put_record(record, &put_cfg)
            .await
            .inspect_err(|err| {
                error!("Failed to put record - pointer {address:?} to the network: {err}")
            })?;

        Ok(())
    }

    /// Calculate the cost of storing a pointer
    pub async fn pointer_cost(&self, key: SecretKey) -> Result<AttoTokens, PointerError> {
        let pk = key.public_key();
        trace!("Getting cost for pointer of {pk:?}");

        let address = PointerAddress::from_owner(pk);
        let xor = *address.xorname();
        let store_quote = self.get_store_quotes(std::iter::once(xor)).await?;
        let total_cost = AttoTokens::from_atto(
            store_quote
                .0
                .values()
                .map(|quote| quote.price())
                .sum::<Amount>(),
        );
        debug!("Calculated the cost to create pointer of {pk:?} is {total_cost}");
        Ok(total_cost)
    }
}
