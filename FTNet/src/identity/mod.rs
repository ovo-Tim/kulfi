mod bb8;
mod create;
mod read;
mod run;

pub use bb8::get_endpoint;

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
        use eyre::WrapErr;

        let bytes = data_encoding::BASE32_DNSSEC.decode(id.as_bytes())?;
        if bytes.len() != 32 {
            return Err(eyre::anyhow!(
                "read: id has invalid length: {}",
                bytes.len()
            ));
        }

        let bytes: [u8; 32] = bytes.try_into().unwrap(); // unwrap ok as already asserted

        let public_key: iroh::PublicKey = iroh::PublicKey::from_bytes(&bytes)
            .wrap_err_with(|| "failed to parse id to public key")?;

        Ok(Self {
            id52: data_encoding::BASE32_DNSSEC.encode(public_key.as_bytes()),
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
pub type PeerConnections =
    std::sync::Arc<tokio::sync::Mutex<std::collections::HashMap<String, ::bb8::Pool<Identity>>>>;
