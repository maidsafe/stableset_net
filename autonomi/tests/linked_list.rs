// Copyright 2024 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use anyhow::{Context, Result};
use ant_logging::LogBuilder;
use ant_networking::find_local_ip;
use ant_protocol::storage::LinkedList;
use autonomi::{Client, ClientConfig};
use autonomi::client::linked_list::LinkedListError;
use test_utils::evm::get_funded_wallet;
use bls::SecretKey;
use libp2p::Multiaddr;
use std::net::IpAddr;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::{Child, Command};
use tokio::sync::Mutex;
use tokio::time::{sleep, Duration};
use serial_test::serial;

lazy_static::lazy_static! {
    static ref LOCAL_IP: IpAddr = find_local_ip().expect("Failed to find local IP");
}

#[derive(Debug)]
struct NodeOutput {
    peer_id: Option<String>,
    listeners: Vec<String>,
}

async fn process_output(
    stdout: tokio::process::ChildStdout,
    stderr: tokio::process::ChildStderr,
    _port: u16,
    node_output: Arc<Mutex<NodeOutput>>,
    prefix: String,
) {
    let stdout_reader = BufReader::new(stdout);
    let stderr_reader = BufReader::new(stderr);

    let mut stdout_lines = stdout_reader.lines();
    let mut stderr_lines = stderr_reader.lines();

    loop {
        tokio::select! {
            Ok(Some(line)) = stdout_lines.next_line() => {
                println!("[{}] stdout: {}", prefix, line);
                if line.contains("PeerId is ") {
                    if let Some(peer_id) = line.split("PeerId is ").nth(1) {
                        let mut output = node_output.lock().await;
                        output.peer_id = Some(peer_id.trim().to_string());
                    }
                }
            }
            Ok(Some(line)) = stderr_lines.next_line() => {
                println!("[{}] stderr: {}", prefix, line);
            }
            else => break,
        }
    }
}

async fn start_local_node(port: u16) -> Result<(Child, Arc<Mutex<NodeOutput>>)> {
    println!("Setting up node to listen on {}:{}", *LOCAL_IP, port);

    let mut cmd = Command::new("../target/debug/antnode");
    cmd.arg("--rewards-address")
        .arg("0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266")
        .arg("--home-network")
        .arg("--local")
        .arg("--first")
        .arg("--ip")
        .arg(LOCAL_IP.to_string())
        .arg("--port")
        .arg(port.to_string())
        .arg("--ignore-cache")
        .arg("evm-custom")
        .arg("--data-payments-address")
        .arg("0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266")
        .arg("--payment-token-address")
        .arg("0x5FbDB2315678afecb367f032d93F642f64180aa3")
        .arg("--rpc-url")
        .arg("http://localhost:8545");

    println!("Starting node with command: {:?}", cmd);

    let node_output = Arc::new(Mutex::new(NodeOutput {
        peer_id: None,
        listeners: Vec::new(),
    }));
    let node_output_clone = node_output.clone();

    let mut child = cmd
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .context("Failed to start node")?;

    let stdout = child.stdout.take().unwrap();
    let stderr = child.stderr.take().unwrap();

    tokio::spawn(process_output(
        stdout,
        stderr,
        port,
        node_output_clone,
        format!("Node {}", port),
    ));

    Ok((child, node_output))
}

async fn wait_for_node_ready(node_output: Arc<Mutex<NodeOutput>>) -> Result<String> {
    let mut retries = 0;
    let max_retries = 30;
    let retry_delay = Duration::from_secs(2);

    while retries < max_retries {
        let output = node_output.lock().await;
        if let Some(peer_id) = &output.peer_id {
            println!("Node is ready with peer_id {}", peer_id);
            return Ok(peer_id.clone());
        }
        drop(output);

        println!(
            "Waiting for node to be ready (attempt {}/{})",
            retries + 1,
            max_retries
        );
        sleep(retry_delay).await;
        retries += 1;
    }

    anyhow::bail!("Node failed to start and provide peer ID")
}

#[tokio::test]
#[serial]
async fn test_linked_list() -> Result<()> {
    let _log_appender_guard = LogBuilder::init_single_threaded_tokio_test("linked_list", false);

    // Start a local node
    let port = 50000;
    let (_node, node_output) = start_local_node(port).await?;
    let peer_id = wait_for_node_ready(node_output).await?;

    // Allow time for the node to be fully ready
    sleep(Duration::from_secs(10)).await;

    // Initialize client with local network configuration
    let node_addr = format!("/ip4/{}/udp/{}/quic-v1/p2p/{}", LOCAL_IP.to_string(), port, peer_id);
    let config = ClientConfig {
        local: true,
        peers: Some(vec![node_addr.parse()?]),
    };
    let mut client = Client::init_with_config(config).await?;
    let wallet = get_funded_wallet();
    client.upgrade_to_read_write(wallet.clone())?;

    // Wait for the network to be ready and connected
    sleep(Duration::from_secs(10)).await;

    // Create a new linked list
    let secret_key = SecretKey::random();
    let key = vec![secret_key.public_key()];
    let value = [0u8; 32];

    let linked_list = LinkedList::new(
        SecretKey::random().public_key(),
        key.clone(),
        value,
        None,
        &secret_key,
    );

    // Put the linked list
    client.linked_list_put(linked_list.clone(), &wallet).await?;

    // Wait for replication
    sleep(Duration::from_secs(10)).await;

    // Get the linked list
    let lists = client.linked_list_get(linked_list.address()).await?;
    assert_eq!(lists.len(), 1);
    assert_eq!(&lists[0].parents, &key);
    assert_eq!(&lists[0].content, &value);

    // Try to put a duplicate linked list (should fail)
    let value2 = [1u8; 32];
    let linked_list2 = LinkedList::new(
        SecretKey::random().public_key(),
        key.clone(),
        value2,
        None,
        &secret_key,
    );
    let res = client.linked_list_put(linked_list2.clone(), &wallet).await;
    assert!(matches!(
        res,
        Err(LinkedListError::LinkedListAlreadyExists(_))
    ));

    Ok(())
}

#[tokio::test]
#[serial]
async fn test_linked_list_with_cost() -> Result<()> {
    let _log_appender_guard = LogBuilder::init_single_threaded_tokio_test("linked_list_cost", false);

    // Start a local node
    let port = 50001;
    let (_node, node_output) = start_local_node(port).await?;
    let peer_id = wait_for_node_ready(node_output).await?;

    // Allow time for the node to be fully ready
    sleep(Duration::from_secs(10)).await;

    // Initialize client with local network configuration
    let node_addr = format!("/ip4/{}/udp/{}/quic-v1/p2p/{}", LOCAL_IP.to_string(), port, peer_id);
    let config = ClientConfig {
        local: true,
        peers: Some(vec![node_addr.parse()?]),
    };
    let mut client = Client::init_with_config(config).await?;
    let wallet = get_funded_wallet();
    client.upgrade_to_read_write(wallet.clone())?;

    // Wait for the network to be ready and connected
    sleep(Duration::from_secs(10)).await;

    // Create a new linked list
    let key = SecretKey::random();
    let content = [0u8; 32];
    let linked_list = LinkedList::new(key.public_key(), vec![], content, vec![].into(), &key);

    // Estimate the cost of the linked list
    let cost = client.linked_list_cost(key.clone()).await?;
    println!("linked list cost: {cost}");

    // Put the linked list
    client.linked_list_put(linked_list.clone(), &wallet).await?;
    println!("linked list put 1");

    // Wait for replication
    sleep(Duration::from_secs(10)).await;

    // Check that the linked list is stored
    let lists = client.linked_list_get(linked_list.address()).await?;
    assert_eq!(lists, vec![linked_list.clone()]);
    println!("linked list got 1");

    // Try to put another linked list with the same address
    let content2 = [1u8; 32];
    let linked_list2 = LinkedList::new(key.public_key(), vec![], content2, vec![].into(), &key);
    let res = client.linked_list_put(linked_list2.clone(), &wallet).await;

    assert!(matches!(
        res,
        Err(LinkedListError::LinkedListAlreadyExists(address))
        if address == linked_list2.address()
    ));
    Ok(())
}
