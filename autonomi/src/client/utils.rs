// Copyright 2024 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use futures::stream::{FuturesUnordered, StreamExt};
use std::future::Future;

pub(crate) async fn process_tasks_with_max_concurrency<I, R>(tasks: I, batch_size: usize) -> Vec<R>
where
    I: IntoIterator,
    I::Item: Future<Output = R> + Send,
    R: Send,
{
    let mut futures = FuturesUnordered::new();
    let mut results = Vec::new();

    for task in tasks.into_iter() {
        futures.push(task);

        if futures.len() >= batch_size {
            if let Some(result) = futures.next().await {
                results.push(result);
            }
        }
    }

    // Process remaining tasks
    while let Some(result) = futures.next().await {
        results.push(result);
    }

    results
}
