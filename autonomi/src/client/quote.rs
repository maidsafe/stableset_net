// Copyright 2024 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use super::Client;
use ant_evm::payment_vault::get_market_price;
use ant_evm::{Amount, PaymentQuote, QuotePayment, QuotingMetrics};
use ant_networking::{Network, NetworkError};
use ant_protocol::{storage::ChunkAddress, NetworkAddress, CLOSE_GROUP_SIZE};
use libp2p::PeerId;
use std::collections::HashMap;
use xor_name::XorName;

pub use ant_protocol::storage::DataTypes;

/// A quote for a single address
pub struct QuoteForAddress(pub(crate) Vec<(PeerId, PaymentQuote, Amount)>);

impl QuoteForAddress {
    pub fn price(&self) -> Amount {
        self.0.iter().map(|(_, _, price)| price).sum()
    }
}

/// A quote for many addresses
pub struct StoreQuote(pub HashMap<XorName, QuoteForAddress>);

impl StoreQuote {
    pub fn price(&self) -> Amount {
        self.0.values().map(|quote| quote.price()).sum()
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn payments(&self) -> Vec<QuotePayment> {
        let mut quote_payments = vec![];
        for (_address, quote) in self.0.iter() {
            for (_peer, quote, price) in quote.0.iter() {
                quote_payments.push((quote.hash(), quote.rewards_address, *price));
            }
        }
        quote_payments
    }
}

/// Errors that can occur during the cost calculation.
#[derive(Debug, thiserror::Error)]
pub enum CostError {
    #[error("Failed to self-encrypt data.")]
    SelfEncryption(#[from] crate::self_encryption::Error),
    #[error("Could not get store quote for: {0:?} after several retries")]
    CouldNotGetStoreQuote(XorName),
    #[error("Could not get store costs: {0:?}")]
    CouldNotGetStoreCosts(NetworkError),
    #[error("Not enough node quotes for {0:?}, got: {1:?} and need at least {2:?}")]
    NotEnoughNodeQuotes(XorName, usize, usize),
    #[error("Failed to serialize {0}")]
    Serialization(String),
    #[error("Market price error: {0:?}")]
    MarketPriceError(#[from] ant_evm::payment_vault::error::Error),
    #[error("Received invalid cost")]
    InvalidCost,
}

impl Client {
    pub async fn get_store_quotes(
        &self,
        data_type: DataTypes,
        content_addrs: impl Iterator<Item = (XorName, usize)>,
    ) -> Result<StoreQuote, CostError> {
        // get all quotes from nodes
        let futures: Vec<_> = content_addrs
            .into_iter()
            .map(|(content_addr, data_size)| {
                fetch_store_quote_with_retries(
                    &self.network,
                    content_addr,
                    data_type.get_index(),
                    data_size,
                )
            })
            .collect();

        let raw_quotes_per_addr = futures::future::try_join_all(futures).await?;

        // choose the quotes to pay for each address
        let mut quotes_to_pay_per_addr = HashMap::new();

        for (content_addr, raw_quotes) in raw_quotes_per_addr {
            debug!(
                "fetching market price for content_addr: {content_addr}, with {} quotes.",
                raw_quotes.len()
            );

            // FIXME: find better way to deal with paid content addrs and feedback to the user
            // assume that content addr is already paid for and uploaded
            if raw_quotes.len() <= CLOSE_GROUP_SIZE / 2 {
                debug!("content_addr: {content_addr} is already paid for. No need to fetch market price.");
                continue;
            }

            // ask smart contract for the market price
            let quoting_metrics: Vec<QuotingMetrics> = raw_quotes
                .clone()
                .iter()
                .map(|(_, q)| q.quoting_metrics.clone())
                .collect();

            let all_prices = get_market_price(&self.evm_network, quoting_metrics.clone()).await?;

            debug!("market prices: {all_prices:?}");

            let mut prices: Vec<(PeerId, PaymentQuote, Amount)> = all_prices
                .into_iter()
                .zip(raw_quotes.into_iter())
                .map(|(price, (peer, quote))| (peer, quote, price))
                .collect();

            // sort by price
            prices.sort_by(|(_, _, price_a), (_, _, price_b)| price_a.cmp(price_b));

            // we need at least 5 valid quotes to pay for the data
            const MINIMUM_QUOTES_TO_PAY: usize = 5;
            match &prices[..] {
                [first, second, third, fourth, fifth, ..] => {
                    let (p1, q1, _) = first;
                    let (p2, q2, _) = second;

                    // don't pay for the cheapest 2 quotes but include them
                    let first = (*p1, q1.clone(), Amount::ZERO);
                    let second = (*p2, q2.clone(), Amount::ZERO);

                    // pay for the rest
                    quotes_to_pay_per_addr.insert(
                        content_addr,
                        QuoteForAddress(vec![
                            first,
                            second,
                            third.clone(),
                            fourth.clone(),
                            fifth.clone(),
                        ]),
                    );
                }
                _ => {
                    return Err(CostError::NotEnoughNodeQuotes(
                        content_addr,
                        prices.len(),
                        MINIMUM_QUOTES_TO_PAY,
                    ));
                }
            }
        }

        Ok(StoreQuote(quotes_to_pay_per_addr))
    }
}

/// Fetch a store quote for a content address.
async fn fetch_store_quote(
    network: &Network,
    content_addr: XorName,
    data_type: u32,
    data_size: usize,
) -> Result<Vec<(PeerId, PaymentQuote)>, NetworkError> {
    network
        .get_store_quote_from_network(
            NetworkAddress::from_chunk_address(ChunkAddress::new(content_addr)),
            data_type,
            data_size,
            vec![],
        )
        .await
}

/// Fetch a store quote for a content address with a retry strategy.
async fn fetch_store_quote_with_retries(
    network: &Network,
    content_addr: XorName,
    data_type: u32,
    data_size: usize,
) -> Result<(XorName, Vec<(PeerId, PaymentQuote)>), CostError> {
    let mut retries = 0;

    loop {
        match fetch_store_quote(network, content_addr, data_type, data_size).await {
            Ok(quote) => {
                if quote.len() < CLOSE_GROUP_SIZE {
                    retries += 1;
                    error!("Error while fetching store quote: not enough quotes ({}/{CLOSE_GROUP_SIZE}), retry #{retries}, quotes {quote:?}",
                        quote.len());
                    if retries > 2 {
                        break Err(CostError::CouldNotGetStoreQuote(content_addr));
                    }
                }
                break Ok((content_addr, quote));
            }
            Err(err) if retries < 2 => {
                retries += 1;
                error!("Error while fetching store quote: {err:?}, retry #{retries}");
            }
            Err(err) => {
                error!(
                    "Error while fetching store quote: {err:?}, stopping after {retries} retries"
                );
                break Err(CostError::CouldNotGetStoreQuote(content_addr));
            }
        }
        // Shall have a sleep between retries to avoid choking the network.
        // This shall be rare to happen though.
        tokio::time::sleep(std::time::Duration::from_secs(5)).await;
    }
}
