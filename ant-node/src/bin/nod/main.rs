// Copyright 2024 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use ant_bootstrap::{BootstrapCacheStore, PeersArgs};
use ant_evm::{EvmNetwork, EvmWalletError, RewardsAddress};
use ant_node::{NodeBuilder, NodeEvent};
use ant_protocol::{node::get_antnode_root_dir, version};
use color_eyre::{eyre::eyre, Result};
use const_hex::traits::FromHex;
use libp2p::identity::Keypair;
use std::{
    io::Write,
    net::{IpAddr, Ipv4Addr, SocketAddr},
    path::{Path, PathBuf},
};
use tokio::sync::broadcast::error::RecvError;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

#[derive(Debug)]
struct EasyNodeBuilder {
    addr: SocketAddr,

    /// Specify whether the node is operating from a home network and situated behind a NAT without port forwarding
    /// capabilities. Setting this to true, activates hole-punching to facilitate direct connections from other nodes.
    ///
    /// If this not enabled and you're behind a NAT, the node is terminated.
    home_network: bool,

    /// Specify the network ID to use. This will allow you to run the node on a different network.
    ///
    /// By default, the network ID is set to 1, which represents the mainnet.
    network_id: Option<u8>,

    /// Specify the rewards address.
    /// The rewards address is the address that will receive the rewards for the node.
    /// It should be a valid EVM address.
    rewards_address: RewardsAddress,

    /// Specify the EVM network to use.
    /// The network can either be a pre-configured one or a custom network.
    /// When setting a custom network, you must specify the RPC URL to a fully synced node and
    /// the addresses of the network token and chunk payments contracts.
    evm_network: EvmNetwork,

    /// Specify the node's data directory.
    ///
    /// If not provided, the default location is platform specific:
    ///  - Linux: $HOME/.local/share/autonomi/node/<peer-id>
    ///  - macOS: $HOME/Library/Application Support/autonomi/node/<peer-id>
    ///  - Windows: C:\Users\<username>\AppData\Roaming\autonomi\node\<peer-id>
    root_dir: Option<PathBuf>,

    peers: PeersArgs,
}

impl EasyNodeBuilder {
    fn new(rewards_address: RewardsAddress) -> Self {
        Self {
            addr: SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 0),
            home_network: Default::default(),
            network_id: Default::default(),
            rewards_address,
            evm_network: Default::default(),
            root_dir: Default::default(),
            peers: PeersArgs::default(),
        }
    }

    async fn run(&mut self, local: bool) -> Result<()> {
        self.peers.local = local;

        if let Some(network_id) = self.network_id {
            version::set_network_id(network_id);
        }

        let (root_dir, keypair) = get_root_dir_and_keypair(&self.root_dir)?;

        let peers = self.peers.get_addrs(None, Some(100)).await?;
        let mut node_builder = NodeBuilder::new(
            keypair,
            self.rewards_address,
            self.evm_network.clone(),
            self.addr,
            self.peers.local,
            root_dir,
            #[cfg(feature = "upnp")]
            false,
        );
        node_builder.initial_peers(peers);
        node_builder.is_behind_home_network(self.home_network);

        if !local {
            let mut bootstrap_cache = BootstrapCacheStore::new_from_peers_args(&self.peers, None)?;
            // To create the file before startup if it doesn't exist.
            bootstrap_cache.sync_and_flush_to_disk(true)?;
            node_builder.bootstrap_cache(bootstrap_cache);
        }

        run_node(node_builder).await?;

        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(EnvFilter::from_default_env())
        .init();

    color_eyre::install()?;

    let wallet = ant_evm::EvmWallet::new_with_random_wallet(EvmNetwork::default());
    let mut easy_node: EasyNodeBuilder = EasyNodeBuilder::new(wallet.address());
    easy_node.run(true).await?;

    Ok(())
}

async fn run_node(node_builder: NodeBuilder) -> Result<(), ant_node::Error> {
    let running_node = node_builder.build_and_run()?;
    let mut node_events_rx = running_node.node_events_channel().subscribe();

    // let port = running_node.get_node_listening_port().await?;

    loop {
        match node_events_rx.recv().await {
            Ok(NodeEvent::ChannelClosed)
            | Ok(NodeEvent::TerminateNode(_))
            | Err(RecvError::Closed) => {
                return Ok(());
            }
            Ok(event) => {
                tracing::error!("EVENT: {event:?}");
            }
            Err(RecvError::Lagged(n)) => {
                tracing::warn!("Skipped {n} node events!");
                continue;
            }
        }
    }
}

fn create_secret_key_file(path: impl AsRef<Path>) -> Result<std::fs::File, std::io::Error> {
    let mut opt = std::fs::OpenOptions::new();
    opt.write(true).create_new(true);

    // On Unix systems, make sure only the current user can read/write.
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        opt.mode(0o600);
    }

    opt.open(path)
}

fn keypair_from_path(path: impl AsRef<Path>) -> Result<Keypair> {
    let keypair = match std::fs::read(&path) {
        // If the file is opened successfully, read the key from it
        Ok(key) => {
            let keypair = Keypair::ed25519_from_bytes(key)
                .map_err(|err| eyre!("could not read ed25519 key from file: {err}"))?;

            tracing::info!("loaded secret key from file: {:?}", path.as_ref());

            keypair
        }
        // In case the file is not found, generate a new keypair and write it to the file
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
            let secret_key = libp2p::identity::ed25519::SecretKey::generate();
            let mut file = create_secret_key_file(&path)
                .map_err(|err| eyre!("could not create secret key file: {err}"))?;
            file.write_all(secret_key.as_ref())?;

            tracing::info!("generated new key and stored to file: {:?}", path.as_ref());

            libp2p::identity::ed25519::Keypair::from(secret_key).into()
        }
        // Else the file can't be opened, for whatever reason (e.g. permissions).
        Err(err) => {
            return Err(eyre!("failed to read secret key file: {err}"));
        }
    };

    Ok(keypair)
}

/// The keypair is located inside the root directory. At the same time, when no dir is specified,
/// the dir name is derived from the keypair used in the application: the peer ID is used as the directory name.
fn get_root_dir_and_keypair(root_dir: &Option<PathBuf>) -> Result<(PathBuf, Keypair)> {
    match root_dir {
        Some(dir) => {
            std::fs::create_dir_all(dir)?;

            let secret_key_path = dir.join("secret-key");
            Ok((dir.clone(), keypair_from_path(secret_key_path)?))
        }
        None => {
            let secret_key = libp2p::identity::ed25519::SecretKey::generate();
            let keypair: Keypair =
                libp2p::identity::ed25519::Keypair::from(secret_key.clone()).into();
            let peer_id = keypair.public().to_peer_id();

            let dir = get_antnode_root_dir(peer_id)?;
            std::fs::create_dir_all(&dir)?;

            let secret_key_path = dir.join("secret-key");

            let mut file = create_secret_key_file(secret_key_path)
                .map_err(|err| eyre!("could not create secret key file: {err}"))?;
            file.write_all(secret_key.as_ref())?;

            Ok((dir, keypair))
        }
    }
}
