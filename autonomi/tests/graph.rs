// Copyright 2024 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use ant_logging::LogBuilder;
use ant_protocol::storage::GraphEntry;
use autonomi::{client::graph::GraphError, Client};
use eyre::Result;
use test_utils::evm::get_funded_wallet;

#[tokio::test]
async fn graph_entry_put() -> Result<()> {
    let _log_appender_guard = LogBuilder::init_single_threaded_tokio_test("graph_entry", false);

    let client = Client::init_local().await?;
    let wallet = get_funded_wallet();

    let key = bls::SecretKey::random();
    let content = [0u8; 32];
    let graph_entry = GraphEntry::new(key.public_key(), vec![], content, vec![], &key);

    // estimate the cost of the graph_entry
    let cost = client.graph_entry_cost(key.clone()).await?;
    println!("graph_entry cost: {cost}");

    // put the graph_entry
    client.graph_entry_put(graph_entry.clone(), &wallet).await?;
    println!("graph_entry put 1");

    // wait for the graph_entry to be replicated
    tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;

    // check that the graph_entry is stored
    let txs = client.graph_entry_get(graph_entry.address()).await?;
    assert_eq!(txs, vec![graph_entry.clone()]);
    println!("graph_entry got 1");

    // try put another graph_entry with the same address
    let content2 = [1u8; 32];
    let graph_entry2 = GraphEntry::new(key.public_key(), vec![], content2, vec![], &key);
    let res = client.graph_entry_put(graph_entry2.clone(), &wallet).await;

    assert!(matches!(
        res,
        Err(GraphError::AlreadyExists(address))
        if address == graph_entry2.address()
    ));
    Ok(())
}
