// Copyright 2024 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use ant_evm::{payment_vault::error::Error as MarketPriceError, EvmWalletError};
use ant_networking::NetworkError;
use ant_protocol::NetworkAddress;
use xor_name::XorName;

/// Errors that can occur during data storage operations
#[derive(Debug, thiserror::Error)]
pub enum PutError {
    /// No wallet available for write operations
    #[error("Write operations require a wallet. Use upgrade_to_read_write to add a wallet.")]
    NoWallet,
    /// Network-related error
    #[error("Network error: {0}")]
    Network(#[from] NetworkError),
    /// Wallet-related error
    #[error("Wallet error: {0}")]
    Wallet(#[from] EvmWalletError),
    /// Data encryption error
    #[error("Encryption error: {0}")]
    Encryption(#[from] crate::self_encryption::Error),
    /// Payment-related error
    #[error("Payment error: {0}")]
    Payment(#[from] PayError),
    /// Cost estimation error
    #[error("Cost estimation error: {0}")]
    Cost(#[from] CostError),
    /// Data serialization error
    #[error("Serialization error: {0}")]
    Serialization(String),
    /// Vault owner key mismatch
    #[error("The vault owner key does not match the client's public key")]
    VaultBadOwner,
    /// Payment validation failed
    #[error("Payment unexpectedly invalid for {0:?}")]
    PaymentUnexpectedlyInvalid(NetworkAddress),
    /// No payees in payment proof
    #[error("The payment proof contains no payees.")]
    PayeesMissing,
}

/// Errors that can occur during payment operations
#[derive(Debug, thiserror::Error)]
pub enum PayError {
    /// Failed to get payment quote
    #[error("Failed to get quote: {0}")]
    GetQuote(#[from] NetworkError),
    /// Failed to pay for quote
    #[error("Failed to pay for quote: {0}")]
    PayForQuote(#[from] ant_evm::EvmError),
    /// Failed to get cost estimate
    #[error("Failed to get cost estimate: {0}")]
    Cost(#[from] CostError),
    /// Failed to process wallet operation
    #[error("Failed to process wallet operation: {0}")]
    Wallet(#[from] EvmWalletError),
}

/// Errors that can occur during data retrieval
#[derive(Debug, thiserror::Error)]
pub enum GetError {
    /// Network-related error
    #[error("Network error: {0}")]
    Network(#[from] NetworkError),
    /// Data decryption error
    #[error("Failed to decrypt data")]
    Decryption(#[from] crate::self_encryption::Error),
    /// Invalid data map
    #[error("Failed to deserialize data map: {0}")]
    InvalidDataMap(String),
    /// Deserialization error
    #[error("Deserialization error: {0}")]
    Deserialization(String),
}

/// Errors that can occur during cost estimation
#[derive(Debug, thiserror::Error)]
pub enum CostError {
    /// Failed to get storage quote
    #[error("Failed to get quote: {0}")]
    GetQuote(#[from] NetworkError),
    /// Data encryption error during cost estimation
    #[error("Failed to encrypt data: {0}")]
    Encryption(#[from] crate::self_encryption::Error),
    /// Not enough node quotes received
    #[error("Not enough node quotes received: got {got} but need {need} for {addr:?}")]
    NotEnoughNodeQuotes {
        addr: XorName,
        got: usize,
        need: usize,
    },
    /// Could not get store quote for content
    #[error("Could not get store quote for content: {0:?}")]
    CouldNotGetStoreQuote(NetworkAddress),
    /// Market price error
    #[error("Failed to get market price: {0}")]
    MarketPrice(#[from] MarketPriceError),
    /// Data serialization error during cost estimation
    #[error("Serialization error: {0}")]
    Serialization(String),
}

impl From<anyhow::Error> for PutError {
    fn from(err: anyhow::Error) -> Self {
        Self::Serialization(err.to_string())
    }
}

impl From<rmp_serde::decode::Error> for GetError {
    fn from(err: rmp_serde::decode::Error) -> Self {
        GetError::Deserialization(err.to_string())
    }
}
