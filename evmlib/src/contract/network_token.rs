// Copyright 2024 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use crate::common::{Address, Calldata, TxHash, U256};
use crate::contract::network_token::NetworkTokenContract::NetworkTokenContractInstance;
use crate::retry::{retry, send_transaction_with_retries};
use alloy::providers::{Network, Provider};
use alloy::sol;
use alloy::transports::{RpcError, Transport, TransportErrorKind};

sol!(
    #[allow(clippy::too_many_arguments)]
    #[allow(missing_docs)]
    #[sol(rpc)]
    NetworkTokenContract,
    "artifacts/AutonomiNetworkToken.json"
);

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(transparent)]
    ContractError(#[from] alloy::contract::Error),
    #[error(transparent)]
    RpcError(#[from] RpcError<TransportErrorKind>),
    #[error(transparent)]
    PendingTransactionError(#[from] alloy::providers::PendingTransactionError),
}

pub struct NetworkToken<T: Transport + Clone, P: Provider<T, N>, N: Network> {
    pub contract: NetworkTokenContractInstance<T, P, N>,
}

impl<T, P, N> NetworkToken<T, P, N>
where
    T: Transport + Clone,
    P: Provider<T, N>,
    N: Network,
{
    /// Create a new NetworkToken contract instance.
    pub fn new(contract_address: Address, provider: P) -> Self {
        let contract = NetworkTokenContract::new(contract_address, provider);
        NetworkToken { contract }
    }

    /// Deploys the AutonomiNetworkToken smart contract to the network of the provider.
    /// ONLY DO THIS IF YOU KNOW WHAT YOU ARE DOING!
    pub async fn deploy(provider: P) -> Self {
        let contract = NetworkTokenContract::deploy(provider)
            .await
            .expect("Could not deploy contract, update anvil by running `foundryup` and try again");
        NetworkToken { contract }
    }

    pub fn set_provider(&mut self, provider: P) {
        let address = *self.contract.address();
        self.contract = NetworkTokenContract::new(address, provider);
    }

    /// Get the raw token balance of an address.
    pub async fn balance_of(&self, account: Address) -> Result<U256, Error> {
        debug!("Getting balance of account: {account:?}");
        let balance = retry(
            || async { self.contract.balanceOf(account).call().await },
            "balanceOf",
            None,
        )
        .await?
        ._0;
        debug!("Balance of account {account} is {balance}");
        Ok(balance)
    }

    /// See how many tokens are approved to be spent.
    pub async fn allowance(&self, owner: Address, spender: Address) -> Result<U256, Error> {
        debug!("Getting allowance of owner: {owner} for spender: {spender}");
        let allowance = retry(
            || async { self.contract.allowance(owner, spender).call().await },
            "allowance",
            None,
        )
        .await?
        ._0;
        debug!("Allowance of owner: {owner} for spender: {spender} is: {allowance}");
        Ok(allowance)
    }

    /// Approve spender to spend a raw amount of tokens.
    pub async fn approve(&self, spender: Address, value: U256) -> Result<TxHash, Error> {
        debug!("Approving spender {spender:?} to spend {value}");
        let (calldata, to) = self.approve_calldata(spender, value);
        send_transaction_with_retries(self.contract.provider(), calldata, to, "approve").await
    }

    /// Approve spender to spend a raw amount of tokens.
    /// Returns the transaction calldata.
    pub fn approve_calldata(&self, spender: Address, value: U256) -> (Calldata, Address) {
        let calldata = self.contract.approve(spender, value).calldata().to_owned();
        (calldata, *self.contract.address())
    }

    /// Transfer a raw amount of tokens.
    pub async fn transfer(&self, receiver: Address, amount: U256) -> Result<TxHash, Error> {
        debug!("Transferring raw amount of tokens: {amount} to {receiver:?}");
        let (calldata, to) = self.transfer_calldata(receiver, amount);
        send_transaction_with_retries(self.contract.provider(), calldata, to, "transfer").await
    }

    /// Transfer a raw amount of tokens.
    /// Returns the transaction calldata.
    pub fn transfer_calldata(&self, receiver: Address, amount: U256) -> (Calldata, Address) {
        let calldata = self
            .contract
            .transfer(receiver, amount)
            .calldata()
            .to_owned();
        (calldata, *self.contract.address())
    }
}
