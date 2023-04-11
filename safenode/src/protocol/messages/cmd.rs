// Copyright 2023 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use super::RegisterCmd;

use crate::protocol::types::{
    address::{dbc_address, ChunkAddress, DataAddress},
    chunk::Chunk,
};

use sn_dbc::{DbcTransaction, SignedSpend};

use serde::{Deserialize, Serialize};

/// Data and Dbc cmds - recording spends or creating, updating, and removing data.
///
/// See the [`types`] module documentation for more details of the types supported by the Safe
/// Network, and their semantics.
///
/// [`types`]: crate::protocol::types
#[allow(clippy::large_enum_variant)]
#[derive(Eq, PartialEq, Clone, Serialize, Deserialize, Debug)]
pub enum Cmd {
    /// [`SignedSpend`] write operation.
    ///
    /// [`SignedSpend`]: sn_dbc::SignedSpend
    Dbc {
        /// The spend to be recorded.
        /// It contains the transaction it is being spent in.
        signed_spend: Box<SignedSpend>,
        /// The transaction that this spend was created in.
        source_tx: Box<DbcTransaction>,
    },
    /// [`Chunk`] write operation.
    ///
    /// [`Chunk`]: crate::protocol::types::chunk::Chunk
    StoreChunk(Chunk),
    /// [`Register`] write operation.
    ///
    /// [`Register`]: crate::protocol::types::register::Register
    Register(RegisterCmd),
}

impl Cmd {
    /// Used to send a cmd to the close group of the address.
    pub fn dst(&self) -> DataAddress {
        match self {
            Cmd::StoreChunk(chunk) => DataAddress::Chunk(ChunkAddress::new(*chunk.name())),
            Cmd::Register(cmd) => DataAddress::Register(cmd.dst()),
            Cmd::Dbc { signed_spend, .. } => DataAddress::Spend(dbc_address(signed_spend.dbc_id())),
        }
    }
}
