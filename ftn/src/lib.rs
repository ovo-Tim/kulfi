#![deny(unused_extern_crates)]
#![deny(unused_crate_dependencies)]
#![deny(unsafe_code)]

extern crate self as ftn;

#[expect(unused)]
#[expect(clippy::single_component_path_imports)]
use ftn_tcp_proxy;

mod cli;
mod client;
mod config;
mod counters;
mod identity;
mod server;
mod start;
pub mod utils;

pub use cli::{Cli, Command};
pub use config::Config;
pub use counters::OPEN_CONNECTION_COUNT;
pub use identity::Identity;
pub use start::start;

/// Iroh supports multiple protocols, and we do need multiple protocols, lets say one for proxying
/// TCP connection, another for proxying HTTP connection, and so on. But if we use different APNS
/// to handle them, we will end up creating more connections than minimally required (one connection
/// can only talk one APNS). So, we use a single APNS for all the protocols, and we use the first
/// line of the input to determine the protocol.
const APNS: &[u8] = b"ftn";
