// Copyright 2024 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use ant_bootstrap::{BootstrapCacheConfig, BootstrapCacheStore};
use ant_logging::LogBuilder;
use libp2p::Multiaddr;
use std::net::{IpAddr, Ipv4Addr};
use std::time::Duration;
use tempfile::TempDir;
use tokio::time::sleep;

// Use a private network IP instead of loopback for mDNS to work
const LOCAL_IP: IpAddr = IpAddr::V4(Ipv4Addr::new(192, 168, 1, 23));

#[tokio::test]
async fn test_cache_store_basic() -> Result<(), Box<dyn std::error::Error>> {
    let _guard = LogBuilder::init_single_threaded_tokio_test("cache_tests", false);

    let temp_dir = TempDir::new()?;
    let cache_path = temp_dir.path().join("cache.json");

    let config = BootstrapCacheConfig::empty().with_cache_path(&cache_path);
    let mut cache_store = BootstrapCacheStore::new(config)?;

    let addr: Multiaddr = format!(
        "/ip4/{}/udp/8080/quic-v1/p2p/12D3KooWRBhwfeP2Y4TCx1SM6s9rUoHhR5STiGwxBhgFRcw3UERE",
        LOCAL_IP
    )
    .parse()?;
    cache_store.add_addr(addr.clone());
    cache_store.update_addr_status(&addr, true);

    let addrs = cache_store.get_sorted_addrs().collect::<Vec<_>>();
    assert!(!addrs.is_empty(), "Cache should contain the added peer");
    assert!(
        addrs.iter().any(|&a| a == &addr),
        "Cache should contain our specific peer"
    );

    Ok(())
}

#[tokio::test]
async fn test_cache_max_peers() -> Result<(), Box<dyn std::error::Error>> {
    let _guard = LogBuilder::init_single_threaded_tokio_test("cache_tests", false);

    let temp_dir = TempDir::new()?;
    let cache_path = temp_dir.path().join("cache.json");

    let mut config = BootstrapCacheConfig::empty().with_cache_path(&cache_path);
    config.max_peers = 2;

    let mut cache_store = BootstrapCacheStore::new(config)?;

    for i in 1..=3 {
        let addr: Multiaddr = format!(
            "/ip4/{}/udp/808{}/quic-v1/p2p/12D3KooWRBhwfeP2Y4TCx1SM6s9rUoHhR5STiGwxBhgFRcw3UER{}",
            LOCAL_IP, i, i
        )
        .parse()?;
        cache_store.add_addr(addr);
        sleep(Duration::from_millis(100)).await;
    }

    let addrs = cache_store.get_all_addrs().collect::<Vec<_>>();
    assert_eq!(addrs.len(), 2, "Cache should respect max_peers limit");

    Ok(())
}

#[tokio::test]
async fn test_cache_file_corruption() -> Result<(), Box<dyn std::error::Error>> {
    let _guard = LogBuilder::init_single_threaded_tokio_test("cache_tests", false);

    let temp_dir = TempDir::new()?;
    let cache_path = temp_dir.path().join("cache.json");

    let config = BootstrapCacheConfig::empty().with_cache_path(&cache_path);
    let mut cache_store = BootstrapCacheStore::new(config.clone())?;

    let addr: Multiaddr = format!(
        "/ip4/{}/udp/8080/quic-v1/p2p/12D3KooWRBhwfeP2Y4TCx1SM6s9rUoHhR5STiGwxBhgFRcw3UER1",
        LOCAL_IP
    )
    .parse()?;
    cache_store.add_addr(addr.clone());

    assert_eq!(cache_store.peer_count(), 1);

    tokio::fs::write(&cache_path, "invalid json content").await?;

    let mut new_cache_store = BootstrapCacheStore::new(config)?;
    let addrs = new_cache_store.get_all_addrs().collect::<Vec<_>>();
    assert!(addrs.is_empty(), "Cache should be empty after corruption");

    new_cache_store.add_addr(addr);
    let addrs = new_cache_store.get_all_addrs().collect::<Vec<_>>();
    assert_eq!(
        addrs.len(),
        1,
        "Should be able to add peers after corruption"
    );

    Ok(())
}
