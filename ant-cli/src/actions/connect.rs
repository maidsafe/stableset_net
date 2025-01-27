// Copyright 2024 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use crate::network::NetworkPeers;
use autonomi::{get_evm_network, Client, ClientConfig};
use color_eyre::eyre::bail;
use color_eyre::eyre::Result;
use indicatif::ProgressBar;
use std::time::Duration;

pub async fn connect_to_network(peers: NetworkPeers) -> Result<Client> {
    let progress_bar = ProgressBar::new_spinner();
    progress_bar.enable_steady_tick(Duration::from_millis(120));
    progress_bar.set_message("Connecting to The Autonomi Network...");
    let new_style = progress_bar.style().tick_chars("⠁⠂⠄⡀⢀⠠⠐⠈🔗");
    progress_bar.set_style(new_style);

    let local = peers.is_local();

    if local {
        progress_bar.set_message("Connecting to a local Autonomi Network...");
    } else {
        progress_bar.set_message("Connecting to The Autonomi Network...");
    };

    let evm_network = get_evm_network(local)?;

    let peers_opt = match peers.peers().to_vec() {
        peers if peers.is_empty() => None,
        peers => Some(peers),
    };

    let config = ClientConfig {
        local,
        peers: peers_opt,
        evm_network,
    };

    let res = Client::init_with_config(config).await;

    match res {
        Ok(client) => {
            info!("Connected to the Network");
            progress_bar.finish_with_message("Connected to the Network");
            Ok(client)
        }
        Err(e) => {
            error!("Failed to connect to the network: {e}");
            progress_bar.finish_with_message("Failed to connect to the network");
            bail!("Failed to connect to the network: {e}")
        }
    }
}
