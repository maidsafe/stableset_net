// Copyright 2024 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

// Copyright 2024 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use super::archive_private::{PrivateArchive, PrivateArchiveAccess};
use super::{get_relative_file_path_from_abs_file_and_folder_path, FILE_UPLOAD_BATCH_SIZE};
use super::{DownloadError, UploadError};

use crate::client::event::FileEvent;
use crate::client::Client;
use crate::client::{data_types::chunk::DataMapChunk, utils::process_tasks_with_max_concurrency};
use crate::ClientEvent;
use crate::{AttoTokens, Wallet};
use bytes::Bytes;
use std::path::PathBuf;

impl Client {
    /// Download a private file from network to local file system
    pub async fn file_download(
        &self,
        data_access: &DataMapChunk,
        to_dest: PathBuf,
    ) -> Result<(), DownloadError> {
        let data = self.data_get(data_access).await?;
        if let Some(parent) = to_dest.parent() {
            tokio::fs::create_dir_all(parent).await?;
            debug!("Created parent directories for {to_dest:?}");
        }
        tokio::fs::write(to_dest.clone(), data).await?;
        debug!("Downloaded file to {to_dest:?}");
        Ok(())
    }

    /// Download a private directory from network to local file system
    pub async fn dir_download(
        &self,
        archive_access: &PrivateArchiveAccess,
        to_dest: PathBuf,
    ) -> Result<(), DownloadError> {
        let archive = self.archive_get(archive_access).await?;
        for (path, addr, _meta) in archive.iter() {
            self.file_download(addr, to_dest.join(path)).await?;
        }
        debug!("Downloaded directory to {to_dest:?}");
        Ok(())
    }

    /// Upload a directory to the network. The directory is recursively walked and each file is uploaded to the network.
    /// The data maps of these (private) files are not uploaded but returned within the [`PrivateArchive`] return type.
    pub async fn dir_upload(
        &self,
        dir_path: PathBuf,
        wallet: &Wallet,
    ) -> Result<(AttoTokens, PrivateArchive), UploadError> {
        info!("Uploading directory as private: {dir_path:?}");
        let start = tokio::time::Instant::now();

        // start upload of file in parallel
        let mut upload_tasks = Vec::new();
        for entry in walkdir::WalkDir::new(dir_path.clone()) {
            let entry = entry?;
            if !entry.file_type().is_file() {
                continue;
            }

            let metadata = super::fs_public::metadata_from_entry(&entry);
            let path = entry.path().to_path_buf();
            upload_tasks.push(async move {
                let res = self.file_upload(path.clone(), wallet).await;
                (path, metadata, res)
            });
        }

        // wait for all files to be uploaded
        let uploads =
            process_tasks_with_max_concurrency(upload_tasks, *FILE_UPLOAD_BATCH_SIZE).await;
        info!(
            "Upload of {} files completed in {:?}",
            uploads.len(),
            start.elapsed()
        );
        let mut archive = PrivateArchive::new();
        let mut total_cost = AttoTokens::zero();
        for (path, metadata, maybe_file) in uploads.into_iter() {
            let rel_path = get_relative_file_path_from_abs_file_and_folder_path(&path, &dir_path);

            match maybe_file {
                Ok((cost, file)) => {
                    archive.add_file(rel_path, file, metadata);
                    total_cost = total_cost.checked_add(cost).unwrap_or_else(|| {
                        error!("Total cost overflowed: {total_cost:?} + {cost:?}");
                        total_cost
                    });
                }
                Err(err) => {
                    error!("Failed to upload file: {path:?}: {err:?}");
                    return Err(err);
                }
            }
        }

        Ok((total_cost, archive))
    }

    /// Same as [`Client::dir_upload`] but also uploads the archive (privately) to the network.
    ///
    /// Returns the [`PrivateArchiveAccess`] allowing the private archive to be downloaded from the network.
    pub async fn dir_and_archive_upload(
        &self,
        dir_path: PathBuf,
        wallet: &Wallet,
    ) -> Result<(AttoTokens, PrivateArchiveAccess), UploadError> {
        let (cost1, archive) = self.dir_upload(dir_path, wallet).await?;
        let (cost2, archive_addr) = self.archive_put(&archive, wallet.into()).await?;
        let total_cost = cost1.checked_add(cost2).unwrap_or_else(|| {
            error!("Total cost overflowed: {cost1:?} + {cost2:?}");
            cost1
        });
        Ok((total_cost, archive_addr))
    }

    /// Upload a private file to the network.
    /// Reads file, splits into chunks, uploads chunks, uploads datamap, returns [`DataMapChunk`] (pointing to the datamap)
    async fn file_upload(
        &self,
        path: PathBuf,
        wallet: &Wallet,
    ) -> Result<(AttoTokens, DataMapChunk), UploadError> {
        info!("Uploading file: {path:?}");

        if let Some(sender) = self.client_event_sender.as_ref() {
            let _ = sender
                .send(ClientEvent::File(FileEvent::UploadingFile {
                    path: path.clone(),
                    public: false,
                }))
                .await
                .inspect_err(|err| {
                    error!("Failed to send client event: {err:?}");
                });
        }

        let data = tokio::fs::read(path).await?;
        let data = Bytes::from(data);
        let (total_cost, addr) = self.data_put(data, wallet.into()).await?;
        debug!("Uploaded file successfully in the privateAchive: {addr:?}");
        Ok((total_cost, addr))
    }
}
