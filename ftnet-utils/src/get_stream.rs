/// PeerConnections stores the iroh connections for every peer.
///
/// when a connection is broken, etc., we remove the connection from the map.
pub type PeerConnections = std::sync::Arc<
    tokio::sync::Mutex<std::collections::HashMap<String, iroh::endpoint::Connection>>,
>;

/// get_stream takes the protocol as well, as every outgoing bi-direction stream must have a
/// protocol. get_stream tries to check if the bidirectional stream is healthy, as simply opening
/// a bidirectional stream, or even simply writing on it does not guarantee that the stream is
/// open. only the read request times out to tell us something is wrong.
///
/// so solve this, we send a protocol message on the stream, and wait for an acknowledgement. if we
/// do not get the ack almost right away on a connection that we got from the cache, we assume the
/// connection is not healthy, and we try to recreate the connection. if it is a fresh connection,
/// then we use a longer timeout.
pub async fn get_stream(
    self_endpoint: iroh::Endpoint,
    protocol: ftnet_utils::Protocol,
    remote_node_id52: &str,
    peer_connections: ftnet_utils::PeerConnections,
) -> eyre::Result<(iroh::endpoint::SendStream, ftnet_utils::FrameReader)> {
    use tokio_stream::StreamExt;

    tracing::trace!("getting stream");
    let conn = get_connection(self_endpoint, remote_node_id52, peer_connections.clone()).await?;
    // TODO: this is where we can check if the connection is healthy or not. if we fail to get the
    //       bidirectional stream, probably we should try to recreate connection.
    tracing::trace!("getting stream - got connection");
    let (mut send, recv) = match conn.open_bi().await {
        Ok(v) => v,
        Err(e) => {
            tracing::trace!("get-stream forgetting connection: {e}");
            forget_connection(remote_node_id52, peer_connections).await?;
            tracing::error!("failed to get bidirectional stream: {e:?}");
            return Err(eyre::anyhow!("failed to get bidirectional stream: {e:?}"));
        }
    };

    tracing::info!("got stream");
    send.write_all(&serde_json::to_vec(&protocol)?).await?;
    send.write(b"\n").await?;

    let mut recv = ftnet_utils::frame_reader(recv);

    // TODO: use tokio::select!{} to implement timeout here, resilient
    match recv.next().await {
        Some(Ok(v)) => {
            if v != ftnet_utils::ACK {
                forget_connection(remote_node_id52, peer_connections).await?;
                eprintln!(
                    "got unexpected message: {v:?}, expected {}",
                    ftnet_utils::ACK
                );
                return Err(eyre::anyhow!("got unexpected message: {v:?}"));
            }
        }
        Some(Err(e)) => {
            forget_connection(remote_node_id52, peer_connections).await?;
            tracing::error!("failed to get bidirectional stream: {e:?}");
            return Err(eyre::anyhow!("failed to get bidirectional stream: {e:?}"));
        }
        None => {
            tracing::error!("failed to read from incoming connection");
            return Err(eyre::anyhow!("failed to read from incoming connection"));
        }
    }

    Ok((send, recv))
}

pub async fn forget_connection(
    remote_node_id52: &str,
    peer_connections: ftnet_utils::PeerConnections,
) -> eyre::Result<()> {
    tracing::trace!("forgetting connection");
    let mut connections = peer_connections.lock().await;
    connections.remove(remote_node_id52);
    tracing::trace!("forgot connection");
    Ok(())
}

async fn get_connection(
    self_endpoint: iroh::Endpoint,
    remote_node_id52: &str,
    peer_connections: ftnet_utils::PeerConnections,
) -> eyre::Result<iroh::endpoint::Connection> {
    tracing::trace!("getting connections lock");
    let connections = peer_connections.lock().await;
    tracing::trace!("got connections lock");
    let connection = connections.get(remote_node_id52).map(ToOwned::to_owned);

    // we drop the connections mutex guard so that we do not hold lock across await point.
    drop(connections);

    if let Some(conn) = connection {
        return Ok(conn);
    }

    let conn = match self_endpoint
        .connect(
            ftnet_utils::id52_to_public_key(remote_node_id52)?,
            ftnet_utils::APNS_IDENTITY,
        )
        .await
    {
        Ok(conn) => conn,
        Err(e) => {
            tracing::error!("failed to create connection: {e:?}");
            return Err(eyre::anyhow!("failed to create connection: {e:?}"));
        }
    };

    tracing::trace!("storing connection");
    let mut connections = peer_connections.lock().await;
    connections.insert(remote_node_id52.to_string(), conn.clone());
    tracing::trace!("stored connection");

    Ok(conn)
}
