// Copyright 2024 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

#[macro_use]
extern crate tracing;

mod access;
mod actions;
mod commands;
mod exit_code;
mod opt;
mod output;
mod wallet;

pub use access::data_dir;
pub use access::keys;
pub use access::network;
pub use access::user_data;

use clap::Parser;
use color_eyre::Result;

use ant_logging::metrics::init_metrics;
use ant_logging::{LogBuilder, LogFormat, ReloadHandle, WorkerGuard};
use ant_protocol::version;
use opt::Opt;
use serde_json::json;
use tracing::Level;

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install().expect("Failed to initialise error handler");
    let opt = Opt::parse();
    if let Some(network_id) = opt.network_id {
        ant_protocol::version::set_network_id(network_id);
    }

    // The clone is necessary to resolve a clippy warning related to a mutex.
    let identify_protocol_str = version::IDENTIFY_PROTOCOL_STR
        .read()
        .expect("Failed to obtain read lock for IDENTIFY_PROTOCOL_STR")
        .clone();
    if opt.version {
        let version_info = ant_build_info::get_version_info(
            "Autonomi Client",
            env!("CARGO_PKG_VERSION"),
            Some(&identify_protocol_str),
        );
        if opt.json {
            eprintln!("{}", serde_json::to_string_pretty(&version_info)?);
        } else {
            eprintln!("{version_info}");
        }

        return Ok(());
    }

    if opt.crate_version {
        let crate_version = env!("CARGO_PKG_VERSION");
        if opt.json {
            let json_value = json!({
                "crate_version": crate_version,
            });
            eprintln!("{}", serde_json::to_string_pretty(&json_value)?);
        } else {
            eprintln!("Crate version: {}", env!("CARGO_PKG_VERSION"));
        }
    }

    if opt.protocol_version {
        if opt.json {
            let json_value = json!({
                "protocol_version": identify_protocol_str,
            });
            eprintln!("{}", serde_json::to_string_pretty(&json_value)?);
        } else {
            eprintln!("Protocol version: {identify_protocol_str}");
        }
        return Ok(());
    }

    #[cfg(not(feature = "nightly"))]
    if opt.package_version {
        if opt.json {
            let json_value = json!({
                "package_version": ant_build_info::package_version(),
            });
            eprintln!("{}", serde_json::to_string_pretty(&json_value)?);
        } else {
            eprintln!("Package version: {}", ant_build_info::package_version());
        }
        return Ok(());
    }

    let _log_guards = init_logging_and_metrics(&opt)?;
    if opt.peers.local {
        tokio::spawn(init_metrics(std::process::id()));
    }

    info!("\"{}\"", std::env::args().collect::<Vec<_>>().join(" "));
    let version = ant_build_info::git_info();
    info!("autonomi client built with git version: {version}");

    commands::handle_subcommand(opt).await?;

    Ok(())
}

fn init_logging_and_metrics(opt: &Opt) -> Result<(ReloadHandle, Option<WorkerGuard>)> {
    let logging_targets = vec![
        ("ant_bootstrap".to_string(), Level::DEBUG),
        ("ant_build_info".to_string(), Level::TRACE),
        ("ant_evm".to_string(), Level::TRACE),
        ("ant_networking".to_string(), Level::INFO),
        ("autonomi_cli".to_string(), Level::TRACE),
        ("autonomi".to_string(), Level::TRACE),
        ("evmlib".to_string(), Level::TRACE),
        ("ant_logging".to_string(), Level::TRACE),
        ("ant_protocol".to_string(), Level::TRACE),
    ];
    let mut log_builder = LogBuilder::new(logging_targets);
    log_builder.output_dest(opt.log_output_dest.clone());
    log_builder.format(opt.log_format.unwrap_or(LogFormat::Default));
    log_builder.print_updates_to_stdout(!opt.json);
    let guards = log_builder.initialize()?;
    Ok(guards)
}
