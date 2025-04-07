mod bb8;
mod create;
mod read;
mod run;

pub use bb8::{get_endpoint, PeerIdentity};

#[derive(Debug)]
pub struct Identity {
    pub id52: String,
    pub public_key: iroh::PublicKey,
    pub client_pools: ftnet::http::client::ConnectionPools,
}


impl Identity {
    pub fn peer_identity(&self, fastn_port: u16, peer_id: &str) -> eyre::Result<PeerIdentity> {
        Ok(PeerIdentity {
            fastn_port,
            self_id52: self.id52.clone(),
            self_public_key: self.public_key,
            peer_public_key: ftnet::utils::id52_to_public_key(peer_id)?,
            client_pools: self.client_pools.clone(),
        })
    }

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
pub type PeerConnections = std::sync::Arc<
    tokio::sync::Mutex<std::collections::HashMap<String, ::bb8::Pool<PeerIdentity>>>,
>;
