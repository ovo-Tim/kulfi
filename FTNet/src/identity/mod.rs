// mod bb8;
mod create;
mod read;
mod run;

// pub use bb8::{PeerIdentity, get_endpoint};

#[derive(Debug)]
pub struct Identity {
    pub id52: String,
    pub public_key: iroh::PublicKey,
    pub client_pools: ftnet::http::client::ConnectionPools,
}

impl Identity {
    pub fn from_id52(
        id: &str,
        client_pools: ftnet::http::client::ConnectionPools,
    ) -> eyre::Result<Self> {
        let public_key = ftnet::utils::id52_to_public_key(id)?;
        Ok(Self {
            id52: ftnet::utils::public_key_to_id52(&public_key),
            public_key,
            client_pools,
        })
    }
}

/// IDMap stores the fastn port and the endpoint for every identity
///
/// why is it a Vec and not a HasMap? the incoming requests contain the first few characters of id
/// and not the full id. the reason for this is we want to use <id>.localhost.direct as well, and
/// subdomain can be max 63 char long, and our ids are 64 chars. if we use <id>.ftnet, then this
/// will not be a problem. we still do prefix match instead of exact match just to be sure.
///
/// since the number of identities will be small, a prefix match is probably going to be the same
/// speed as the hash map exact lookup.
pub type IDMap = std::sync::Arc<tokio::sync::Mutex<Vec<(String, (u16, iroh::endpoint::Endpoint))>>>;

/// PeerConnections stores the iroh connections for every peer.
///
/// when a connection is broken, etc., we remove the connection from the map.
pub type PeerConnections = std::sync::Arc<
    tokio::sync::Mutex<std::collections::HashMap<String, iroh::endpoint::Connection>>,
>;
