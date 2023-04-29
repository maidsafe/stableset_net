// Copyright 2023 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use super::{
    error::{Error, Result},
    event::NodeEventsChannel,
    Network, Node, NodeEvent, NodeId,
};

use crate::{
    domain::{
        dbc_genesis::is_genesis_parent_tx,
        node_transfers::{Error as TransferError, Transfers},
        storage::{
            dbc_address, register::User, ChunkStorage, DbcAddress, Error as StorageError,
            RegisterStorage,
        },
        wallet::LocalWallet,
    },
    network::{close_group_majority, MsgResponder, NetworkEvent, SwarmDriver, SwarmLocalState},
    protocol::{
        error::Error as ProtocolError,
        messages::{
            Cmd, CmdResponse, Event, Query, QueryResponse, RegisterCmd, Request, Response,
            SpendQuery,
        },
        NetworkKey,
    },
};

use sn_dbc::{DbcTransaction, SignedSpend};

use async_recursion::async_recursion;
use futures::future::join_all;
use libp2p::{Multiaddr, PeerId};
use std::{collections::BTreeSet, net::SocketAddr, path::Path};
use tokio::task::spawn;

// Replicated data will all be sent in one message,
// as such we want to keep size fairly low.
const MAX_REPLICATION_BATCH_SIZE: usize = 25;

/// Once a node is started and running, the user obtains
/// a `NodeRunning` object which can be used to interact with it.
pub struct RunningNode {
    network: Network,
    node_events_channel: NodeEventsChannel,
}

impl RunningNode {
    /// Returns this node's `PeerId`
    pub fn peer_id(&self) -> PeerId {
        self.network.peer_id
    }

    /// Returns a `SwarmLocalState` with some information obtained from swarm's local state.
    pub async fn get_swarm_local_state(&self) -> Result<SwarmLocalState> {
        let state = self.network.get_swarm_local_state().await?;
        Ok(state)
    }

    /// Returns the node events channel where to subscribe to receive `NodeEvent`s
    pub fn node_events_channel(&self) -> &NodeEventsChannel {
        &self.node_events_channel
    }
}

impl Node {
    /// Asynchronously runs a new node instance, setting up the swarm driver,
    /// creating a data storage, and handling network events. Returns the
    /// created node and a `NodeEventsChannel` for listening to node-related
    /// events.
    ///
    /// # Returns
    ///
    /// A tuple containing a `Node` instance and a `NodeEventsChannel`.
    ///
    /// # Errors
    ///
    /// Returns an error if there is a problem initializing the `SwarmDriver`.
    pub async fn run(
        addr: SocketAddr,
        initial_peers: Vec<(PeerId, Multiaddr)>,
        root_dir: &Path,
    ) -> Result<RunningNode> {
        let (network, mut network_event_receiver, swarm_driver) = SwarmDriver::new(addr)?;
        let node_events_channel = NodeEventsChannel::default();

        let node_id = NodeId::from(network.peer_id);
        let node_wallet = LocalWallet::load_from(root_dir)
            .await
            .map_err(|e| Error::CouldNotLoadWallet(e.to_string()))?;

        let node = Self {
            network: network.clone(),
            chunks: ChunkStorage::new(root_dir),
            registers: RegisterStorage::new(root_dir),
            transfers: Transfers::new(root_dir, node_id, node_wallet),
            events_channel: node_events_channel.clone(),
            initial_peers,
        };

        let _handle = spawn(swarm_driver.run());
        let _handle = spawn(async move {
            loop {
                let event = match network_event_receiver.recv().await {
                    Some(event) => event,
                    None => {
                        error!("The `NetworkEvent` channel has been closed");
                        continue;
                    }
                };
                if let Err(err) = node.handle_network_event(event).await {
                    warn!("Error handling network event: {err}");
                }
            }
        });

        Ok(RunningNode {
            network,
            node_events_channel,
        })
    }

    async fn handle_network_event(&self, event: NetworkEvent) -> Result<()> {
        match event {
            NetworkEvent::RequestReceived { req, channel } => {
                self.handle_request(req, channel).await?
            }
            NetworkEvent::PeerAdded(peer) => {
                trace!("Peer {peer:?} added.");
                self.events_channel.broadcast(NodeEvent::ConnectedToNetwork);
                let key = NetworkKey::from_peer(peer);
                let network = self.network.clone();
                let _handle = spawn(async move {
                    trace!("Getting closest peers for target {key:?}...");
                    let result = network.node_query_for_closest_peers(&key).await;
                    trace!("Closest peers to {key:?} got: {result:?}.");
                });
            }
            NetworkEvent::PeersAdded(peers) => {
                trace!("{} peers added.", peers.len());
                self.events_channel.broadcast(NodeEvent::ConnectedToNetwork);
                for peer in peers {
                    let key = NetworkKey::from_peer(peer);
                    let network = self.network.clone();
                    let _handle = spawn(async move {
                        trace!("Getting closest peers for target {key:?}...");
                        let result = network.node_query_for_closest_peers(&key).await;
                        trace!("Closest peers to {key:?} got: {result:?}.");
                    });
                }
            }
            NetworkEvent::NewListenAddr(_) => {
                let network = self.network.clone();
                let peers = self.initial_peers.clone();
                let _handle = spawn(async move {
                    for (peer_id, addr) in &peers {
                        if let Err(err) = network.dial(*peer_id, addr.clone()).await {
                            tracing::error!("Failed to dial {peer_id}: {err:?}");
                        };
                    }
                });
            }
            NetworkEvent::ClosePeerDied(_peer) => {
                let key = NetworkKey::from_peer(self.network.peer_id);

                let closest_peers = match self.network.node_get_closest_local_peers(&key).await {
                    Ok(result) => result,
                    Err(err) => {
                        warn!(
                            "Could not replicate data due to failure to get closest peers: {err}"
                        );
                        return Ok(());
                    }
                };

                let tasks = closest_peers.into_iter().map(|peer| {
                    let network = self.network.clone();
                    let chunks = self.chunks.clone();
                    async move { Self::ask_for_missing_data(peer, network, chunks).await }
                });

                for result in join_all(tasks).await {
                    if let Err(err) = result {
                        warn!("Error occurred in replication process: {err}. We may be temporarily missing data.");
                    }
                }
            }
        }

        Ok(())
    }

    async fn handle_request(&self, request: Request, response_channel: MsgResponder) -> Result<()> {
        trace!("Handling request: {request:?}");
        let response = match request {
            Request::Cmd(cmd) => Response::Cmd(self.handle_cmd(cmd).await),
            Request::Query(query) => Response::Query(self.handle_query(query).await),
            Request::Event(event) => {
                match event {
                    Event::ValidSpendReceived {
                        spend,
                        parent_tx,
                        fee_ciphers,
                        parent_spends,
                    } => {
                        self.transfers
                            .try_add(spend, parent_tx, fee_ciphers, parent_spends)
                            .await
                            .map_err(ProtocolError::Transfers)?;
                        return Ok(());
                    }
                    Event::DoubleSpendAttempted { new, existing } => {
                        self.transfers
                            .try_add_double(new.as_ref(), existing.as_ref())
                            .await
                            .map_err(ProtocolError::Transfers)?;
                        return Ok(());
                    }
                };
            }
        };

        self.send_response(response, response_channel).await;

        Ok(())
    }

    async fn handle_query(&self, query: Query) -> QueryResponse {
        match query {
            Query::Register(query) => self.registers.read(&query, User::Anyone).await,
            Query::GetChunk(address) => {
                let resp = self
                    .chunks
                    .get(&address)
                    .await
                    .map_err(ProtocolError::Storage);
                trace!("Sending response back on query GetChunk({address:?}): {resp:?}");
                QueryResponse::GetChunk(resp)
            }
            Query::Spend(query) => {
                match query {
                    SpendQuery::GetFees { dbc_id, priority } => {
                        // The client is asking for the fee to spend a specific dbc, and including the id of that dbc.
                        // The required fee content is encrypted to that dbc id, and so only the holder of the dbc secret
                        // key can unlock the contents.
                        let required_fee = self.transfers.get_required_fee(dbc_id, priority).await;
                        QueryResponse::GetFees(Ok(required_fee))
                    }
                    SpendQuery::GetDbcSpend(address) => {
                        let res = self
                            .transfers
                            .get(address)
                            .await
                            .map_err(ProtocolError::Transfers);
                        trace!("Sending response back on query DbcSpend {address:?}");
                        QueryResponse::GetDbcSpend(res)
                    }
                }
            }
            Query::GetMissingData {
                existing_data,
                sender,
            } => {
                if let Ok(peers) = self.network.node_get_closest_local_peers(&sender).await {
                    if !peers.contains(&self.network.peer_id) {
                        return QueryResponse::GetMissingData(Err(
                            ProtocolError::InternalProcessing(format!(
                                "{:?} is not among the closest peers to {sender:?}",
                                self.network.peer_id,
                            )),
                        ));
                    }
                };
                // If error in getting closest.. meh ok, let's assume we are one of
                // the closest peers. Should be an exceptional case, and no harm if happens.

                let mut data_i_have = self.chunks.addrs();

                // To make each data storage node reply with different copies, so that the
                // overall queries can be reduced, the data names are scrambled.
                use rand::seq::SliceRandom;
                data_i_have.shuffle(&mut rand::thread_rng());

                let mut data_for_sender = Vec::new();
                for addr in data_i_have {
                    if existing_data.contains(&addr) {
                        continue;
                    }

                    let data = match self.chunks.get(&addr).await {
                        Ok(data) => data,
                        Err(err) => {
                            error!("Failed to get data {addr:?}: {err}");
                            continue;
                        }
                    };

                    data_for_sender.push(data);
                    debug!(
                        "Added {:?} to data batch going to: {sender:?} ",
                        addr.name(),
                    );

                    // To avoid too large amounts of data per msg,
                    // we limit the number of items per query round.
                    // This way, the flow acts like pageing.
                    if data_for_sender.len() == MAX_REPLICATION_BATCH_SIZE {
                        break;
                    }
                }

                QueryResponse::GetMissingData(Ok(data_for_sender))
            }
        }
    }

    async fn handle_cmd(&self, cmd: Cmd) -> CmdResponse {
        match cmd {
            Cmd::StoreChunk(chunk) => {
                let resp = self
                    .chunks
                    .store(&chunk)
                    .await
                    .map_err(ProtocolError::Storage);
                CmdResponse::StoreChunk(resp)
            }
            Cmd::Register(cmd) => {
                let result = self
                    .registers
                    .write(&cmd)
                    .await
                    .map_err(ProtocolError::Storage);

                let xorname = cmd.dst();
                match cmd {
                    RegisterCmd::Create(_) => {
                        self.events_channel
                            .broadcast(NodeEvent::RegisterCreated(xorname));
                        CmdResponse::CreateRegister(result)
                    }
                    RegisterCmd::Edit(_) => {
                        self.events_channel
                            .broadcast(NodeEvent::RegisterEdited(xorname));
                        CmdResponse::EditRegister(result)
                    }
                }
            }
            Cmd::SpendDbc {
                signed_spend,
                parent_tx,
                fee_ciphers,
            } => {
                // First we fetch all parent spends from the network.
                // They shall naturally all exist as valid spends for this current
                // spend attempt to be valid.
                let parent_spends = match self.get_parent_spends(parent_tx.as_ref()).await {
                    Ok(parent_spends) => parent_spends,
                    Err(Error::Protocol(err)) => return CmdResponse::Spend(Err(err)),
                    Err(error) => {
                        return CmdResponse::Spend(Err(ProtocolError::Transfers(
                            TransferError::SpendParentCloseGroupIssue(error.to_string()),
                        )))
                    }
                };

                // Then we try to add the spend to the transfers.
                // This will validate all the necessary components of the spend.
                let res = match self
                    .transfers
                    .try_add(
                        signed_spend.clone(),
                        parent_tx.clone(),
                        fee_ciphers.clone(),
                        parent_spends.clone(),
                    )
                    .await
                {
                    Ok(()) => {
                        let dbc_id = *signed_spend.dbc_id();
                        trace!("Broadcasting valid spend: {dbc_id:?}");

                        self.events_channel
                            .broadcast(NodeEvent::SpendStored(dbc_id));

                        let event = Event::ValidSpendReceived {
                            spend: signed_spend,
                            parent_tx,
                            fee_ciphers,
                            parent_spends,
                        };
                        match self
                            .network
                            .fire_and_forget_to_local_closest(&Request::Event(event))
                            .await
                        {
                            Ok(_) => {}
                            Err(err) => {
                                warn!("Failed to send valid spend event to closest peers: {err:?}");
                            }
                        }

                        Ok(())
                    }
                    Err(TransferError::Storage(StorageError::DoubleSpendAttempt {
                        new,
                        existing,
                    })) => {
                        warn!("Double spend attempted! New: {new:?}. Existing:  {existing:?}");
                        if let Ok(event) =
                            Event::double_spend_attempt(new.clone(), existing.clone())
                        {
                            match self
                                .network
                                .fire_and_forget_to_local_closest(&Request::Event(event))
                                .await
                            {
                                Ok(_) => {}
                                Err(err) => {
                                    warn!("Failed to send double spend event to closest peers: {err:?}");
                                }
                            }
                        }

                        Err(ProtocolError::Transfers(TransferError::Storage(
                            StorageError::DoubleSpendAttempt { new, existing },
                        )))
                    }
                    other => other.map_err(ProtocolError::Transfers),
                };

                CmdResponse::Spend(res)
            }
        }
    }

    #[async_recursion]
    async fn ask_for_missing_data(
        peer: PeerId,
        network: Network,
        chunks: ChunkStorage,
    ) -> Result<()> {
        let existing_data = chunks.addrs().into_iter().collect();
        let request = Request::Query(Query::GetMissingData {
            sender: NetworkKey::from_peer(network.peer_id),
            existing_data,
        });
        let response = network.send_request(request, peer).await?;

        let missing_data = match response {
            Response::Query(QueryResponse::GetMissingData(Ok(missing_data))) => missing_data,
            resp => {
                warn!("Unexpected response: {resp:?}");
                return Err(Error::Protocol(ProtocolError::UnexpectedResponses));
            }
        };

        for chunk in &missing_data {
            match chunks.store(chunk).await {
                Ok(_) => {} // There are logs in the store method, no need to log here.
                Err(err) => {
                    // Don't fail the whole loop, just log the error.
                    warn!("Failed to store chunk: {err:?}");
                }
            }
        }

        // As long as the data batch is not empty, we send back a query again
        // to continue the replication process (like pageing).
        // This means there that there will be a number of repeated `give-me-data -> here_you_go` msg
        // exchanges, until there is no more data missing on this node.
        if !missing_data.is_empty() {
            return Self::ask_for_missing_data(peer, network, chunks).await;
        }

        Ok(())
    }

    // This call makes sure we get the same spend from all in the close group.
    // If we receive a spend here, it is assumed to be valid. But we will verify
    // that anyway, in the code right after this for loop.
    async fn get_parent_spends(&self, parent_tx: &DbcTransaction) -> Result<BTreeSet<SignedSpend>> {
        // These will be different spends, one for each input that went into
        // creating the above spend passed in to this function.
        let mut all_parent_spends = BTreeSet::new();

        if is_genesis_parent_tx(parent_tx) {
            return Ok(all_parent_spends);
        }

        // First we fetch all parent spends from the network.
        // They shall naturally all exist as valid spends for this current
        // spend attempt to be valid.
        for parent_input in &parent_tx.inputs {
            let parent_address = dbc_address(&parent_input.dbc_id());
            // This call makes sure we get the same spend from all in the close group.
            // If we receive a spend here, it is assumed to be valid. But we will verify
            // that anyway, in the code right after this for loop.
            let parent_spend = self.get_spend(parent_address).await?;
            let _ = all_parent_spends.insert(parent_spend);
        }

        Ok(all_parent_spends)
    }

    /// Retrieve a `Spend` from the closest peers
    async fn get_spend(&self, address: DbcAddress) -> Result<SignedSpend> {
        let request = Request::Query(Query::Spend(SpendQuery::GetDbcSpend(address)));
        // NB: for discovery, might need query for the closest instead of local.
        let responses = self.network.node_send_to_queried_closest(&request).await?;

        // Get all Ok results of the expected response type `GetDbcSpend`.
        let spends: Vec<_> = responses
            .iter()
            .flatten()
            .flat_map(|resp| {
                if let Response::Query(QueryResponse::GetDbcSpend(Ok(signed_spend))) = resp {
                    Some(signed_spend.clone())
                } else {
                    None
                }
            })
            .collect();

        // As to not have a single rogue node deliver a bogus spend,
        // and thereby have us fail the check here
        // (we would have more than 1 spend in the BTreeSet), we must
        // look for a majority of the same responses, and ignore any other responses.
        if spends.len() >= close_group_majority() {
            // Majority of nodes in the close group returned an Ok response.
            use itertools::*;
            if let Some(spend) = spends
                .into_iter()
                .map(|x| (x, 1))
                .into_group_map()
                .into_iter()
                .filter(|(_, v)| v.len() >= close_group_majority())
                .max_by_key(|(_, v)| v.len())
                .map(|(k, _)| k)
            {
                // Majority of nodes in the close group returned the same spend.
                return Ok(spend);
            }
        }

        // The parent is not recognised by majority of peers in its close group.
        // Thus, the parent is not valid.
        info!("The spend could not be verified as valid: {address:?}");

        // If not enough spends were gotten, we try error the first
        // error to the expected query returned from nodes.
        for resp in responses.iter().flatten() {
            if let Response::Query(QueryResponse::GetDbcSpend(result)) = resp {
                let _ = result.clone()?;
            };
        }

        // If there were no success or fail to the expected query,
        // we check if there were any send errors.
        for resp in responses {
            let _ = resp?;
        }

        // If there was none of the above, then we had unexpected responses.
        Err(super::Error::Protocol(ProtocolError::UnexpectedResponses))
    }

    async fn send_response(&self, resp: Response, response_channel: MsgResponder) {
        if let Err(err) = self.network.send_response(resp, response_channel).await {
            warn!("Error while sending response: {err:?}");
        }
    }
}
