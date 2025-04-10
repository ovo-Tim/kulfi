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
pub use protocol::{Protocol, APNS_IDENTITY};
pub use secret::SecretStore;
pub use utils::{frame_reader, id52_to_public_key, public_key_to_id52, FrameReader};


pub async fn get_remote_id52(conn: &iroh::endpoint::Connection) -> eyre::Result<String> {
    let remote_node_id = match conn.remote_node_id() {
        Ok(id) => id,
        Err(e) => {
            tracing::error!("could not read remote node id: {e}, closing connection");
            // TODO: is this how we close the connection in error cases or do we send some error
            //       and wait for other side to close the connection?
            let e2 = conn.closed().await;
            tracing::info!("connection closed: {e2}");
            // TODO: send another error_code to indicate bad remote node id?
            conn.close(0u8.into(), &[]);
            return Err(eyre::anyhow!("could not read remote node id: {e}"));
        }
    };

    Ok(public_key_to_id52(&remote_node_id))
}

const ACK: &str = "ack";

pub async fn ack(
    send: &mut iroh::endpoint::SendStream,
) -> eyre::Result<()> {
    send.write_all(format!("{}\n", ACK).as_bytes()).await?;
    Ok(())
}
