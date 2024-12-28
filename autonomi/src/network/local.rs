use ant_networking::find_local_ip;
use anyhow::{Context, Result};
use libp2p::Multiaddr;
use std::net::UdpSocket;
use std::process::Stdio;
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::{Child, Command};
use tokio::time::sleep;

pub struct LocalNode {
    child: Child,
    multiaddr: Multiaddr,
}

impl LocalNode {
    pub async fn start() -> Result<Self> {
        Self::start_internal(true).await
    }

    pub async fn start_secondary() -> Result<Self> {
        Self::start_internal(false).await
    }

    async fn start_internal(is_first: bool) -> Result<Self> {
        let ip = find_local_ip()?;
        let port = {
            // Use UDP socket for QUIC
            let socket = UdpSocket::bind((ip, 0))?;
            socket.local_addr()?.port()
        };

        let mut cmd = Command::new("../target/debug/antnode");
        cmd.arg("--rewards-address")
            .arg("0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266")
            .arg("--home-network")
            .arg("--local")
            .arg("--ip")
            .arg(ip.to_string())
            .arg("--port")
            .arg(port.to_string());

        if is_first {
            cmd.arg("--first");
        }

        cmd.arg("evm-custom")
            .arg("--data-payments-address")
            .arg("0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266")
            .arg("--payment-token-address")
            .arg("0x5FbDB2315678afecb367f032d93F642f64180aa3")
            .arg("--rpc-url")
            .arg("http://localhost:8545");

        // Create pipes for stdout and stderr
        let mut child = cmd
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .context("Failed to start node")?;

        // Set up output capturing
        let stdout = child.stdout.take().expect("Failed to capture stdout");
        let stderr = child.stderr.take().expect("Failed to capture stderr");

        // Spawn tasks to read the output
        let stdout_reader = BufReader::new(stdout).lines();
        let stderr_reader = BufReader::new(stderr).lines();

        tokio::spawn(async move {
            let mut lines = stdout_reader;
            while let Ok(Some(line)) = lines.next_line().await {
                println!("Node stdout: {}", line);
            }
        });

        tokio::spawn(async move {
            let mut lines = stderr_reader;
            while let Ok(Some(line)) = lines.next_line().await {
                println!("Node stderr: {}", line);
            }
        });

        // Give the node some time to start up
        sleep(Duration::from_secs(5)).await;

        let multiaddr = format!("/ip4/{}/udp/{}/quic-v1", ip, port)
            .parse()
            .context("Failed to parse multiaddr")?;

        Ok(Self { child, multiaddr })
    }

    pub fn get_multiaddr(&self) -> Multiaddr {
        self.multiaddr.clone()
    }

    pub async fn is_running(&mut self) -> Result<bool> {
        match self.child.try_wait()? {
            Some(status) => {
                println!("Node exited with status: {}", status);
                Ok(false)
            }
            None => Ok(true),
        }
    }
}

impl Drop for LocalNode {
    fn drop(&mut self) {
        let _ = self.child.start_kill();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::IpAddr;

    #[test]
    fn test_find_local_ip() {
        let ip = find_local_ip().expect("Should find a local IP");
        println!("Found local IP: {}", ip);

        // Basic checks
        assert!(!ip.is_loopback(), "IP should not be loopback");
        assert!(
            !ip.is_unspecified(),
            "IP should not be unspecified (0.0.0.0)"
        );
        assert!(!ip.is_multicast(), "IP should not be multicast");

        // Additional network property checks
        match ip {
            IpAddr::V4(ipv4) => {
                assert!(
                    ipv4.is_private(),
                    "IPv4 address should be in private range (got {})",
                    ipv4
                );

                // Check it's not in special ranges
                assert!(!ipv4.is_broadcast(), "IP should not be broadcast");
                assert!(!ipv4.is_documentation(), "IP should not be documentation");
                assert!(!ipv4.is_link_local(), "IP should not be link local");
            }
            IpAddr::V6(_) => {
                // If we get an IPv6 address, we just ensure it's valid for our use case
                assert!(!ip.is_loopback(), "IPv6 should not be loopback");
                assert!(!ip.is_unspecified(), "IPv6 should not be unspecified");
            }
        }

        // Test socket binding with UDP for QUIC
        let socket = UdpSocket::bind((ip, 0)).expect("Should be able to bind to the found IP");
        assert!(
            socket.local_addr().is_ok(),
            "Should get local address from socket"
        );

        // Test multiaddr format
        let test_peer_id = "12D3KooWDpJ7As7BWAwRMfu1VU2WCqNjvq387JEYKDBj4kx6nXTN";
        let test_port = 12345;
        let addr = format!("/ip4/{}/udp/{}/quic-v1/p2p/{}", ip, test_port, test_peer_id)
            .parse::<Multiaddr>()
            .expect("Should create valid multiaddr");

        assert!(
            addr.to_string().contains("quic-v1"),
            "Multiaddr should use QUIC protocol"
        );
    }
}
