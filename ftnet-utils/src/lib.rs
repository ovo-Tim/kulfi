extern crate self as ftnet_utils;

pub mod proxy;

pub mod connection;
pub mod get_endpoint;
pub mod http;
mod http_connection;
pub mod http_peer_proxy;
pub mod protocol;
mod utils;

pub use connection::{IDMap, PeerConnections};
pub use get_endpoint::get_endpoint;
pub use http::ProxyResult;
pub use http_connection::{ConnectionManager, ConnectionPool, ConnectionPools};
pub use protocol::{APNS_IDENTITY, Protocol};
pub use utils::{FrameReader, frame_reader, id52_to_public_key, public_key_to_id52};

use eyre::WrapErr;

// TODO: convert it to use id52 (we will store id52 in keyring)
fn keyring_entry(id: &str) -> eyre::Result<keyring::Entry> {
    keyring::Entry::new("FTNet", id)
        .wrap_err_with(|| format!("failed to create keyring Entry for {id}"))
}

// TODO: convert it to use id52 (we will store id52 in keyring)
pub fn save_secret(secret_key: &iroh::SecretKey) -> eyre::Result<()> {
    let public = secret_key.public().to_string();
    Ok(keyring_entry(public.as_str())?.set_secret(&secret_key.to_bytes())?)
}

// TODO: convert it to use id52 (we will store id52 in keyring)
pub fn get_secret(id: &str) -> eyre::Result<iroh::SecretKey> {
    let entry = keyring_entry(id)?;
    let secret = entry
        .get_secret()
        .wrap_err_with(|| format!("keyring: secret not found for {id}"))?;

    if secret.len() != 32 {
        return Err(eyre::anyhow!(
            "keyring: secret has invalid length: {}",
            secret.len()
        ));
    }

    let bytes: [u8; 32] = secret.try_into().unwrap(); // unwrap ok as already asserted
    Ok(iroh::SecretKey::from_bytes(&bytes))
}

pub fn create_public_key() -> eyre::Result<iroh::PublicKey> {
    let mut rng = rand::rngs::OsRng;
    let secret_key = iroh::SecretKey::generate(&mut rng);
    // we do not want to keep secret key in memory, only in keychain
    save_secret(&secret_key).wrap_err_with(|| "failed to store secret key to keychain")?;
    Ok(secret_key.public())
}

pub fn create_secret_key() -> iroh::SecretKey {
    let mut rng = rand::rngs::OsRng;
    iroh::SecretKey::generate(&mut rng)
}
