// Copyright 2024 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use std::path::PathBuf;

use super::Client;
use ant_evm::Amount;
use serde::Serialize;
use xor_name::XorName;

/// Events that can be broadcasted by the client.
#[derive(Debug, Clone)]
pub enum ClientEvent {
    /// Summary of a payment operation. This event is not emitted if the payment fails.
    PaymentSucceeded(PaymentSummary),
    /// The record has been uploaded to the network. This event is not emitted if the record was
    /// already present on the network. Refer to [`Self::DataAlreadyPresent`] for that case.
    UploadSucceeded(XorName),
    /// The record is already present on the network. No payment was made for this record.
    DataAlreadyPresent(XorName),
    /// Failed to upload the record to the network.
    UploadFailed(XorName),
    /// The record has been downloaded from the network.
    DownloadSucceeded(XorName),
    /// Failed to download the record from the network.
    DownloadFailed(XorName),
    /// File event.
    File(FileEvent),
}

/// High level file events.
#[derive(Debug, Clone)]
pub enum FileEvent {
    UploadingFile { path: PathBuf, public: bool },
}

/// Summary of a payment operation.
/// This event is emitted even if the payment fails.
#[derive(Debug, Clone, Serialize)]
pub struct PaymentSummary {
    /// Amount of tokens that were spent.
    pub tokens_spent: Amount,
    /// Records that were paid for.
    pub records_paid: usize,
    /// Records that were already present on the network. No payment were made for these.
    pub records_already_paid: usize,
}

impl Client {
    /// Emit a client Download event based on the success of the operation.
    pub(crate) async fn emit_download_event(&self, xor_name: XorName, success: bool) {
        if let Some(channel) = self.client_event_sender.as_ref() {
            let event = if success {
                ClientEvent::DownloadSucceeded(xor_name)
            } else {
                ClientEvent::DownloadFailed(xor_name)
            };

            if let Err(err) = channel.send(event).await {
                error!("Failed to send client event: {err:?}");
            }
        }
    }

    /// Emit a client Upload event based on the success of the operation.
    pub(crate) async fn emit_upload_event(&self, xor_name: XorName, success: bool) {
        if let Some(channel) = self.client_event_sender.as_ref() {
            let event = if success {
                ClientEvent::UploadSucceeded(xor_name)
            } else {
                ClientEvent::UploadFailed(xor_name)
            };

            if let Err(err) = channel.send(event).await {
                error!("Failed to send client event: {err:?}");
            }
        }
    }
}
