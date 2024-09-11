// Copyright (C) 2024 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use crate::add_services::config::PortRange;
use crate::helpers::{
    check_port_availability, get_bin_version, get_start_port_if_applicable, get_username,
    increment_port_option,
};
use color_eyre::eyre::OptionExt;
use color_eyre::{eyre::eyre, Result};
use colored::Colorize;
use libp2p::{multiaddr::Protocol, Multiaddr, PeerId};
#[cfg(test)]
use mockall::automock;

use sn_evm::get_faucet_data_dir;
use sn_logging::LogFormat;
use sn_service_management::{
    control::ServiceControl,
    rpc::{RpcActions, RpcClient},
    FaucetServiceData, NodeRegistry, NodeServiceData, ServiceStatus,
};
use std::{
    net::{IpAddr, Ipv4Addr, SocketAddr},
    path::PathBuf,
    process::{Command, Stdio},
    str::FromStr,
};
use sysinfo::{Pid, System};

#[cfg_attr(test, automock)]
pub trait Launcher {
    fn get_safenode_path(&self) -> PathBuf;
    fn launch_faucet(&self, genesis_multiaddr: &Multiaddr) -> Result<u32>;
    fn launch_node(
        &self,
        bootstrap_peers: Vec<Multiaddr>,
        log_format: Option<LogFormat>,
        metrics_port: Option<u16>,
        node_port: Option<u16>,
        owner: Option<String>,
        rpc_socket_addr: SocketAddr,
        rewards_address: String,
    ) -> Result<()>;
    fn wait(&self, delay: u64);
}

#[derive(Default)]
pub struct LocalSafeLauncher {
    pub faucet_bin_path: PathBuf,
    pub safenode_bin_path: PathBuf,
}

impl Launcher for LocalSafeLauncher {
    fn get_safenode_path(&self) -> PathBuf {
        self.safenode_bin_path.clone()
    }

    fn launch_faucet(&self, genesis_multiaddr: &Multiaddr) -> Result<u32> {
        info!("Launching the faucet server...");
        let args = vec![
            "--peer".to_string(),
            genesis_multiaddr.to_string(),
            "server".to_string(),
        ];
        let child = Command::new(self.faucet_bin_path.clone())
            .args(args)
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .spawn()?;
        Ok(child.id())
    }

    fn launch_node(
        &self,
        bootstrap_peers: Vec<Multiaddr>,
        log_format: Option<LogFormat>,
        metrics_port: Option<u16>,
        node_port: Option<u16>,
        owner: Option<String>,
        rpc_socket_addr: SocketAddr,
        rewards_address: String,
    ) -> Result<()> {
        let mut args = Vec::new();
        
        args.push("--rewards-address".to_string());
        args.push(rewards_address);

        if let Some(owner) = owner {
            args.push("--owner".to_string());
            args.push(owner);
        }

        if bootstrap_peers.is_empty() {
            args.push("--first".to_string())
        } else {
            for peer in bootstrap_peers {
                args.push("--peer".to_string());
                args.push(peer.to_string());
            }
        }

        if let Some(log_format) = log_format {
            args.push("--log-format".to_string());
            args.push(log_format.as_str().to_string());
        }

        if let Some(metrics_port) = metrics_port {
            args.push("--metrics-server-port".to_string());
            args.push(metrics_port.to_string());
        }

        if let Some(node_port) = node_port {
            args.push("--port".to_string());
            args.push(node_port.to_string());
        }

        args.push("--local".to_string());
        args.push("--rpc".to_string());
        args.push(rpc_socket_addr.to_string());

        Command::new(self.safenode_bin_path.clone())
            .args(args)
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .spawn()
            .inspect_err(|err| error!("Error while spawning node process: {err:?}"))?;

        Ok(())
    }

    /// Provide a delay for the service to start or stop.
    ///
    /// This is wrapped mainly just for unit testing.
    fn wait(&self, delay: u64) {
        std::thread::sleep(std::time::Duration::from_millis(delay));
    }
}

pub fn kill_network(node_registry: &NodeRegistry, keep_directories: bool) -> Result<()> {
    let mut system = System::new_all();
    system.refresh_all();

    // It's possible that the faucet was not spun up because the network failed the validation
    // process. If it wasn't running, we obviously don't need to do anything.
    if let Some(faucet) = &node_registry.faucet {
        // If we're here, the faucet was spun up. However, it's possible for the process to have
        // died since then. In that case, we don't need to do anything.
        // I think the use of `unwrap` is justified here, because for a local network, if the
        // faucet is not `None`, the pid also must have a value.
        if let Some(process) = system.process(Pid::from(faucet.pid.unwrap() as usize)) {
            process.kill();
            debug!("Faucet has been killed");
            println!("{} Killed faucet", "✓".green());
        }
    }

    let faucet_data_path = dirs_next::data_dir()
        .ok_or_else(|| eyre!("Could not obtain user's data directory"))?
        .join("safe")
        .join("test_faucet");
    if faucet_data_path.is_dir() {
        std::fs::remove_dir_all(faucet_data_path)?;
        debug!("Removed faucet data directory");
    }
    let genesis_data_path = dirs_next::data_dir()
        .ok_or_else(|| eyre!("Could not obtain user's data directory"))?
        .join("safe")
        .join("test_genesis");
    if genesis_data_path.is_dir() {
        debug!("Removed genesis data directory");
        std::fs::remove_dir_all(genesis_data_path)?;
    }

    for node in node_registry.nodes.iter() {
        println!("{}:", node.service_name);
        // If the PID is not set it means the `status` command ran and determined the node was
        // already dead anyway, so we don't need to do anything.
        if let Some(pid) = node.pid {
            // It could be possible that None would be returned here, if the process had already
            // died, but the `status` command had not ran. In that case, we don't need to do
            // anything anyway.
            if let Some(process) = system.process(Pid::from(pid as usize)) {
                process.kill();
                debug!("Killed node: {} ({})", node.service_name, pid);
                println!("  {} Killed process", "✓".green());
            }
        }

        if !keep_directories {
            // At this point we don't allow path overrides, so deleting the data directory will clear
            // the log directory also.
            std::fs::remove_dir_all(&node.data_dir_path)?;
            debug!("Removed node data directory: {:?}", node.data_dir_path);
            println!(
                "  {} Removed {}",
                "✓".green(),
                node.data_dir_path.to_string_lossy()
            );
        }
    }

    Ok(())
}

pub struct LocalNetworkOptions {
    pub enable_metrics_server: bool,
    pub faucet_bin_path: PathBuf,
    pub join: bool,
    pub interval: u64,
    pub metrics_port: Option<PortRange>,
    pub node_port: Option<PortRange>,
    pub node_count: u16,
    pub owner: Option<String>,
    pub owner_prefix: Option<String>,
    pub peers: Option<Vec<Multiaddr>>,
    pub rpc_port: Option<PortRange>,
    pub safenode_bin_path: PathBuf,
    pub skip_validation: bool,
    pub log_format: Option<LogFormat>,
    pub rewards_address: String,
}

pub async fn run_network(
    options: LocalNetworkOptions,
    node_registry: &mut NodeRegistry,
    service_control: &dyn ServiceControl,
) -> Result<()> {
    info!("Running local network");

    // Check port availability when joining a local network.
    if let Some(port_range) = &options.node_port {
        port_range.validate(options.node_count)?;
        check_port_availability(port_range, &node_registry.nodes)?;
    }

    if let Some(port_range) = &options.metrics_port {
        port_range.validate(options.node_count)?;
        check_port_availability(port_range, &node_registry.nodes)?;
    }

    if let Some(port_range) = &options.rpc_port {
        port_range.validate(options.node_count)?;
        check_port_availability(port_range, &node_registry.nodes)?;
    }

    let launcher = LocalSafeLauncher {
        safenode_bin_path: options.safenode_bin_path.to_path_buf(),
        faucet_bin_path: options.faucet_bin_path.to_path_buf(),
    };

    let mut node_port = get_start_port_if_applicable(options.node_port);
    let mut metrics_port = get_start_port_if_applicable(options.metrics_port);
    let mut rpc_port = get_start_port_if_applicable(options.rpc_port);

    // Start the bootstrap node if it doesnt exist.
    let (bootstrap_peers, start) = if options.join {
        if let Some(peers) = options.peers {
            (peers, 1)
        } else {
            let peer = node_registry
                .nodes
                .iter()
                .find_map(|n| n.listen_addr.clone())
                .ok_or_else(|| eyre!("Unable to obtain a peer to connect to"))?;
            (peer, 1)
        }
    } else {
        let rpc_free_port = if let Some(port) = rpc_port {
            port
        } else {
            service_control.get_available_port()?
        };
        let metrics_free_port = if let Some(port) = metrics_port {
            Some(port)
        } else if options.enable_metrics_server {
            Some(service_control.get_available_port()?)
        } else {
            None
        };
        let rpc_socket_addr =
            SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), rpc_free_port);
        let rpc_client = RpcClient::from_socket_addr(rpc_socket_addr);

        let number = (node_registry.nodes.len() as u16) + 1;
        let owner = get_node_owner(&options.owner_prefix, &options.owner, &number);
        let node = run_node(
            RunNodeOptions {
                bootstrap_peers: vec![],
                genesis: true,
                metrics_port: metrics_free_port,
                node_port,
                interval: options.interval,
                log_format: options.log_format,
                number,
                owner,
                rpc_socket_addr,
                version: get_bin_version(&launcher.get_safenode_path())?,
                rewards_address: options.rewards_address.clone(),
            },
            &launcher,
            &rpc_client,
        )
        .await?;
        node_registry.nodes.push(node.clone());
        let bootstrap_peers = node
            .listen_addr
            .ok_or_eyre("The listen address was not set")?;
        node_port = increment_port_option(node_port);
        metrics_port = increment_port_option(metrics_port);
        rpc_port = increment_port_option(rpc_port);
        (bootstrap_peers, 2)
    };
    node_registry.save()?;

    for _ in start..=options.node_count {
        let rpc_free_port = if let Some(port) = rpc_port {
            port
        } else {
            service_control.get_available_port()?
        };
        let metrics_free_port = if let Some(port) = metrics_port {
            Some(port)
        } else if options.enable_metrics_server {
            Some(service_control.get_available_port()?)
        } else {
            None
        };
        let rpc_socket_addr =
            SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), rpc_free_port);
        let rpc_client = RpcClient::from_socket_addr(rpc_socket_addr);

        let number = (node_registry.nodes.len() as u16) + 1;
        let owner = get_node_owner(&options.owner_prefix, &options.owner, &number);
        let node = run_node(
            RunNodeOptions {
                bootstrap_peers: bootstrap_peers.clone(),
                genesis: false,
                metrics_port: metrics_free_port,
                node_port,
                interval: options.interval,
                log_format: options.log_format,
                number,
                owner,
                rpc_socket_addr,
                version: get_bin_version(&launcher.get_safenode_path())?,
                rewards_address: options.rewards_address.clone(),
            },
            &launcher,
            &rpc_client,
        )
        .await?;
        node_registry.nodes.push(node);

        // We save the node registry for each launch because it's possible any node can fail to
        // launch, or maybe the validation will fail. In the error case, we will want to use the
        // `kill` command for the nodes that we did spin up. The `kill` command works on the basis
        // of what's in the node registry.
        node_registry.save()?;

        node_port = increment_port_option(node_port);
        metrics_port = increment_port_option(metrics_port);
        rpc_port = increment_port_option(rpc_port);
    }

    if !options.skip_validation {
        debug!("Waiting for 10 seconds before validating the network...");
        println!("Waiting for 10 seconds before validating the network...");
        std::thread::sleep(std::time::Duration::from_secs(10));
        validate_network(node_registry, bootstrap_peers.clone()).await?;
    }

    if !options.join {
        println!("Launching the faucet server...");
        let pid = launcher.launch_faucet(&bootstrap_peers[0])?;
        let version = get_bin_version(&options.faucet_bin_path)?;
        let faucet = FaucetServiceData {
            faucet_path: options.faucet_bin_path,
            local: true,
            log_dir_path: get_faucet_data_dir(),
            pid: Some(pid),
            service_name: "faucet".to_string(),
            status: ServiceStatus::Running,
            user: get_username()?,
            version,
        };
        node_registry.faucet = Some(faucet);
    }

    Ok(())
}

pub struct RunNodeOptions {
    pub bootstrap_peers: Vec<Multiaddr>,
    pub genesis: bool,
    pub interval: u64,
    pub log_format: Option<LogFormat>,
    pub metrics_port: Option<u16>,
    pub node_port: Option<u16>,
    pub number: u16,
    pub owner: Option<String>,
    pub rpc_socket_addr: SocketAddr,
    pub version: String,
    pub rewards_address: String,
}

pub async fn run_node(
    run_options: RunNodeOptions,
    launcher: &dyn Launcher,
    rpc_client: &dyn RpcActions,
) -> Result<NodeServiceData> {
    info!("Launching node {}...", run_options.number);
    println!("Launching node {}...", run_options.number);
    launcher.launch_node(
        run_options.bootstrap_peers.clone(),
        run_options.log_format,
        run_options.metrics_port,
        run_options.node_port,
        run_options.owner.clone(),
        run_options.rpc_socket_addr,
        run_options.rewards_address.clone(),
    )?;
    launcher.wait(run_options.interval);

    let node_info = rpc_client.node_info().await?;
    let peer_id = node_info.peer_id;
    let network_info = rpc_client.network_info().await?;
    let connected_peers = Some(network_info.connected_peers);
    let listen_addrs = network_info
        .listeners
        .into_iter()
        .map(|addr| addr.with(Protocol::P2p(node_info.peer_id)))
        .collect();

    Ok(NodeServiceData {
        auto_restart: false,
        connected_peers,
        data_dir_path: node_info.data_path,
        genesis: run_options.genesis,
        home_network: false,
        listen_addr: Some(listen_addrs),
        local: true,
        log_dir_path: node_info.log_path,
        log_format: run_options.log_format,
        metrics_port: run_options.metrics_port,
        node_port: run_options.node_port,
        number: run_options.number,
        owner: run_options.owner,
        peer_id: Some(peer_id),
        pid: Some(node_info.pid),
        reward_balance: None,
        rpc_socket_addr: run_options.rpc_socket_addr,
        safenode_path: launcher.get_safenode_path(),
        status: ServiceStatus::Running,
        service_name: format!("safenode-local{}", run_options.number),
        upnp: false,
        user: None,
        user_mode: false,
        version: run_options.version.to_string(),
    })
}

///
/// Private Helpers
///

async fn validate_network(node_registry: &mut NodeRegistry, peers: Vec<Multiaddr>) -> Result<()> {
    let mut all_peers = node_registry
        .nodes
        .iter()
        .map(|n| n.peer_id.ok_or_eyre("The PeerId was not set"))
        .collect::<Result<Vec<PeerId>>>()?;
    // The additional peers are peers being managed outwith the node manager. This only applies
    // when we've joined a network not being managed by the node manager. Otherwise, this list will
    // be empty.
    let additional_peers = peers
        .into_iter()
        .filter_map(|addr| {
            addr.to_string()
                .rsplit('/')
                .next()
                .and_then(|id_str| PeerId::from_str(id_str).ok())
        })
        .collect::<Vec<PeerId>>();
    all_peers.extend(additional_peers);

    for node in node_registry.nodes.iter() {
        let rpc_client = RpcClient::from_socket_addr(node.rpc_socket_addr);
        let net_info = rpc_client.network_info().await?;
        let peers = net_info.connected_peers;
        let peer_id = node.peer_id.ok_or_eyre("The PeerId was not set")?;
        debug!("Node {peer_id} has {} peers", peers.len());
        println!("Node {peer_id} has {} peers", peers.len());

        // Look for peers that are not supposed to be present in the network. This can happen if
        // the node has connected to peers on other networks.
        let invalid_peers: Vec<PeerId> = peers
            .iter()
            .filter(|peer| !all_peers.contains(peer))
            .cloned()
            .collect();
        if !invalid_peers.is_empty() {
            for invalid_peer in invalid_peers.iter() {
                println!("Invalid peer found: {}", invalid_peer);
            }
            error!("Network validation failed: {invalid_peers:?}");
            return Err(eyre!("Network validation failed",));
        }
    }
    Ok(())
}

fn get_node_owner(
    owner_prefix: &Option<String>,
    owner: &Option<String>,
    number: &u16,
) -> Option<String> {
    if let Some(prefix) = owner_prefix {
        Some(format!("{}_{}", prefix, number))
    } else {
        owner.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use libp2p_identity::PeerId;
    use mockall::mock;
    use mockall::predicate::*;
    use sn_service_management::{
        error::Result as RpcResult,
        rpc::{NetworkInfo, NodeInfo, RecordAddress, RpcActions},
    };
    use std::str::FromStr;

    mock! {
        pub RpcClient {}
        #[async_trait]
        impl RpcActions for RpcClient {
            async fn node_info(&self) -> RpcResult<NodeInfo>;
            async fn network_info(&self) -> RpcResult<NetworkInfo>;
            async fn record_addresses(&self) -> RpcResult<Vec<RecordAddress>>;
            async fn node_restart(&self, delay_millis: u64, retain_peer_id: bool) -> RpcResult<()>;
            async fn node_stop(&self, delay_millis: u64) -> RpcResult<()>;
            async fn node_update(&self, delay_millis: u64) -> RpcResult<()>;
            async fn is_node_connected_to_network(&self, timeout: std::time::Duration) -> RpcResult<()>;
            async fn update_log_level(&self, log_levels: String) -> RpcResult<()>;
        }
    }

    #[tokio::test]
    async fn run_node_should_launch_the_genesis_node() -> Result<()> {
        let mut mock_launcher = MockLauncher::new();
        let mut mock_rpc_client = MockRpcClient::new();

        let peer_id = PeerId::from_str("12D3KooWS2tpXGGTmg2AHFiDh57yPQnat49YHnyqoggzXZWpqkCR")?;
        let rpc_socket_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 13000);
        mock_launcher
            .expect_launch_node()
            .with(
                eq(vec![]),
                eq(None),
                eq(None),
                eq(None),
                eq(None),
                eq(rpc_socket_addr),
                eq("0x24D1E7a7EC1aDd67F0D4B2745971dd0D31F648b4".to_string()),
            )
            .times(1)
            .returning(|_, _, _, _, _, _, _| Ok(()));
        mock_launcher
            .expect_wait()
            .with(eq(100))
            .times(1)
            .returning(|_| ());
        mock_launcher
            .expect_get_safenode_path()
            .times(1)
            .returning(|| PathBuf::from("/usr/local/bin/safenode"));

        mock_rpc_client
            .expect_node_info()
            .times(1)
            .returning(move || {
                Ok(NodeInfo {
                    pid: 1000,
                    peer_id,
                    data_path: PathBuf::from(format!("~/.local/share/safe/{peer_id}")),
                    log_path: PathBuf::from(format!("~/.local/share/safe/{peer_id}/logs")),
                    version: "0.100.12".to_string(),
                    uptime: std::time::Duration::from_secs(1), // the service was just started
                    wallet_balance: 0,
                })
            });
        mock_rpc_client
            .expect_network_info()
            .times(1)
            .returning(move || {
                Ok(NetworkInfo {
                    connected_peers: Vec::new(),
                    listeners: Vec::new(),
                })
            });

        let node = run_node(
            RunNodeOptions {
                bootstrap_peers: vec![],
                genesis: true,
                interval: 100,
                log_format: None,
                metrics_port: None,
                node_port: None,
                number: 1,
                owner: None,
                rpc_socket_addr,
                version: "0.100.12".to_string(),
                rewards_address: "0x24D1E7a7EC1aDd67F0D4B2745971dd0D31F648b4".to_string(),
            },
            &mock_launcher,
            &mock_rpc_client,
        )
        .await?;

        assert!(node.genesis);
        assert_eq!(node.version, "0.100.12");
        assert_eq!(node.service_name, "safenode-local1");
        assert_eq!(
            node.data_dir_path,
            PathBuf::from(format!("~/.local/share/safe/{peer_id}"))
        );
        assert_eq!(
            node.log_dir_path,
            PathBuf::from(format!("~/.local/share/safe/{peer_id}/logs"))
        );
        assert_eq!(node.number, 1);
        assert_eq!(node.pid, Some(1000));
        assert_eq!(node.rpc_socket_addr, rpc_socket_addr);
        assert_eq!(node.status, ServiceStatus::Running);
        assert_eq!(node.safenode_path, PathBuf::from("/usr/local/bin/safenode"));

        Ok(())
    }
}
