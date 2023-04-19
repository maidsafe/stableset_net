// Copyright 2023 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

mod cli;

use self::cli::{cfg_cmds, files_cmds, register_cmds, wallet_cmds, Opt};

use safenode::client::{Client, ClientEvent};

use clap::Parser;
use eyre::Result;
use tracing::info;

#[tokio::main]
async fn main() -> Result<()> {
    let opt = Opt::parse();

    if let Opt::Cfg(cmds) = &opt {
        cfg_cmds(cmds).await?;
    }

    info!("Instantiating a SAFE client...");

    let secret_key = bls::SecretKey::random();
    let client = Client::new(secret_key)?;

    let mut client_events_rx = client.events_channel();
    if let Ok(event) = client_events_rx.recv().await {
        match event {
            ClientEvent::ConnectedToNetwork => {
                info!("Client connected to the Network");
            }
        }
    }

    match opt {
        Opt::Cfg(cmds) => cfg_cmds(&cmds).await?,
        Opt::Wallet(cmds) => wallet_cmds(cmds, &client).await?,
        Opt::Files(cmds) => files_cmds(cmds, client.clone()).await?,
        Opt::Register(cmds) => register_cmds(cmds, &client).await?,
    };

    Ok(())
}
