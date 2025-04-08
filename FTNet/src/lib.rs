#![deny(unused_extern_crates)]
#![deny(unused_crate_dependencies)]
#![deny(unsafe_code)]

// TODO: Remove this and separate ftnet binary from library to get rid of unused_crate_dependencies
// lint check errors
// Only the binary is using the following crates:
use clap as _;
use directories as _;
use fastn_observer as _;
use tracing_subscriber as _;

extern crate self as ftnet;

mod client;
mod config;
pub mod control_server;
mod counters;
pub mod http;
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
