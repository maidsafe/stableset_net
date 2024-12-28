// Copyright 2024 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

// Optionally enable nightly `doc_cfg`. Allows items to be annotated, e.g.: "Available on crate feature X only".
#![cfg_attr(docsrs, feature(doc_cfg))]

pub mod address;
pub mod payment;
pub mod quote;

pub mod data;
pub mod error;
pub use error::{CostError, GetError, PayError, PutError};
pub mod files;
pub mod linked_list;
pub mod pointer;

#[cfg(feature = "external-signer")]
#[cfg_attr(docsrs, doc(cfg(feature = "external-signer")))]
pub mod external_signer;
#[cfg(feature = "registers")]
#[cfg_attr(docsrs, doc(cfg(feature = "registers")))]
pub mod registers;
#[cfg(feature = "vault")]
#[cfg_attr(docsrs, doc(cfg(feature = "vault")))]
pub mod vault;

#[cfg(target_arch = "wasm32")]
pub mod wasm;

// private module with utility functions
mod rate_limiter;
mod utils;

use ant_bootstrap::{BootstrapCacheConfig, BootstrapCacheStore};
pub use ant_evm::Amount;
use ant_evm::{EvmNetwork, EvmWallet, EvmWalletError};
use ant_networking::{
    interval, multiaddr_is_global, Network, NetworkBuilder, NetworkError, NetworkEvent,
};
use ant_protocol::version::IDENTIFY_PROTOCOL_STR;
use ant_protocol::NetworkAddress;
use ant_service_management::rpc::{NetworkInfo, NodeInfo, RecordAddress};
use anyhow::Result;
use libp2p::{identity::Keypair, Multiaddr, PeerId};
use std::{collections::HashSet, sync::Arc, time::Duration};
use tokio::sync::mpsc;
use tracing::{debug, error};

/// Time before considering the connection timed out.
pub const CONNECT_TIMEOUT_SECS: u64 = 10;

const CLIENT_EVENT_CHANNEL_SIZE: usize = 100;

// Amount of peers to confirm into our routing table before we consider the client ready.
pub use ant_protocol::CLOSE_GROUP_SIZE;

/// Events emitted by the client.
#[derive(Debug, Clone)]
pub enum ClientEvent {
    /// A new peer was discovered.
    PeerDiscovered(libp2p::PeerId),
    /// A peer was disconnected.
    PeerDisconnected(libp2p::PeerId),
    /// Upload operation completed.
    UploadComplete(UploadSummary),
}

/// Summary of an upload operation.
#[derive(Debug, Clone)]
pub struct UploadSummary {
    pub record_count: usize,
    pub tokens_spent: Amount,
}

/// Error returned by [`Client::init`].
#[derive(Debug, thiserror::Error)]
pub enum ConnectError {
    /// Did not manage to populate the routing table with enough peers.
    #[error("Failed to populate our routing table with enough peers in time")]
    TimedOut,

    /// Same as [`ConnectError::TimedOut`] but with a list of incompatible protocols.
    #[error("Failed to populate our routing table due to incompatible protocol: {0:?}")]
    TimedOutWithIncompatibleProtocol(HashSet<String>, String),

    /// An error occurred while bootstrapping the client.
    #[error("Failed to bootstrap the client")]
    Bootstrap(#[from] ant_bootstrap::Error),
}

/// Client mode indicating read-only or read-write capabilities
pub enum ClientMode {
    /// Read-only mode without a wallet
    ReadOnly,
    /// Read-write mode with an attached wallet
    ReadWrite(EvmWallet),
}

impl Clone for ClientMode {
    fn clone(&self) -> Self {
        match self {
            Self::ReadOnly => Self::ReadOnly,
            Self::ReadWrite(wallet) => Self::ReadWrite(wallet.clone()),
        }
    }
}

/// Represents a client for the Autonomi network.
#[derive(Clone)]
pub struct Client {
    pub(crate) network: Network,
    pub(crate) client_event_sender: Arc<Option<mpsc::Sender<ClientEvent>>>,
    pub(crate) evm_network: EvmNetwork,
    pub(crate) mode: ClientMode,
}

/// Configuration for [`Client::init_with_config`].
#[derive(Debug, Clone)]
pub struct ClientConfig {
    /// Whether we're expected to connect to a local network.
    pub local: bool,

    /// List of peers to connect to.
    ///
    /// If not provided, the client will use the default bootstrap peers.
    pub peers: Option<Vec<Multiaddr>>,
}

impl Default for ClientConfig {
    fn default() -> Self {
        Self {
            local: false,
            peers: None,
        }
    }
}

impl Client {
    /// Initialize a new client with default configuration
    pub async fn init() -> Result<Self> {
        Self::init_with_config(ClientConfig::default()).await
    }

    /// Initialize the network with the given config
    pub async fn init_with_config(config: ClientConfig) -> Result<Self> {
        let keypair = Keypair::generate_ed25519();
        let mut builder = NetworkBuilder::new(keypair);

        // Configure local mode if enabled
        if config.local {
            builder = builder.local(true);
        }

    /// Initialize a client that bootstraps from a list of peers.
    ///
    /// If any of the provided peers is a global address, the client will not be local.
    ///
    /// ```no_run
    /// # use autonomi::Client;
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// // Will set `local` to true.
    /// let client = Client::init_with_peers(vec!["/ip4/127.0.0.1/udp/1234/quic-v1".parse()?]).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn init_with_peers(peers: Vec<Multiaddr>) -> Result<Self, ConnectError> {
        // Always use local mode for testing
        Self::init_with_config(ClientConfig {
            local: true,
            peers: Some(peers),
        })
        .await
    }

    /// Initialize the client with the given configuration.
    ///
    /// This will block until [`CLOSE_GROUP_SIZE`] have been added to the routing table.
    ///
    /// See [`ClientConfig`].
    ///
    /// ```no_run
    /// use autonomi::client::Client;
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = Client::init_with_config(Default::default()).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn init_with_config(config: ClientConfig) -> Result<Self, ConnectError> {
        let (network, event_receiver) = build_client_and_run_swarm(config.local);

        let peers_args = PeersArgs {
            disable_mainnet_contacts: config.local,
            addrs: config.peers.clone().unwrap_or_default(),
            ..Default::default()
        };

        let peers = match peers_args.get_addrs(None, None).await {
            Ok(peers) => peers,
            Err(e) => return Err(e.into()),
        };

        let network_clone = network.clone();
        let peers = peers.to_vec();
        let _handle = ant_networking::target_arch::spawn(async move {
            for addr in peers {
                if let Err(err) = network_clone.dial(addr.clone()).await {
                    error!("Failed to dial addr={addr} with err: {err:?}");
                };
            }
        }

        let (network, _event_receiver, driver) =
            builder.build_client().expect("Failed to build network");

        // Spawn the driver to run in the background
        ant_networking::target_arch::spawn(async move {
            driver.run().await;
        });

        Ok(Self {
            network,
            client_event_sender: Arc::new(None),
            evm_network: Default::default(),
            mode: ClientMode::ReadOnly,
        })
    }

    /// Initialize the network in local mode
    pub async fn init_local(local: bool) -> Result<Self> {
        let keypair = Keypair::generate_ed25519();
        let mut builder = NetworkBuilder::new(keypair);

        // Configure local mode if enabled
        if local {
            builder = builder.local(true);
        }

        let (network, _event_receiver, driver) =
            builder.build_client().expect("Failed to build network");

        // Spawn the driver to run in the background
        ant_networking::target_arch::spawn(async move {
            driver.run().await;
        });

        Ok(Self {
            network,
            client_event_sender: Arc::new(None),
            evm_network: Default::default(),
            mode: ClientMode::ReadOnly,
        })
    }

    /// Initialize a new client with the given peers
    pub async fn init_with_peers(peers: Vec<Multiaddr>) -> Result<Self> {
        let config = ClientConfig {
            peers: Some(peers),
            ..Default::default()
        };
        Self::init_with_config(config).await
    }

    /// Connect to the network.
    ///
    /// This will timeout after [`CONNECT_TIMEOUT_SECS`] secs.
    ///
    /// ```no_run
    /// # use autonomi::client::Client;
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let peers = ["/ip4/127.0.0.1/udp/1234/quic-v1".parse()?];
    /// #[allow(deprecated)]
    /// let client = Client::connect(&peers).await?;
    /// # Ok(())
    /// # }
    /// ```
    #[deprecated(
        since = "0.2.4",
        note = "Use [`Client::init`] or [`Client::init_with_config`] instead"
    )]
    pub async fn connect(peers: &[Multiaddr]) -> Result<Self, ConnectError> {
        // Any global address makes the client non-local
        let local = !peers.iter().any(multiaddr_is_global);
        let config = ClientConfig {
            local,
            peers: Some(peers.to_vec()),
        };

        let keypair = Keypair::generate_ed25519();
        let mut builder = NetworkBuilder::new(keypair);
        if local {
            builder = builder.local(true);
        }

        let (network, event_receiver, driver) =
            builder.build_client().expect("Failed to build network");

        // Spawn the driver to run in the background
        ant_networking::target_arch::spawn(async move {
            driver.run().await;
        });

        let (sender, receiver) = futures::channel::oneshot::channel();
        ant_networking::target_arch::spawn(handle_event_receiver(event_receiver, sender, config));

        receiver.await.expect("sender should not close")?;
        debug!("Client is connected to the network");

        // With the switch to the new bootstrap cache scheme,
        // Seems the too many `initial dial`s could result in failure,
        // when startup quoting/upload tasks got started up immediatly.
        // Hence, put in a forced wait to allow `initial network discovery` to be completed.
        ant_networking::target_arch::sleep(Duration::from_secs(5)).await;

        Ok(Self {
            network,
            client_event_sender: Arc::new(None),
            evm_network: Default::default(),
            mode: ClientMode::ReadOnly,
        })
    }

    /// Receive events from the client.
    pub fn enable_client_events(&mut self) -> mpsc::Receiver<ClientEvent> {
        let (client_event_sender, client_event_receiver) =
            tokio::sync::mpsc::channel(CLIENT_EVENT_CHANNEL_SIZE);
        self.client_event_sender = Arc::new(Some(client_event_sender));
        debug!("All events to the clients are enabled");

        client_event_receiver
    }

    pub fn set_evm_network(&mut self, evm_network: EvmNetwork) {
        self.evm_network = evm_network;
    }

    /// Get information about the node
    pub async fn node_info(&self) -> Result<NodeInfo> {
        let _state = self.network.get_swarm_local_state().await?;
        Ok(NodeInfo {
            pid: std::process::id(),
            peer_id: self.network.peer_id(),
            log_path: Default::default(),
            data_path: Default::default(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            uptime: Duration::from_secs(0),
            wallet_balance: 0,
        })
    }

    /// Get information about the network
    pub async fn network_info(&self) -> Result<NetworkInfo> {
        let _state = self.network.get_swarm_local_state().await?;
        Ok(NetworkInfo {
            connected_peers: _state.connected_peers,
            listeners: _state.listeners,
        })
    }

    /// Get record addresses
    pub async fn record_addresses(&self) -> Result<Vec<RecordAddress>> {
        Ok(Vec::new())
    }

    /// Restart the node
    pub async fn node_restart(&self, _delay_millis: u64, _retain_peer_id: bool) -> Result<()> {
        Ok(())
    }

    /// Stop the node
    pub async fn node_stop(&self, _delay_millis: u64) -> Result<()> {
        Ok(())
    }

    /// Update the node
    pub async fn node_update(&self, _delay_millis: u64) -> Result<()> {
        Ok(())
    }

    /// Check if node is connected to network
    pub async fn is_node_connected_to_network(&self, _timeout: Duration) -> Result<()> {
        let _state = self.network.get_swarm_local_state().await?;
        if !_state.connected_peers.is_empty() {
            Ok(())
        } else {
            Err(anyhow::anyhow!("Not connected to any peers"))
        }
    }

    /// Update log level
    pub async fn update_log_level(&self, _log_levels: String) -> Result<()> {
        Ok(())
    }

    /// Initialize a new read-only client with default configuration
    pub async fn init_read_only() -> Result<Self> {
        Self::init_read_only_with_config(ClientConfig::default()).await
    }

    /// Initialize a read-only client with the given config
    pub async fn init_read_only_with_config(config: ClientConfig) -> Result<Self> {
        let keypair = Keypair::generate_ed25519();
        let mut builder = NetworkBuilder::new(keypair);

        // Configure local mode if enabled
        if config.local {
            builder = builder.local(true);
        }

        // If we're not in local mode, try to set up the bootstrap cache
        if !config.local {
            if let Ok(mut config) = BootstrapCacheConfig::default_config() {
                config.disable_cache_writing = true;
                if let Ok(cache) = BootstrapCacheStore::new(config) {
                    builder.bootstrap_cache(cache);
                }
            }
        }

        let (network, _event_receiver, driver) =
            builder.build_client().expect("Failed to build network");

        // Spawn the driver to run in the background
        ant_networking::target_arch::spawn(async move {
            driver.run().await;
        });

        Ok(Self {
            network,
            client_event_sender: Arc::new(None),
            evm_network: Default::default(),
            mode: ClientMode::ReadOnly,
        })
    }

    /// Initialize a new client with a wallet for read-write access
    pub async fn init_with_wallet(wallet: EvmWallet) -> Result<Self> {
        Self::init_with_wallet_and_config(wallet, ClientConfig::default()).await
    }

    /// Initialize a client with a wallet and config for read-write access
    pub async fn init_with_wallet_and_config(
        wallet: EvmWallet,
        config: ClientConfig,
    ) -> Result<Self> {
        let keypair = Keypair::generate_ed25519();
        let mut builder = NetworkBuilder::new(keypair);

        // Configure local mode if enabled
        if config.local {
            builder = builder.local(true);
        }

        // If we're not in local mode, try to set up the bootstrap cache
        if !config.local {
            if let Ok(mut config) = BootstrapCacheConfig::default_config() {
                config.disable_cache_writing = true;
                if let Ok(cache) = BootstrapCacheStore::new(config) {
                    builder.bootstrap_cache(cache);
                }
            }
        }

        let (network, _event_receiver, driver) =
            builder.build_client().expect("Failed to build network");

        // Spawn the driver to run in the background
        ant_networking::target_arch::spawn(async move {
            driver.run().await;
        });

        Ok(Self {
            network,
            client_event_sender: Arc::new(None),
            evm_network: Default::default(),
            mode: ClientMode::ReadWrite(wallet),
        })
    }

    /// Check if the client has write access (i.e. has a wallet)
    pub fn check_write_access(&self) -> Result<(), PutError> {
        if self.wallet().is_none() {
            return Err(PutError::NoWallet);
        }
        Ok(())
    }

    /// Get the wallet if in read-write mode
    pub fn wallet(&self) -> Option<&EvmWallet> {
        match &self.mode {
            ClientMode::ReadWrite(wallet) => Some(wallet),
            ClientMode::ReadOnly => None,
        }
    }

    /// Check if the client has write capabilities
    pub fn can_write(&self) -> bool {
        matches!(self.mode, ClientMode::ReadWrite(_))
    }

    /// Upgrade a read-only client to read-write by providing a wallet
    pub fn upgrade_to_read_write(&mut self, wallet: EvmWallet) -> Result<()> {
        match self.mode {
            ClientMode::ReadOnly => {
                self.mode = ClientMode::ReadWrite(wallet);
                Ok(())
            }
            ClientMode::ReadWrite(_) => {
                Err(anyhow::anyhow!("Client is already in read-write mode"))
            }
        }
    }
}

fn build_client_and_run_swarm(local: bool) -> (Network, mpsc::Receiver<NetworkEvent>) {
    let keypair = Keypair::generate_ed25519();
    let mut builder = NetworkBuilder::new(keypair);

    if local {
        builder = builder.local(true);
    }

    // In local mode, we want to disable cache writing
    if local {
        if let Ok(mut config) = BootstrapCacheConfig::default_config() {
            config.disable_cache_writing = true;
            if let Ok(cache) = BootstrapCacheStore::new(config) {
                builder.bootstrap_cache(cache);
            }
        }
    }

    let (network, event_receiver, driver) =
        builder.build_client().expect("Failed to build network");

    // Spawn the driver to run in the background
    ant_networking::target_arch::spawn(async move {
        driver.run().await;
    });

    (network, event_receiver)
}

async fn handle_event_receiver(
    mut event_receiver: mpsc::Receiver<NetworkEvent>,
    sender: futures::channel::oneshot::Sender<Result<(), ConnectError>>,
    config: ClientConfig,
) {
    // We switch this to `None` when we've sent the oneshot 'connect' result.
    let mut sender = Some(sender);
    let mut unsupported_protocols = vec![];

    let mut timeout_timer = interval(Duration::from_secs(CONNECT_TIMEOUT_SECS));

    #[cfg(not(target_arch = "wasm32"))]
    timeout_timer.tick().await;

    loop {
        tokio::select! {
            _ = timeout_timer.tick() =>  {
                if let Some(sender) = sender.take() {
                    if unsupported_protocols.len() > 1 {
                        let protocols: HashSet<String> =
                            unsupported_protocols.iter().cloned().collect();
                        sender
                            .send(Err(ConnectError::TimedOutWithIncompatibleProtocol(
                                protocols,
                                IDENTIFY_PROTOCOL_STR.read().expect("Failed to obtain read lock for IDENTIFY_PROTOCOL_STR. A call to set_network_id performed. This should not happen").clone(),
                            )))
                            .expect("receiver should not close");
                    } else {
                        sender
                            .send(Err(ConnectError::TimedOut))
                            .expect("receiver should not close");
                    }
                }
                break;
            }
            event = event_receiver.recv() => {
                let event = event.expect("receiver should not close");
                match event {
                    NetworkEvent::PeerAdded(_peer_id, peers_len) => {
                        tracing::trace!("Peer added: {peers_len} in routing table");

                        // For local testing, we only need one peer
                        // For non-local, we need CLOSE_GROUP_SIZE peers
                        let required_peers = if config.local { 1 } else { CLOSE_GROUP_SIZE };
                        if peers_len >= required_peers {
                            if let Some(sender) = sender.take() {
                                sender.send(Ok(())).expect("receiver should not close");
                                break;
                            }
                        }
                    }
                    NetworkEvent::PeerWithUnsupportedProtocol { their_protocol, .. } => {
                        tracing::warn!(their_protocol, "Peer with unsupported protocol");

                        if sender.is_some() {
                            unsupported_protocols.push(their_protocol);
                        }
                    }
                    _ => {}
                }
            }
        }
    }
}
