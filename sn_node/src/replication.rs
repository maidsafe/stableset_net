// Copyright 2024 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use crate::{error::Result, node::Node};
use libp2p::{
    kad::{Quorum, Record, RecordKey},
    PeerId,
};
use sn_networking::{sort_peers_by_address, GetRecordCfg, Network, REPLICATION_PEERS_COUNT};
use sn_protocol::{
    messages::{Cmd, Query, QueryResponse, Request, Response},
    storage::RecordType,
    NetworkAddress, PrettyPrintRecordKey,
};
use tokio::task::spawn;

impl Node {
    /// Sends _all_ record keys every interval to all peers within the REPLICATE_RANGE.
    pub(crate) fn try_interval_replication(network: Network) {
        network.trigger_interval_replication()
    }

    /// Cleanup unrelevant records if accumulated too many.
    pub(crate) fn trigger_irrelevant_record_cleanup(network: Network) {
        network.trigger_irrelevant_record_cleanup()
    }

    /// Get the Record from a peer or from the network without waiting.
    pub(crate) fn fetch_replication_keys_without_wait(
        &self,
        keys_to_fetch: Vec<(PeerId, RecordKey)>,
    ) -> Result<()> {
        for (holder, key) in keys_to_fetch {
            let node = self.clone();
            let requester = NetworkAddress::from_peer(self.network().peer_id());
            let _handle = spawn(async move {
                let pretty_key = PrettyPrintRecordKey::from(&key).into_owned();
                debug!("Fetching record {pretty_key:?} from node {holder:?}");
                let req = Request::Query(Query::GetReplicatedRecord {
                    requester,
                    key: NetworkAddress::from_record_key(&key),
                });
                let record_opt = if let Ok(resp) = node.network().send_request(req, holder).await {
                    match resp {
                        Response::Query(QueryResponse::GetReplicatedRecord(result)) => match result
                        {
                            Ok((_holder, record_content)) => Some(record_content),
                            Err(err) => {
                                debug!("Failed fetch record {pretty_key:?} from node {holder:?}, with error {err:?}");
                                None
                            }
                        },
                        other => {
                            debug!("Cannot fetch record {pretty_key:?} from node {holder:?}, with response {other:?}");
                            None
                        }
                    }
                } else {
                    None
                };

                let record = if let Some(record_content) = record_opt {
                    Record::new(key, record_content.to_vec())
                } else {
                    debug!(
                        "Can not fetch record {pretty_key:?} from node {holder:?}, fetching from the network"
                    );
                    let get_cfg = GetRecordCfg {
                        get_quorum: Quorum::One,
                        retry_strategy: None,
                        target_record: None,
                        expected_holders: Default::default(),
                        // This is for replication, which doesn't have target_recrod to verify with.
                        // Hence value of the flag actually doesn't matter.
                        is_register: false,
                    };
                    match node
                        .network()
                        .get_record_from_network(key.clone(), &get_cfg)
                        .await
                    {
                        Ok(record) => record,
                        Err(error) => match error {
                            sn_networking::NetworkError::DoubleSpendAttempt(spends) => {
                                debug!("Failed to fetch record {pretty_key:?} from the network, double spend attempt {spends:?}");

                                let bytes = try_serialize_record(&spends, RecordKind::Spend)?;

                                Record {
                                    key,
                                    value: bytes.to_vec(),
                                    publisher: None,
                                    expires: None,
                                }
                            }
                            other_error => return Err(other_error.into()),
                        },
                    }
                };

                debug!(
                    "Got Replication Record {pretty_key:?} from network, validating and storing it"
                );
                if let Err(err) = node.store_replicated_in_record(record).await {
                    error!("During store replication fetched {pretty_key:?}, got error {err:?}");
                } else {
                    debug!("Completed storing Replication Record {pretty_key:?} from network.");
                }
                Ok::<(), Error>(())
            });
        }
        Ok(())
    }

    /// Replicate a fresh record to its close group peers.
    /// This should not be triggered by a record we receive via replicaiton fetch
    pub(crate) fn replicate_valid_fresh_record(
        &self,
        paid_key: RecordKey,
        record_type: RecordType,
    ) {
        let network = self.network().clone();

        let _handle = spawn(async move {
            let start = std::time::Instant::now();
            let pretty_key = PrettyPrintRecordKey::from(&paid_key);

            // first we wait until our own network store can return the record
            // otherwise it may not be fully written yet
            let mut retry_count = 0;
            debug!("Checking we have successfully stored the fresh record {pretty_key:?} in the store before replicating");
            loop {
                let record = match network.get_local_record(&paid_key).await {
                    Ok(record) => record,
                    Err(err) => {
                        error!(
                            "Replicating fresh record {pretty_key:?} get_record_from_store errored: {err:?}"
                        );
                        None
                    }
                };

                if record.is_some() {
                    break;
                }

                if retry_count > 10 {
                    error!(
                        "Could not get record from store for replication: {pretty_key:?} after 10 retries"
                    );
                    return;
                }

                retry_count += 1;
                tokio::time::sleep(std::time::Duration::from_millis(100)).await;
            }

            debug!("Start replication of fresh record {pretty_key:?} from store");

            // Already contains self_peer_id
            let mut closest_k_peers = match network.get_closest_k_value_local_peers().await {
                Ok(peers) => peers,
                Err(err) => {
                    error!("Replicating fresh record {pretty_key:?} get_closest_local_peers errored: {err:?}");
                    return;
                }
            };

            // remove ourself from these calculations
            closest_k_peers.retain(|peer_id| peer_id != &network.peer_id());

            let data_addr = NetworkAddress::from_record_key(&paid_key);

            let sorted_based_on_addr = match sort_peers_by_address(
                &closest_k_peers,
                &data_addr,
                REPLICATION_PEERS_COUNT,
            ) {
                Ok(result) => result,
                Err(err) => {
                    error!(
                            "When replicating fresh record {pretty_key:?}, having error when sort {err:?}"
                        );
                    return;
                }
            };

            let our_peer_id = network.peer_id();
            let our_address = NetworkAddress::from_peer(our_peer_id);
            let keys = vec![(data_addr.clone(), record_type.clone())];

            for peer_id in sorted_based_on_addr {
                debug!("Replicating fresh record {pretty_key:?} to {peer_id:?}");
                let request = Request::Cmd(Cmd::Replicate {
                    holder: our_address.clone(),
                    keys: keys.clone(),
                });

                network.send_req_ignore_reply(request, *peer_id);
            }
            debug!(
                "Completed replicate fresh record {pretty_key:?} on store, in {:?}",
                start.elapsed()
            );
        });
    }
}
