// Copyright 2024 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use crate::client::data::{DataMapChunk, CHUNK_UPLOAD_BATCH_SIZE};
use crate::client::{
    error::{GetError, PutError},
    payment::PaymentOption,
};
use crate::Client;
use anyhow::Result;
use bytes::Bytes;
use futures::{Stream, StreamExt};
use self_encryption::DataMap;
use std::path::Path;
use tokio::fs::File;
use tokio::io::{AsyncRead, AsyncWrite, AsyncWriteExt, ReadBuf};
use tracing::info;

// Use a 1MB buffer size for streaming
const STREAM_BUFFER_SIZE: usize = 1024 * 1024;

/// A stream of data chunks for uploading
pub struct UploadStream<R> {
    reader: R,
    buffer: Vec<u8>,
    position: usize,
    total_bytes: u64,
}

impl<R: AsyncRead + Unpin> UploadStream<R> {
    /// Create a new upload stream from an async reader
    pub fn new(reader: R) -> Self {
        Self {
            reader,
            buffer: vec![0; STREAM_BUFFER_SIZE],
            position: 0,
            total_bytes: 0,
        }
    }
}

impl<R: AsyncRead + Unpin> Stream for UploadStream<R> {
    type Item = Result<Bytes, std::io::Error>;

    fn poll_next(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        use std::task::Poll;

        let this = &mut *self;

        // If we've reached the end of the buffer, read more data
        if this.position >= this.buffer.len() {
            let mut read_buf = ReadBuf::new(&mut this.buffer);
            match futures::ready!(std::pin::Pin::new(&mut this.reader).poll_read(cx, &mut read_buf))
            {
                Ok(()) => {
                    let n = read_buf.filled().len();
                    if n == 0 {
                        return Poll::Ready(None); // EOF
                    }
                    this.position = 0;
                    this.total_bytes += n as u64;
                    Poll::Ready(Some(Ok(Bytes::copy_from_slice(read_buf.filled()))))
                }
                Err(e) => Poll::Ready(Some(Err(e))),
            }
        } else {
            // Return data from the buffer
            let remaining = this.buffer.len() - this.position;
            let chunk =
                Bytes::copy_from_slice(&this.buffer[this.position..this.position + remaining]);
            this.position += remaining;
            Poll::Ready(Some(Ok(chunk)))
        }
    }
}

/// A stream of data chunks for downloading
pub struct DownloadStream<W> {
    writer: W,
    data_map: DataMap,
    current_chunk: usize,
}

impl<W: AsyncWrite + Unpin> DownloadStream<W> {
    /// Create a new download stream to an async writer
    pub fn new(writer: W, data_map: DataMap) -> Self {
        Self {
            writer,
            data_map,
            current_chunk: 0,
        }
    }

    /// Write a chunk of data to the stream
    pub async fn write_chunk(&mut self, chunk: Bytes) -> Result<(), std::io::Error> {
        self.writer.write_all(&chunk).await?;
        self.current_chunk += 1;
        Ok(())
    }

    /// Check if all chunks have been written
    pub fn is_complete(&self) -> bool {
        self.current_chunk >= self.data_map.chunk_identifiers.len()
    }
}

#[derive(Debug, thiserror::Error)]
pub enum StreamError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Put error: {0}")]
    Put(#[from] PutError),
    #[error("Get error: {0}")]
    Get(#[from] GetError),
}

impl Client {
    /// Upload a file using streaming encryption
    pub async fn upload_streaming<P: AsRef<Path>>(
        &self,
        path: P,
        payment_option: PaymentOption,
    ) -> Result<DataMapChunk, StreamError> {
        let file = File::open(path).await?;
        let stream = UploadStream::new(file);

        let mut chunks = Vec::new();
        let mut stream = Box::pin(stream);

        // Collect chunks in batches and upload them
        let mut current_batch = Vec::new();
        let batch_size = *CHUNK_UPLOAD_BATCH_SIZE;

        while let Some(chunk) = stream.next().await {
            current_batch.push(chunk?);

            // When we have enough chunks for a batch, process them
            if current_batch.len() >= batch_size {
                info!("Processing batch of {} chunks", current_batch.len());
                let batch_data = Bytes::from(current_batch.concat());
                let data_map = self.data_put(batch_data, payment_option.clone()).await?;
                chunks.push(data_map);
                current_batch.clear();
            }
        }

        // Process any remaining chunks
        if !current_batch.is_empty() {
            info!("Processing final batch of {} chunks", current_batch.len());
            let batch_data = Bytes::from(current_batch.concat());
            let data_map = self.data_put(batch_data, payment_option.clone()).await?;
            chunks.push(data_map);
        }

        // If we only have one chunk, return it directly
        if chunks.len() == 1 {
            Ok(chunks.pop().unwrap())
        } else {
            // Otherwise combine the chunks into a final data map
            let combined_data = chunks.into_iter().fold(Vec::new(), |mut acc, chunk| {
                acc.extend(chunk.to_hex().as_bytes());
                acc
            });
            Ok(self
                .data_put(Bytes::from(combined_data), payment_option.clone())
                .await?)
        }
    }

    /// Download a file using streaming decryption
    pub async fn download_streaming<P: AsRef<Path>>(
        &self,
        data_map: DataMapChunk,
        path: P,
    ) -> Result<(), StreamError> {
        let file = File::create(path).await?;
        let mut writer = tokio::io::BufWriter::new(file);

        // For downloads we can parallelize fully
        let data = self.data_get(data_map).await?;

        // Check if this is a combined data map
        if let Ok(hex) = std::str::from_utf8(&data) {
            if hex.len() % 2 == 0 && hex.chars().all(|c| c.is_ascii_hexdigit()) {
                // This is a combined data map, download each chunk in parallel
                let chunk_maps: Vec<_> = (0..hex.len())
                    .step_by(2)
                    .filter_map(|i| DataMapChunk::from_hex(&hex[i..i + 2]).ok())
                    .collect();

                let mut futures = Vec::new();
                for chunk_map in chunk_maps {
                    futures.push(self.data_get(chunk_map));
                }

                let chunks = futures::future::join_all(futures).await;
                for chunk in chunks {
                    writer.write_all(&chunk?).await?;
                }
            } else {
                writer.write_all(&data).await?;
            }
        } else {
            writer.write_all(&data).await?;
        }

        writer.flush().await?;
        Ok(())
    }
}

#[cfg(test)]
#[cfg(feature = "local")]
mod tests {
    use super::*;
    use crate::client::payment::Receipt;
    use crate::network::LocalNode;
    use crate::ClientConfig;
    use tempfile::NamedTempFile;
    use tokio::fs::File;
    use tokio::io::AsyncReadExt;

    #[tokio::test]
    async fn test_streaming_upload_download() -> Result<(), StreamError> {
        // Start a local node first
        let local_node = LocalNode::start()
            .await
            .map_err(|e| StreamError::Put(PutError::Serialization(e.to_string())))?;

        // Create test data
        let temp_file = NamedTempFile::new().map_err(StreamError::Io)?;
        let test_data = b"Hello, World!".repeat(1000);
        std::fs::write(temp_file.path(), &test_data).map_err(StreamError::Io)?;

        // Initialize client with local config
        let config = ClientConfig {
            local: true,
            peers: Some(vec![local_node.get_multiaddr()]),
        };
        let client = Client::init_with_config(config)
            .await
            .map_err(|e| StreamError::Put(PutError::Serialization(e.to_string())))?;

        // Upload the file
        let data_map = client
            .upload_streaming(temp_file.path(), PaymentOption::Receipt(Receipt::new()))
            .await?;

        // Create a new temp file for downloading
        let download_file = NamedTempFile::new().map_err(StreamError::Io)?;
        client
            .download_streaming(data_map, download_file.path())
            .await?;

        // Verify the downloaded content matches the original
        let mut original = File::open(temp_file.path())
            .await
            .map_err(StreamError::Io)?;
        let mut downloaded = File::open(download_file.path())
            .await
            .map_err(StreamError::Io)?;

        let mut original_content = Vec::new();
        let mut downloaded_content = Vec::new();
        original
            .read_to_end(&mut original_content)
            .await
            .map_err(StreamError::Io)?;
        downloaded
            .read_to_end(&mut downloaded_content)
            .await
            .map_err(StreamError::Io)?;

        assert_eq!(original_content, downloaded_content);
        assert_eq!(original_content, test_data);
        Ok(())
    }
}
