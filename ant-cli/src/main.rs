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
mod opt;
mod utils;
mod wallet;

pub use access::data_dir;
pub use access::keys;
pub use access::network;
pub use access::user_data;

use clap::Parser;
use color_eyre::Result;

use ant_logging::metrics::init_metrics;
use ant_logging::{LogBuilder, LogFormat, LogOutputDest, ReloadHandle, WorkerGuard};
use ant_protocol::version;
use clap_verbosity_flag::log::LevelFilter;
use opt::Opt;
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
        println!(
            "{}",
            ant_build_info::version_string(
                "Autonomi Client",
                env!("CARGO_PKG_VERSION"),
                Some(&identify_protocol_str)
            )
        );
        return Ok(());
    }

    if opt.crate_version {
        println!("Crate version: {}", env!("CARGO_PKG_VERSION"));
        return Ok(());
    }

    if opt.protocol_version {
        println!("Network version: {identify_protocol_str}");
        return Ok(());
    }

    #[cfg(not(feature = "nightly"))]
    if opt.package_version {
        println!("Package version: {}", ant_build_info::package_version());
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
    let level = match opt.verbose.log_level_filter() {
        LevelFilter::Off => None,
        LevelFilter::Error => Some(Level::ERROR),
        LevelFilter::Warn => Some(Level::WARN),
        LevelFilter::Info => Some(Level::INFO),
        LevelFilter::Debug => Some(Level::DEBUG),
        LevelFilter::Trace => Some(Level::TRACE),
    };

    let targets = if let Some(level) = level {          
        vec![
            ("ant_evm".to_string(), level),
            ("autonomi_cli".to_string(), level),
            ("autonomi".to_string(), level),
            ("evmlib".to_string(), level),
        ]
    } else {
        vec![]
    };

    let mut log_builder = LogBuilder::new(targets);
    log_builder.output_dest(LogOutputDest::Stderr);
    log_builder.format(opt.log_format.unwrap_or(LogFormat::Default));
    let guards = log_builder.initialize()?;
    Ok(guards)
}
