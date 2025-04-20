#![deny(unused_extern_crates)]
#![deny(unused_crate_dependencies)]
#![deny(unsafe_code)]

// TODO: Remove this and separate kulfi binary from library to get rid of unused_crate_dependencies
// lint check errors
// Only the binary is using the following crates:
use clap as _;
use clap_verbosity_flag as _;
use directories as _;
use fastn_observer as _;
use tauri as _;
use tauri_plugin_opener as _;
use tracing_subscriber as _;

extern crate self as kulfi;

mod client;
mod config;
pub mod control_server;
mod counters;
mod identity;
pub mod peer_server;
mod start;
pub mod utils;

pub use config::Config;
pub use counters::{
    CONTROL_CONNECTION_COUNT, CONTROL_REQUEST_COUNT, IN_FLIGHT_REQUESTS,
    OPEN_CONTROL_CONNECTION_COUNT,
};
// pub use identity::{Identity, PeerIdentity};
pub use identity::Identity;
pub use start::start;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn ui() -> eyre::Result<()> {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");

    Ok(())
}
