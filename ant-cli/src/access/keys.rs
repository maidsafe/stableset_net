// Copyright 2024 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use crate::wallet::load_wallet_private_key;
use autonomi::client::vault::VaultSecretKey;
use autonomi::{Network, Wallet};
use color_eyre::eyre::{eyre, Context, Result};
use std::env;

const SECRET_KEY_ENV: &str = "SECRET_KEY";

/// EVM wallet
pub fn load_evm_wallet_from_env(evm_network: &Network) -> Result<Wallet> {
    let secret_key =
        get_secret_key_from_env().wrap_err("The secret key is required to perform this action")?;
    let wallet = Wallet::new_from_private_key(evm_network.clone(), &secret_key)
        .wrap_err("Failed to load EVM wallet from key")?;
    Ok(wallet)
}

/// EVM wallet private key
pub fn get_secret_key_from_env() -> Result<String> {
    env::var(SECRET_KEY_ENV).wrap_err(eyre!(
        "make sure you've provided the {SECRET_KEY_ENV} env var"
    ))
}

pub fn get_vault_secret_key() -> Result<VaultSecretKey> {
    let secret_key = load_wallet_private_key()?;
    autonomi::client::vault::derive_vault_key(&secret_key)
        .wrap_err("Failed to derive vault secret key from EVM secret key")
}
