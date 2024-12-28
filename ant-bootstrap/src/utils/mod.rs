use std::net::IpAddr;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum UtilsError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Could not find non-loopback interface")]
    NoNonLoopbackInterface,
}

/// Returns the first non-loopback IPv4 address found
pub fn find_local_ip() -> Result<IpAddr, UtilsError> {
    let socket = std::net::UdpSocket::bind("0.0.0.0:0")?;
    // This doesn't actually send any packets, just sets up the socket
    socket.connect("8.8.8.8:80")?;
    let addr = socket.local_addr()?;

    if addr.ip().is_loopback() {
        return Err(UtilsError::NoNonLoopbackInterface);
    }

    Ok(addr.ip())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_local_ip() {
        let ip = find_local_ip().expect("Should find a local IP");
        assert!(!ip.is_loopback(), "IP should not be loopback");
        assert!(!ip.is_unspecified(), "IP should not be unspecified");
    }
} 