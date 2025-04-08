// mod bb8;
mod create;
mod read;
mod run;

// pub use bb8::{PeerIdentity, get_endpoint};

#[derive(Debug)]
pub struct Identity {
    pub id52: String,
    pub public_key: iroh::PublicKey,
    pub client_pools: ftnet_utils::ConnectionPools,
}

impl Identity {
    pub fn from_id52(id: &str, client_pools: ftnet_utils::ConnectionPools) -> eyre::Result<Self> {
        let public_key = ftnet_utils::utils::id52_to_public_key(id)?;
        Ok(Self {
            id52: ftnet_utils::utils::public_key_to_id52(&public_key),
            public_key,
            client_pools,
        })
    }
}
