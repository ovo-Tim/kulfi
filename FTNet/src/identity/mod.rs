mod bb8;
mod create;
mod read;
mod run;

pub use bb8::get_endpoint;

#[derive(Debug)]
pub struct Identity {
    pub id: String,
    pub public_key: iroh::PublicKey,
    pub client_pools: ftnet::http::client::ConnectionPools,
}

/// IDMap stores the fastn port for every identity
///
/// why is it a Vec and not a HasMap? the incoming requests contain the first few characters of id
/// and not the full id. the reason for this is we want to use <id>.localhost.direct as well, and
/// subdomain can be max 63 char long, and our ids are 64 chars. if we use <id>.ftnet, then this
/// will not be a problem. we still do prefix match instead of exact match just to be sure.
///
/// since the number of identities will be small, a prefix match is probably going to be the same
/// speed as the hash map exact lookup.
pub type IDMap = std::sync::Arc<tokio::sync::Mutex<Vec<(String, u16)>>>;
pub type PeerConnections =
    std::sync::Arc<tokio::sync::Mutex<std::collections::HashMap<String, ::bb8::Pool<Identity>>>>;
