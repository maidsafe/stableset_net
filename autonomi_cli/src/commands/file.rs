// Copyright 2024 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use autonomi::client::address::{str_to_xorname, xorname_to_str};
use autonomi::Wallet;
use autonomi::Multiaddr;
use color_eyre::eyre::{eyre, Context};
use color_eyre::eyre::Result;
use std::path::PathBuf;

pub async fn cost(file: &str, peers: Vec<Multiaddr>) -> Result<()> {
    let mut client = crate::actions::connect_to_network(peers).await?;

    println!("Getting upload cost...");
    let cost = client.file_cost(&PathBuf::from(file)).await
        .wrap_err("Failed to calculate cost for file")?;

    println!("Estimate cost to upload file: {file}");
    println!("Total cost: {cost}");
    Ok(())
}

pub async fn upload(file: &str, peers: Vec<Multiaddr>) -> Result<()> {
    let secret_key = crate::utils::get_secret_key()
        .wrap_err("The secret key is required to perform this action")?;
    let network = crate::utils::get_evm_network()
        .wrap_err("Failed to get evm network")?;
    let wallet = Wallet::new_from_private_key(network, &secret_key)
        .wrap_err("Failed to load wallet")?;

    let mut client = crate::actions::connect_to_network(peers).await?;

    println!("Uploading data to network...");
    let (_, xor_name) = client.upload_from_dir(PathBuf::from(file), &wallet).await
        .wrap_err("Failed to upload file")?;
    let addr = xorname_to_str(xor_name);

    println!("Successfully uploaded: {file}");
    println!("At address: {addr}");
    Ok(())
}

pub async fn download(addr: &str, dest_path: &str, peers: Vec<Multiaddr>) -> Result<()> {
    let mut client = crate::actions::connect_to_network(peers).await?;

    println!("Downloading data from {addr} to {dest_path}");
    let address = str_to_xorname(addr)
        .wrap_err("Failed to parse data address")?;
    let root = client.fetch_root(address).await
        .wrap_err("Failed to fetch root")?;

    let mut all_errs = vec![];
    for (path, file) in root.map {
        println!("Fetching file: {path:?}");
        let bytes = match client.fetch_file(&file).await {
            Ok(bytes) => bytes,
            Err(e) => {
                let err = format!("Failed to fetch file {path:?}: {e}");
                all_errs.push(err);
                continue;
            }
        };

        let path = PathBuf::from(dest_path).join(path);
        let here = PathBuf::from(".");
        let parent = path.parent().unwrap_or_else(|| &here);
        std::fs::create_dir_all(parent)?;
        std::fs::write(path, bytes)?;
    }

    if all_errs.is_empty() {
        println!("Successfully downloaded data at: {addr}");
        Ok(())
    } else {
        let err_no = all_errs.len();
        eprintln!("{err_no} errors while downloading data at: {addr}");
        eprintln!("{all_errs:#?}");
        Err(eyre!("Errors while downloading data"))
    }
}

pub fn list(peers: Vec<Multiaddr>) -> Result<()> {
    println!("Listing previous uploads...");
    Ok(())
}
