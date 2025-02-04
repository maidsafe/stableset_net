// Copyright 2024 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use crate::{NetworkError, Result};
use ant_protocol::storage::{DataTypes, GraphEntry};
use ant_protocol::{
    storage::{try_deserialize_record, RecordHeader, RecordKind},
    PrettyPrintRecordKey,
};
use libp2p::kad::Record;

pub fn get_graph_entry_from_record(record: &Record) -> Result<Vec<GraphEntry>> {
    let header = RecordHeader::from_record(record)?;
    if let RecordKind::DataOnly(DataTypes::GraphEntry) = header.kind {
        let entry = try_deserialize_record::<Vec<GraphEntry>>(record)?;
        Ok(entry)
    } else {
        warn!(
            "RecordKind mismatch while trying to retrieve graph_entry from record {:?}",
            PrettyPrintRecordKey::from(&record.key)
        );
        Err(NetworkError::RecordKindMismatch(RecordKind::DataOnly(
            DataTypes::GraphEntry,
        )))
    }
}
