// Copyright 2023 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use crate::error::Result;
use crate::Node;
use libp2p::{kad::KBucketKey, PeerId};
use sn_networking::{sort_peers_by_address, sort_peers_by_key, CLOSE_GROUP_SIZE};
use sn_protocol::{
    messages::{Cmd, Query, Request},
    NetworkAddress,
};
use std::collections::{BTreeMap, HashSet};

// To reduce the number of messages exchanged, patch max 500 replication keys into one request.
const MAX_PRELICATION_KEYS_PER_REQUEST: usize = 500;

// Defines how close that a node will trigger repliation.
// That is, the node has to be among the REPLICATION_RANGE closest to data,
// to carry out the replication.
const REPLICATION_RANGE: usize = 8;

impl Node {
    /// Replication is triggered when the newly added peer or the dead peer was among our closest.
    pub(crate) async fn try_trigger_replication(
        &mut self,
        peer: &PeerId,
        is_dead_peer: bool,
    ) -> Result<()> {
        let our_address = NetworkAddress::from_peer(self.network.peer_id);
        trace!(
            "Self peer id {:?} converted to {our_address:?}",
            self.network.peer_id
        );
        // Fetch from local shall be enough.
        let closest_peers = self.network.get_closest_local_peers(&our_address).await?;
        if !closest_peers.iter().any(|key| key == peer) {
            return Ok(());
        }

        let mut all_peers = self.network.get_all_local_peers().await?;
        if all_peers.len() <= CLOSE_GROUP_SIZE {
            return Ok(());
        }

        // Setup the record storage distance range.
        let sorted_peers: Vec<PeerId> = if let Ok(sorted_peers) =
            sort_peers_by_address(all_peers.clone(), &our_address, CLOSE_GROUP_SIZE + 1)
        {
            sorted_peers
        } else {
            return Ok(());
        };
        let distance_bar =
            NetworkAddress::from_peer(sorted_peers[CLOSE_GROUP_SIZE]).distance(&our_address);
        self.network.set_record_distance_range(distance_bar).await?;

        all_peers.push(self.network.peer_id);
        let churned_peer_address = NetworkAddress::from_peer(*peer);
        // Only nearby peers (two times of the CLOSE_GROUP_SIZE) may affect the later on
        // calculation of `closest peers to each entry`.
        // Hecence to reduce the computation work, no need to take all peers.
        // Plus 1 because the result contains self.
        let sorted_peers: Vec<PeerId> = if let Ok(sorted_peers) =
            sort_peers_by_address(all_peers, &churned_peer_address, 2 * CLOSE_GROUP_SIZE + 1)
        {
            sorted_peers
        } else {
            return Ok(());
        };

        let distance_bar = NetworkAddress::from_peer(sorted_peers[CLOSE_GROUP_SIZE])
            .distance(&churned_peer_address);

        // The fetched entries are records that supposed to be held by the churned_peer.
        let entries_to_be_replicated = self
            .network
            .get_record_keys_closest_to_target(&churned_peer_address, distance_bar)
            .await?;

        let mut replications: BTreeMap<PeerId, Vec<NetworkAddress>> = Default::default();
        for key in entries_to_be_replicated.iter() {
            let record_key = KBucketKey::from(key.to_vec());
            let closest_peers: Vec<_> = if let Ok(sorted_peers) =
                sort_peers_by_key(sorted_peers.clone(), &record_key, CLOSE_GROUP_SIZE + 1)
            {
                sorted_peers
            } else {
                continue;
            };

            // Only carry out replication when self within REPLICATION_RANGE
            let replicate_range = NetworkAddress::from_peer(closest_peers[REPLICATION_RANGE]);
            if our_address.as_kbucket_key().distance(&record_key)
                >= replicate_range.as_kbucket_key().distance(&record_key)
            {
                continue;
            }

            let dsts = if is_dead_peer {
                // To ensure more copies to be retained across the network,
                // make all closest_peers as target in case of peer drop out.
                // This can be reduced depends on the performance.
                closest_peers
            } else {
                vec![*peer]
            };

            for peer in dsts {
                let keys_to_replicate = replications.entry(peer).or_insert(Default::default());
                keys_to_replicate.push(NetworkAddress::from_record_key(key.clone()));
            }
        }

        let _ = replications.remove(&self.network.peer_id);
        if is_dead_peer {
            let _ = replications.remove(peer);
        }

        for (peer_id, keys) in replications {
            let (left, mut remaining_keys) = keys.split_at(0);
            trace!("Left len {:?}", left.len());
            trace!("Remaining keys len {:?}", remaining_keys.len());
            while remaining_keys.len() > MAX_PRELICATION_KEYS_PER_REQUEST {
                let (left, right) = remaining_keys.split_at(MAX_PRELICATION_KEYS_PER_REQUEST);
                remaining_keys = right;
                self.send_replicate_list_without_wait(&our_address, &peer_id, left.to_vec())
                    .await?;
            }
            self.send_replicate_list_without_wait(&our_address, &peer_id, remaining_keys.to_vec())
                .await?;
        }
        Ok(())
    }

    fn send_replicate_list_without_wait(
        &mut self,
        our_address: &NetworkAddress,
        peer_id: &PeerId,
        keys: Vec<NetworkAddress>,
    ) {
        let len = keys.len();
        let request = Request::Cmd(Cmd::Replicate {
            holder: our_address.clone(),
            keys,
        });
        let request_id = self
            .swarm
            .behaviour_mut()
            .request_response
            .send_request(peer_id, request);
        trace!("Sending a replication list({request_id:?}) with {len:?} keys to {peer_id:?}");
    }

    /// Notify a list of keys within a holder to be replicated to self.
    /// The `chunk_storage` is currently held by `swarm_driver` within `network` instance.
    /// Hence has to carry out this notification.
    pub(crate) async fn replication_keys_to_fetch(
        &mut self,
        holder: NetworkAddress,
        keys: Vec<NetworkAddress>,
    ) -> Result<()> {
        let peer_id = if let Some(peer_id) = holder.as_peer_id() {
            peer_id
        } else {
            warn!("Cann't parse PeerId from NetworkAddress {holder:?}");
            return Ok(());
        };
        trace!("Convert {holder:?} to {peer_id:?}");
        let existing_keys: HashSet<NetworkAddress> =
            self.network.get_all_local_record_addresses().await?;
        let non_existing_keys: Vec<NetworkAddress> = keys
            .iter()
            .filter(|key| !existing_keys.contains(key))
            .cloned()
            .collect();
        let keys_to_fetch = self
            .network
            .add_keys_to_replication_fetcher(peer_id, non_existing_keys)
            .await?;
        self.fetching_replication_keys(keys_to_fetch).await?;
        Ok(())
    }

    pub(crate) async fn fetching_replication_keys(
        &mut self,
        keys_to_fetch: Vec<(PeerId, NetworkAddress)>,
    ) -> Result<()> {
        for (peer, key) in keys_to_fetch {
            trace!("Fetching replication {key:?} from {peer:?}");
            let request = Request::Query(Query::GetReplicatedData {
                requester: NetworkAddress::from_peer(self.network.peer_id),
                address: key,
            });
            self.network.send_req_ignore_reply(request, peer).await?
        }
        Ok(())
    }
}
