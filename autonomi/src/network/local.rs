use anyhow::{anyhow, Context, Result};
use ant_networking::find_local_ip;
use libp2p::Multiaddr;
use std::net::{IpAddr, Ipv4Addr, SocketAddr, TcpListener, UdpSocket};
use std::path::PathBuf;
use std::process::Stdio;
use std::time::Duration;
use tokio::process::{Child, Command};
use tokio::time::sleep;

/// Get an available port by letting the OS assign one
fn get_available_port() -> anyhow::Result<u16> {
    // Use find_local_ip to get a suitable IP address
    let ip = find_local_ip()?;
    let socket = UdpSocket::bind((ip, 0))?;
    Ok(socket.local_addr()?.port())
}

pub struct LocalNetwork {
    port: u16,
    child: Child,
}

impl LocalNetwork {
    pub async fn start() -> Result<Self> {
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
            .arg("--first")
            .arg("--ip")
            .arg(ip.to_string())
            .arg("--port")
            .arg(port.to_string())
            .arg("evm-custom")
            .arg("--data-payments-address")
            .arg("0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266")
            .arg("--payment-token-address")
            .arg("0x5FbDB2315678afecb367f032d93F642f64180aa3")
            .arg("--rpc-url")
            .arg("http://localhost:8545");

        let child = cmd
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .context("Failed to start node")?;

        // Give the node some time to start up
        sleep(Duration::from_secs(5)).await;

        Ok(Self { port, child })
    }

    pub fn get_multiaddr(&self, peer_id: &str) -> String {
        let ip = find_local_ip().expect("Should have a valid local IP");
        format!("/ip4/{}/udp/{}/quic-v1/p2p/{}", ip, self.port, peer_id)
    }

    pub async fn start_peer(&self, peer_id: &str) -> Result<Child> {
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
            .arg(port.to_string())
            .arg("--peer")
            .arg(self.get_multiaddr(peer_id))
            .arg("evm-custom")
            .arg("--data-payments-address")
            .arg("0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266")
            .arg("--payment-token-address")
            .arg("0x5FbDB2315678afecb367f032d93F642f64180aa3")
            .arg("--rpc-url")
            .arg("http://localhost:8545");

        let child = cmd
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .context("Failed to start node")?;

        // Give the node some time to start up
        sleep(Duration::from_secs(5)).await;

        Ok(child)
    }
}

impl Drop for LocalNetwork {
    fn drop(&mut self) {
        let _ = self.child.start_kill();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_local_ip() {
        let ip = find_local_ip().expect("Should find a local IP");
        println!("Found local IP: {}", ip);

        // Basic checks
        assert!(!ip.is_loopback(), "IP should not be loopback");
        assert!(!ip.is_unspecified(), "IP should not be unspecified (0.0.0.0)");
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
        let socket = UdpSocket::bind((ip, 0))
            .expect("Should be able to bind to the found IP");
        assert!(socket.local_addr().is_ok(), "Should get local address from socket");

        // Test multiaddr format
        let test_peer_id = "12D3KooWDpJ7As7BWAwRMfu1VU2WCqNjvq387JEYKDBj4kx6nXTN";
        let test_port = 12345;
        let addr = format!("/ip4/{}/udp/{}/quic-v1/p2p/{}", ip, test_port, test_peer_id)
            .parse::<Multiaddr>()
            .expect("Should create valid multiaddr");
        
        assert!(addr.to_string().contains("quic-v1"), "Multiaddr should use QUIC protocol");
    }
}
