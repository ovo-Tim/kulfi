extern crate self as kulfi_utils;

pub mod keys;

pub mod dot_kulfi;
mod graceful;
pub mod http;
mod http_connection_manager;
pub mod protocol;
mod secret;
mod utils;

pub use graceful::Graceful;
pub use http::ProxyResult;
pub use http_connection_manager::{HttpConnectionManager, HttpConnectionPool, HttpConnectionPools};
pub use protocol::{APNS_IDENTITY, Protocol, ProtocolHeader};
pub use secret::{
    SECRET_KEY_FILE, generate_and_save_key, generate_secret_key, get_secret_key, read_or_create_key,
};
pub use utils::mkdir;

// Re-export key types from the keys module
pub use keys::{PublicKey, SecretKey, Signature};

// Deprecated: These functions are kept for backward compatibility
// Use PublicKey::from_id52 and PublicKey::to_id52 instead
pub use utils::{id52_to_public_key, public_key_to_id52};
