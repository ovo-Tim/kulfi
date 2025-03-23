#![deny(unused_extern_crates)]
#![deny(unused_crate_dependencies)]
#![deny(unsafe_code)]

extern crate self as ftn;

#[expect(unused)]
#[expect(clippy::single_component_path_imports)]
use ftn_tcp_proxy;

mod cli;
mod config;
mod counters;
mod identity;
mod start;
pub mod utils;

pub use cli::{Cli, Command};
pub use config::Config;
pub use counters::OPEN_CONNECTION_COUNT;
pub use identity::Identity;
pub use start::start;
