// Copyright 2024 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use crate::client::error::{CostError, PayError};
use crate::client::{ClientEvent, UploadSummary};
use crate::Client;
use ant_evm::{Amount, AttoTokens, EvmWallet, EvmWalletError};
use ant_networking::{GetRecordCfg, NetworkError, PutRecordCfg, VerificationKind};
use ant_protocol::storage::{
    try_serialize_record, LinkedList, LinkedListAddress, RecordKind,
    RetryStrategy,
};
use ant_protocol::NetworkAddress;
pub use bls::SecretKey;
use libp2p::kad::{Quorum, Record};
use tracing::{debug, error, trace};

#[derive(Debug, thiserror::Error)]
pub enum LinkedListError {
    #[error("Cost error: {0}")]
    Cost(#[from] CostError),
    #[error("Network error")]
    Network(#[from] NetworkError),
    #[error("Serialization error")]
    Serialization,
    #[error("Linked list could not be verified (corrupt)")]
    FailedVerification,
    #[error("Payment failure occurred during linked list creation.")]
    Pay(#[from] PayError),
    #[error("Failed to retrieve wallet payment")]
    Wallet(#[from] EvmWalletError),
    #[error("Received invalid quote from node, this node is possibly malfunctioning, try another node by trying another linked list name")]
    InvalidQuote,
    #[error("Linked list already exists at this address: {0:?}")]
    LinkedListAlreadyExists(LinkedListAddress),
}

impl Client {
    /// Fetches a Linked List from the network.
    pub async fn linked_list_get(
        &self,
        address: LinkedListAddress,
    ) -> Result<Vec<LinkedList>, LinkedListError> {
        let linked_lists = self.network.get_linked_list(address).await?;

        Ok(linked_lists)
    }

    pub async fn linked_list_put(
        &self,
        linked_list: LinkedList,
        wallet: &EvmWallet,
    ) -> Result<(), LinkedListError> {
        let address = linked_list.address();

        // pay for the linked list
        let xor_name = address.xorname();
        debug!("Paying for linked list at address: {address:?}");
        let payment_proofs = self
            .pay(std::iter::once(*xor_name), wallet)
            .await
            .inspect_err(|err| {
                error!("Failed to pay for linked list at address: {address:?} : {err}")
            })?;

        // make sure the linked list was paid for
        let (proof, price) = match payment_proofs.get(xor_name) {
            Some((proof, price)) => (proof, price),
            None => {
                // linked list was skipped, meaning it was already paid for
                error!("Linked list at address: {address:?} was already paid for");
                return Err(LinkedListError::LinkedListAlreadyExists(address));
            }
        };

        // prepare the record for network storage
        let payees = proof.payees();
        let record = Record {
            key: NetworkAddress::from_linked_list_address(address).to_record_key(),
            value: try_serialize_record(&(proof, &linked_list), RecordKind::LinkedListWithPayment)
                .map_err(|_| LinkedListError::Serialization)?
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
            use_put_record_to: Some(payees),
            verification: Some((VerificationKind::Crdt, get_cfg)),
        };

        // put the record to the network
        debug!("Storing linked list at address {address:?} to the network");
        self.network
            .put_record(record, &put_cfg)
            .await
            .inspect_err(|err| {
                error!("Failed to put record - linked list {address:?} to the network: {err}")
            })?;

        // send client event
        if let Some(channel) = self.client_event_sender.as_ref() {
            let summary = UploadSummary {
                record_count: 1,
                tokens_spent: price.as_atto(),
            };
            if let Err(err) = channel.send(ClientEvent::UploadComplete(summary)).await {
                error!("Failed to send client event: {err}");
            }
        }

        Ok(())
    }

    /// Get the cost to create a linked list
    pub async fn linked_list_cost(&self, key: SecretKey) -> Result<AttoTokens, LinkedListError> {
        let pk = key.public_key();
        trace!("Getting cost for linked list of {pk:?}");

        let address = LinkedListAddress::from_owner(pk);
        let xor = *address.xorname();
        let store_quote = self.get_store_quotes(std::iter::once(xor)).await?;
        let total_cost = AttoTokens::from_atto(
            store_quote
                .0
                .values()
                .map(|quote| quote.price())
                .sum::<Amount>(),
        );
        debug!("Calculated the cost to create linked list of {pk:?} is {total_cost}");
        Ok(total_cost)
    }
}
