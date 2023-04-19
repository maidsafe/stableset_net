// Copyright 2023 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use safenode::{
    client::{Client, Files},
    protocol::address::ChunkAddress,
};

use bytes::Bytes;
use clap::Parser;
use eyre::Result;
use std::{fs, path::PathBuf};
use tracing::info;
use walkdir::WalkDir;
use xor_name::XorName;

#[derive(Parser, Debug)]
pub enum FilesCmds {
    Upload {
        /// The location of the files to upload.
        #[clap(name = "files-path")]
        files_path: PathBuf,
    },
    Download {
        /// The location of the file names stored
        /// when uploading files.
        #[clap(name = "file-names-path")]
        file_names_path: PathBuf,
    },
}

pub(crate) async fn files_cmds(cmds: FilesCmds, client: Client) -> Result<()> {
    let file_api: Files = Files::new(client);
    match cmds {
        FilesCmds::Upload { files_path } => upload_files(files_path, &file_api).await?,
        FilesCmds::Download { file_names_path } => {
            download_files(file_names_path, &file_api).await?
        }
    };
    Ok(())
}

async fn upload_files(files_path: PathBuf, file_api: &Files) -> Result<()> {
    let file_names_path = files_path.join("uploaded_files/file_names.txt");
    let mut chunks_to_fetch = Vec::new();

    for entry in WalkDir::new(files_path).into_iter().flatten() {
        if entry.file_type().is_file() {
            let file = fs::read(entry.path())?;
            let bytes = Bytes::from(file);
            let file_name = entry.file_name();

            info!("Storing file {file_name:?} of {} bytes..", bytes.len());
            println!("Storing file {file_name:?}.");

            match file_api.upload(bytes).await {
                Ok(address) => {
                    info!("Successfully stored file to {address:?}");
                    chunks_to_fetch.push(*address.name());
                }
                Err(error) => {
                    panic!(
                        "Did not store file {file_name:?} to all nodes in the close group! {error}"
                    )
                }
            };
        }
    }

    let content = bincode::serialize(&chunks_to_fetch)?;
    tokio::fs::create_dir_all(file_names_path.as_path()).await?;
    fs::write(file_names_path, content)?;

    Ok(())
}

async fn download_files(file_names_dir: PathBuf, file_api: &Files) -> Result<()> {
    for entry in WalkDir::new(file_names_dir).into_iter().flatten() {
        if entry.file_type().is_file() {
            let file = fs::read(entry.path())?;
            let bytes = Bytes::from(file);
            let file_name = entry.file_name();

            info!("Loading file xornames from {file_name:?}");
            println!("Loading file xornames from {file_name:?}");
            let chunks_to_fetch: Vec<XorName> = bincode::deserialize(&bytes)?;

            for xorname in chunks_to_fetch.iter() {
                info!("Downloading file {xorname:?}");
                println!("Downloading file {xorname:?}");
                match file_api.read_bytes(ChunkAddress::new(*xorname)).await {
                    Ok(bytes) => info!("Successfully got file {xorname} of {} bytes!", bytes.len()),
                    Err(error) => {
                        panic!("Did not get file {xorname:?} from the network! {error}")
                    }
                };
            }
        }
    }

    Ok(())
}
