// Copyright 2023 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use super::{
    error::{Error, Result},
    msg::MsgCodec,
    SwarmDriver,
};

use crate::protocol::messages::{Request, Response};
use core::cmp::Ordering;
use itertools::Itertools;
use libp2p::{
    kad::{kbucket, store::MemoryStore, Kademlia, KademliaEvent, QueryResult, K_VALUE},
    mdns,
    multiaddr::Protocol,
    request_response::{self, ResponseChannel},
    swarm::{NetworkBehaviour, SwarmEvent},
    PeerId,
};
use std::collections::{BTreeSet, HashSet};
use tracing::{info, warn};
use xor_name::XorName;

#[derive(NetworkBehaviour)]
#[behaviour(out_event = "NodeEvent")]
pub(super) struct NodeBehaviour {
    pub(super) request_response: request_response::Behaviour<MsgCodec>,
    pub(super) kademlia: Kademlia<MemoryStore>,
    pub(super) mdns: mdns::tokio::Behaviour,
}

#[derive(Debug)]
pub(super) enum NodeEvent {
    RequestResponse(request_response::Event<Request, Response>),
    Kademlia(KademliaEvent),
    Mdns(Box<mdns::Event>),
}

impl From<request_response::Event<Request, Response>> for NodeEvent {
    fn from(event: request_response::Event<Request, Response>) -> Self {
        NodeEvent::RequestResponse(event)
    }
}

impl From<KademliaEvent> for NodeEvent {
    fn from(event: KademliaEvent) -> Self {
        NodeEvent::Kademlia(event)
    }
}

impl From<mdns::Event> for NodeEvent {
    fn from(event: mdns::Event) -> Self {
        NodeEvent::Mdns(Box::new(event))
    }
}

#[derive(Debug)]
/// Events forwarded by the underlying Network; to be used by the upper layers
pub enum NetworkEvent {
    /// Incoming `Request` from a peer
    RequestReceived {
        /// Request
        req: Request,
        /// The channel to send the `Response` through
        channel: ResponseChannel<Response>,
    },
    /// Emitted when the DHT is updated
    PeerAdded {
        /// Our own id
        own_id: PeerId,
        /// ID of the added peer
        added_peer: PeerId,
    },
}

impl SwarmDriver {
    // Handle `SwarmEvents`
    pub(super) async fn handle_swarm_events<EventError: std::error::Error>(
        &mut self,
        event: SwarmEvent<NodeEvent, EventError>,
    ) -> Result<()> {
        trace!("Handling a swarm event {event:?}");
        match event {
            // handle RequestResponse events
            SwarmEvent::Behaviour(NodeEvent::RequestResponse(event)) => {
                if let Err(e) = self.handle_msg(event).await {
                    warn!("RequestResponseError: {e:?}");
                }
            }
            // handle Kademlia events
            SwarmEvent::Behaviour(NodeEvent::Kademlia(event)) => match event {
                KademliaEvent::OutboundQueryProgressed {
                    id,
                    result: QueryResult::GetClosestPeers(Ok(closest_peers)),
                    stats,
                    step,
                } => {
                    trace!("Query task {id:?} returned with peers {closest_peers:?}, {stats:?} - {step:?}");

                    if let Some((sender, mut current_closest, key)) =
                        self.pending_get_closest_peers.remove(&id)
                    {
                        // TODO: consider order the result and terminate when reach any of the
                        //       following creterias:
                        //   1, `stats.num_pending()` is 0
                        //   2, `stats.duration()` is longer than a defined period
                        //   3, `step.last` is true
                        let new_peers: HashSet<PeerId> = closest_peers.peers.into_iter().collect();
                        current_closest.extend(new_peers);

                        if current_closest.contains(self.swarm.local_peer_id()) {
                            trace!("Own id appears in the network closest_peers");
                        }

                        if current_closest.len() >= usize::from(K_VALUE) || step.last {
                            let local_closest_peers: HashSet<PeerId> = self
                                .swarm
                                .behaviour_mut()
                                .kademlia
                                .get_closest_local_peers(&kbucket::Key::new(key.clone()))
                                .map(|key| *key.preimage())
                                .take(usize::from(K_VALUE))
                                .collect();

                            trace!(
                                "Got {:?} closest peers from network and {:?} from local",
                                current_closest.len(),
                                local_closest_peers.len()
                            );
                            if current_closest.len() > local_closest_peers.len() {
                                warn!("Network knowledge is aheading of local knowledge");
                            } else if current_closest.len() < local_closest_peers.len() {
                                warn!("Local knowledge is aheading of network knowledge");
                            } else {
                                let network: BTreeSet<PeerId> =
                                    current_closest.iter().cloned().collect();
                                let local: BTreeSet<PeerId> =
                                    local_closest_peers.iter().cloned().collect();
                                if network != local {
                                    let presented_in_network_only: BTreeSet<_> = network
                                        .iter()
                                        .filter(|peer_id| !local.contains(peer_id))
                                        .map(|peer_id| {
                                            let mut bytes: [u8; 32] = [0; 32];
                                            let _ = bytes
                                                .iter_mut()
                                                .set_from(peer_id.to_bytes().iter().cloned());
                                            XorName(bytes)
                                        })
                                        .collect();
                                    let presented_in_local_only: BTreeSet<_> = local
                                        .iter()
                                        .filter(|peer_id| !network.contains(peer_id))
                                        .map(|peer_id| {
                                            let mut bytes: [u8; 32] = [0; 32];
                                            let _ = bytes
                                                .iter_mut()
                                                .set_from(peer_id.to_bytes().iter().cloned());
                                            XorName(bytes)
                                        })
                                        .collect();
                                    let mut bytes: [u8; 32] = [0; 32];
                                    let _ = bytes.iter_mut().set_from(key.iter().cloned());
                                    let target_name = XorName(bytes);
                                    warn!("target_name is {target_name:?}");
                                    warn!("Closest peers presented in network only: {presented_in_network_only:?}");
                                    warn!("Closest peers presented in local only: {presented_in_local_only:?}");
                                    if presented_in_network_only.iter().any(|a| {
                                        presented_in_local_only.iter().any(|b| {
                                            target_name.cmp_distance(b, a) == Ordering::Less
                                        })
                                    }) {
                                        warn!("Local closest peers contains entry closer than network");
                                    } else {
                                        warn!("All network closest peers are closer than local");
                                    }

                                    warn!("Got closest_peers from network {network:?}");
                                    warn!("Got closest_peers from local   {local:?}");
                                }
                            }

                            sender.send(current_closest).map_err(|_| {
                                Error::Other("Receiver not to be dropped".to_string())
                            })?;
                        } else {
                            let _ = self
                                .pending_get_closest_peers
                                .insert(id, (sender, current_closest, key));
                        }
                    } else {
                        trace!("Can't locate query task {id:?}, shall be completed already.");
                    }
                }
                KademliaEvent::RoutingUpdated {
                    peer, is_new_peer, ..
                } => {
                    if is_new_peer {
                        let own_id = *self.swarm.local_peer_id();
                        self.event_sender
                            .send(NetworkEvent::PeerAdded {
                                own_id,
                                added_peer: peer,
                            })
                            .await?;
                    }
                }
                KademliaEvent::InboundRequest { request } => {
                    info!("got inbound request: {request:?}");
                }
                todo => {
                    error!("KademliaEvent has not been implemented: {todo:?}");
                }
            },
            SwarmEvent::Behaviour(NodeEvent::Mdns(mdns_event)) => match *mdns_event {
                mdns::Event::Discovered(list) => {
                    for (peer_id, multiaddr) in list {
                        info!("Node discovered: {multiaddr:?}");
                        let _routing_update = self
                            .swarm
                            .behaviour_mut()
                            .kademlia
                            .add_address(&peer_id, multiaddr);
                    }
                    let own_id = *self.swarm.local_peer_id();
                    self.event_sender
                        .send(NetworkEvent::PeerAdded {
                            own_id,
                            added_peer: own_id,
                        })
                        .await?;
                }
                mdns::Event::Expired(_) => {
                    info!("mdns peer expired");
                }
            },
            SwarmEvent::NewListenAddr { address, .. } => {
                let local_peer_id = *self.swarm.local_peer_id();
                info!(
                    "Local node is listening on {:?}",
                    address.with(Protocol::P2p(local_peer_id.into()))
                );
            }
            SwarmEvent::IncomingConnection { .. } => {}
            SwarmEvent::ConnectionEstablished {
                peer_id, endpoint, ..
            } => {
                if endpoint.is_dialer() {
                    info!("Connected with {peer_id:?}");
                    if let Some(sender) = self.pending_dial.remove(&peer_id) {
                        let _ = sender.send(Ok(()));
                    }
                }
            }
            SwarmEvent::ConnectionClosed { .. } => {}
            SwarmEvent::OutgoingConnectionError { peer_id, error, .. } => {
                if let Some(peer_id) = peer_id {
                    if let Some(sender) = self.pending_dial.remove(&peer_id) {
                        let _ = sender.send(Err(error.into()));
                    }
                }
            }
            SwarmEvent::IncomingConnectionError { .. } => {}
            SwarmEvent::Dialing(peer_id) => info!("Dialing {peer_id}"),
            todo => error!("SwarmEvent has not been implemented: {todo:?}"),
        }
        Ok(())
    }
}
