// mod bb8;
mod create;
mod read;
mod run;

// pub use bb8::{PeerIdentity, get_endpoint};

#[derive(Debug)]
pub struct Identity {
    pub id52: String,
    pub public_key: iroh::PublicKey,
    pub client_pools: kulfi_utils::HttpConnectionPools,
}

impl Identity {
    pub fn from_id52(
        id: &str,
        client_pools: kulfi_utils::HttpConnectionPools,
    ) -> eyre::Result<Self> {
        let public_key = kulfi_utils::id52_to_public_key(id)?;
        Ok(Self {
            id52: kulfi_utils::public_key_to_id52(&public_key),
            public_key,
            client_pools,
        })
    }
}
