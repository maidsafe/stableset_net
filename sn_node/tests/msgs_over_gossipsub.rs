// Copyright 2023 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

mod common;

use common::safenode_proto::{
    safe_node_client::SafeNodeClient, GossipsubPublishRequest, GossipsubSubscribeRequest,
    GossipsubUnsubscribeRequest, NodeEventsRequest,
};
use sn_node::NodeEvent;

use eyre::Result;
use std::{
    net::{IpAddr, Ipv4Addr, SocketAddr},
    time::Duration,
};
use tokio::time::timeout;
use tokio_stream::StreamExt;
use tonic::Request;

const NODE_COUNT: u8 = 25;
const NODES_SUBSCRIBED: u8 = NODE_COUNT / 2; // 12 out of 25 nodes will be subscribers
const TEST_CYCLES: u8 = 20;

#[tokio::test]
async fn msgs_over_gossipsub() -> Result<()> {
    let mut addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 12000);
    for c in 0..TEST_CYCLES {
        let topic = format!("TestTopic-{}", rand::random::<u64>());
        println!("Testing cicle {}/{TEST_CYCLES} - topic: {topic}", c + 1);
        println!("============================================================");

        let mut subs_addrs = vec![];
        let mut subs_handles = vec![];

        // get a random subset of NODES_SUBSCRIBED out of NODE_COUNT nodes to subscribe to the topic
        let mut rng = rand::thread_rng();
        let random_indexes =
            rand::seq::index::sample(&mut rng, NODE_COUNT.into(), NODES_SUBSCRIBED.into())
                .into_vec();

        for node_index in random_indexes {
            // request current node to subscribe to the topic
            addr.set_port(12000 + node_index as u16);
            node_subscribe_to_topic(addr, topic.clone()).await?;
            subs_addrs.push(addr);

            println!("Node {node_index} subscribed to {topic}");

            let handle = tokio::spawn(async move {
                let endpoint = format!("https://{addr}");
                let mut rpc_client = SafeNodeClient::connect(endpoint).await?;
                let response = rpc_client
                    .node_events(Request::new(NodeEventsRequest {}))
                    .await?;

                println!("Listening to node events...");
                let mut count = 0;

                let _ = timeout(Duration::from_millis(6000), async {
                    let mut stream = response.into_inner();
                    while let Some(Ok(e)) = stream.next().await {
                        match NodeEvent::from_bytes(&e.event) {
                            Ok(NodeEvent::GossipsubMsg { topic, msg }) => {
                                println!(
                                    "New gossipsub msg received on '{topic}': {}",
                                    String::from_utf8(msg).unwrap()
                                );
                                count += 1;
                                if count == NODE_COUNT - NODES_SUBSCRIBED {
                                    break;
                                }
                            }
                            Ok(_) => { /* ignored */ }
                            Err(_) => {
                                println!("Error while parsing received NodeEvent");
                            }
                        }
                    }
                })
                .await;

                Ok::<u8, eyre::Error>(count)
            });

            subs_handles.push((node_index, addr, handle));
        }

        tokio::time::sleep(Duration::from_millis(3000)).await;

        // have all other nodes to publish each a different msg to that same topic
        other_nodes_to_publish_on_topic(subs_addrs, topic.clone()).await?;

        for (node_index, addr, handle) in subs_handles.into_iter() {
            let count = handle.await??;
            println!("Messages received by node {node_index}: {count}");
            assert_eq!(
                count,
                NODE_COUNT - NODES_SUBSCRIBED,
                "Not enough messages received by node at index {}",
                node_index
            );
            node_unsubscribe_from_topic(addr, topic.clone()).await?;
        }
    }

    Ok(())
}

async fn node_subscribe_to_topic(addr: SocketAddr, topic: String) -> Result<()> {
    let endpoint = format!("https://{addr}");
    let mut rpc_client = SafeNodeClient::connect(endpoint).await?;

    // subscribe to given topic
    let _response = rpc_client
        .subscribe_to_topic(Request::new(GossipsubSubscribeRequest { topic }))
        .await?;

    Ok(())
}

async fn node_unsubscribe_from_topic(addr: SocketAddr, topic: String) -> Result<()> {
    let endpoint = format!("https://{addr}");
    let mut rpc_client = SafeNodeClient::connect(endpoint).await?;

    // unsubscribe from given topic
    let _response = rpc_client
        .unsubscribe_from_topic(Request::new(GossipsubUnsubscribeRequest { topic }))
        .await?;

    Ok(())
}

async fn other_nodes_to_publish_on_topic(
    filter_addrs: Vec<SocketAddr>,
    topic: String,
) -> Result<()> {
    let mut addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 12000);
    for node_index in 1..NODE_COUNT + 1 {
        addr.set_port(12000 + node_index as u16);
        if filter_addrs.iter().all(|a| a != &addr) {
            let msg = format!("TestMsgOnTopic-{topic}-from-{node_index}");

            let endpoint = format!("https://{addr}");
            let mut rpc_client = SafeNodeClient::connect(endpoint).await?;
            println!("Node {node_index} to publish on {topic} message: {msg}");

            let _response = rpc_client
                .publish_on_topic(Request::new(GossipsubPublishRequest {
                    topic: topic.clone(),
                    msg: msg.into(),
                }))
                .await?;
        }
    }

    Ok(())
}
