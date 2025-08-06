extern crate self as kulfi_iroh_utils;

pub mod get_endpoint;
mod get_stream;
mod http_to_peer;
mod peer_to_http;
mod ping;
mod tcp;
mod utils;

pub use get_endpoint::get_endpoint;
pub use get_stream::{PeerStreamSenders, get_stream};
pub use http_to_peer::{http_to_peer, http_to_peer_non_streaming};
pub use peer_to_http::peer_to_http;
pub use ping::{PONG, ping};
pub use tcp::{peer_to_tcp, pipe_tcp_stream_over_iroh, tcp_to_peer};
pub use utils::{
    accept_bi, accept_bi_with, get_remote_id52, global_iroh_endpoint, next_json, next_string,
};

/// IDMap stores the fastn port and the endpoint for every identity
///
/// why is it a Vec and not a HasMap? the incoming requests contain the first few characters of id
/// and not the full id. the reason for this is we want to use <id>.localhost.direct as well, and
/// subdomain can be max 63 char long, and our ids are 64 chars. if we use <id>.kulfi, then this
/// will not be a problem. we still do prefix match instead of exact match just to be sure.
///
/// since the number of identities will be small, a prefix match is probably going to be the same
/// speed as the hash map exact lookup.
pub type IDMap = std::sync::Arc<tokio::sync::Mutex<Vec<(String, (u16, iroh::endpoint::Endpoint))>>>;

pub const ACK: &str = "ack";
