// Copyright 2023 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use thiserror::Error;

/// Specialisation of `std::Result`.
pub type Result<T, E = Error> = std::result::Result<T, E>;

/// Transfer errors.
#[derive(Debug, Error)]
pub enum Error {
    /// No CashNotes available for spend
    #[error("No CashNotes available for spend")]
    NoCashNotesAvailable,
    /// Address provided is of the wrong type
    #[error("Invalid address type")]
    InvalidAddressType,
    /// CashNote add would overflow
    #[error("Total price exceed possible token amount")]
    TotalPriceTooHigh,
    /// Failed to create offline transfer.
    #[error("Offline transfer creation error {0}")]
    CreateOfflineTransfer(#[from] crate::transfers::Error),
    /// A general error when a transfer fails.
    #[error("Failed to send tokens due to {0}")]
    CouldNotSendMoney(String),
    /// A general error when a retrieving a store cost fails
    #[error("Failed to get store cost due to {0}")]
    CouldNotGetStoreCost(String),
    /// A general error when verifying a transfer validity in the network.
    #[error("Failed to verify transfer validity in the network {0}")]
    CouldNotVerifyTransfer(String),
    /// Failed to parse bytes into a bls key.
    #[error("Unconfirmed transactions still persist even after retries")]
    UnconfirmedTxAfterRetries,
    /// Failed to parse bytes into a bls key.
    #[error("Failed to parse bls key")]
    FailedToParseBlsKey,
    /// Failed to decode a hex string to a key.
    #[error("Could not decode hex string to key.")]
    FailedToDecodeHexToKey,
    /// Failed to serialize a main key to hex.
    #[error("Could not serialize main key to hex: {0}")]
    FailedToHexEncodeKey(String),

    #[error("CashNoteRedemption serialisation failed")]
    CashNoteRedemptionSerialisationFailed,
    #[error("CashNoteRedemption decryption failed")]
    CashNoteRedemptionDecryptionFailed,
    #[error("CashNoteRedemption encryption failed")]
    CashNoteRedemptionEncryptionFailed,
    #[error("We are not a recipient of this Transfer")]
    NotRecipient,
    #[error("Transfer serialisation failed")]
    TransferSerializationFailed,
    #[error("Transfer deserialisation failed")]
    TransferDeserializationFailed,

    /// CashNote error.
    #[error("CashNote error: {0}")]
    CashNote(#[from] crate::Error),
    /// Bls error.
    #[error("Bls error: {0}")]
    Bls(#[from] bls::error::Error),
    /// Bincode error.
    #[error("Bincode error:: {0}")]
    Bincode(#[from] bincode::Error),
    /// I/O error.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}
