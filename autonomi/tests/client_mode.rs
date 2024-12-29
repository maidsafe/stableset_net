// Copyright 2024 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use ant_evm::EvmWallet;
use ant_logging::LogBuilder;
use autonomi::{Client, ClientConfig};
use test_utils::evm::get_funded_wallet;

#[tokio::test]
async fn test_read_only_client() -> anyhow::Result<()> {
    let _log_appender_guard = LogBuilder::init_single_threaded_tokio_test("client_mode", false);

    // Initialize a read-only client
    let client = Client::init_read_only().await?;
    assert!(!client.can_write());
    assert!(client.wallet().is_none());

    Ok(())
}

#[tokio::test]
async fn test_read_write_client() -> anyhow::Result<()> {
    let _log_appender_guard = LogBuilder::init_single_threaded_tokio_test("client_mode", false);

    // Get a funded wallet for testing
    let wallet = get_funded_wallet();

    // Initialize a read-write client with wallet
    let client = Client::init_with_wallet(wallet).await?;
    assert!(client.can_write());
    assert!(client.wallet().is_some());

    Ok(())
}

#[tokio::test]
async fn test_upgrade_to_read_write() -> anyhow::Result<()> {
    let _log_appender_guard = LogBuilder::init_single_threaded_tokio_test("client_mode", false);

    // Initialize a read-only client
    let mut client = Client::init_read_only().await?;
    assert!(!client.can_write());
    assert!(client.wallet().is_none());

    // Get a funded wallet for testing
    let wallet = get_funded_wallet();

    // Upgrade to read-write mode
    client.upgrade_to_read_write(wallet)?;
    assert!(client.can_write());
    assert!(client.wallet().is_some());

    Ok(())
}

#[tokio::test]
async fn test_upgrade_already_read_write() -> anyhow::Result<()> {
    let _log_appender_guard = LogBuilder::init_single_threaded_tokio_test("client_mode", false);

    // Get a funded wallet for testing
    let wallet = get_funded_wallet();

    // Initialize a read-write client
    let mut client = Client::init_with_wallet(wallet).await?;
    assert!(client.can_write());

    // Try to upgrade an already read-write client
    let wallet = get_funded_wallet();
    let result = client.upgrade_to_read_write(wallet);
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("already in read-write mode"));

    Ok(())
}

#[tokio::test]
async fn test_client_with_config() -> anyhow::Result<()> {
    let _log_appender_guard = LogBuilder::init_single_threaded_tokio_test("client_mode", false);

    // Test read-only client with default config
    let client = Client::init_read_only_with_config(Default::default()).await?;
    assert!(!client.can_write());
    assert!(client.wallet().is_none());

    // Test read-write client with default config
    let wallet = get_funded_wallet();
    let client = Client::init_with_wallet_and_config(wallet, Default::default()).await?;
    assert!(client.can_write());
    assert!(client.wallet().is_some());

    Ok(())
}

#[tokio::test]
async fn test_write_operations_without_wallet() -> anyhow::Result<()> {
    let _log_appender_guard = LogBuilder::init_single_threaded_tokio_test("client_mode", false);

    // Initialize a read-only client
    let client = Client::init_read_only().await?;

    // Try to perform a write operation (we'll use check_write_access directly since it's private)
    let result = client.check_write_access();
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), autonomi::PutError::NoWallet));

    Ok(())
}
