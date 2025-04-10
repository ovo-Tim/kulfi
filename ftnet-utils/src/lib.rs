extern crate self as ftnet_utils;

mod http_connection;
mod secret;
mod utils;

pub mod connection;
pub mod get_endpoint;
pub mod http;
mod http_to_peer;
mod peer_to_http;
pub mod protocol;

#[cfg(feature = "keyring")]
pub use secret::KeyringSecretStore;

pub use connection::{IDMap, PeerConnections};
pub use get_endpoint::get_endpoint;
pub use http::ProxyResult;
pub use http_connection::{ConnectionManager, ConnectionPool, ConnectionPools};
pub use http_to_peer::http_to_peer;
pub use peer_to_http::peer_to_http;
pub use protocol::{APNS_IDENTITY, Protocol};
pub use secret::SecretStore;
pub use utils::{FrameReader, frame_reader, id52_to_public_key, public_key_to_id52};
