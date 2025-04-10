pub async fn http_to_peer<T>(
    req: hyper::Request<T>,
    self_endpoint: iroh::Endpoint,
    remote_node_id52: &str,
    peer_connections: ftnet_utils::PeerConnections,
    _patch: ftnet_sdk::RequestPatch,
) -> ftnet_utils::http::ProxyResult
where
    T: hyper::body::Body + Unpin + Send,
    T::Data: Into<hyper::body::Bytes> + Send,
    T::Error: std::error::Error + Send + Sync + 'static,
{
    use http_body_util::{BodyDataStream, BodyExt};
    use tokio_stream::StreamExt;

    tracing::info!("peer_proxy: {remote_node_id52}");

    let (mut send, mut recv) =
        get_stream(self_endpoint, ftnet_utils::Protocol::Identity, remote_node_id52, peer_connections.clone()).await?;

    tracing::info!("wrote protocol");

    let (head, body) = req.into_parts();
    send.write_all(&serde_json::to_vec(&crate::http::Request::from(head))?)
        .await?;
    send.write_all(b"\n").await?;

    tracing::info!("sent request header");

    let mut stream = BodyDataStream::new(body);

    while let Some(chunk) = stream.next().await {
        let bytes: hyper::body::Bytes = chunk?.into(); // requires T::Data: Into<Bytes>
        send.write_all(&bytes).await?;
    }

    tracing::info!("sent body");

    let r: ftnet_utils::http::Response = match recv.next().await {
        Some(Ok(v)) => serde_json::from_str(&v)?,
        Some(Err(e)) => {
            forget_connection(remote_node_id52, peer_connections.clone()).await?;
            tracing::error!("failed to get bidirectional stream: {e:?}");
            return Err(eyre::anyhow!("failed to get bidirectional stream: {e:?}"));
        }
        None => {
            tracing::error!("failed to read from incoming connection");
            return Err(eyre::anyhow!("failed to read from incoming connection"));
        }
    };

    tracing::info!("got response header: {r:?}");

    let mut body = recv.read_buffer().to_owned();
    let mut recv = recv.into_inner();

    tracing::trace!("reading body");

    while let Some(v) = match recv.read_chunk(1024 * 64, true).await {
        Ok(v) => Ok(v),
        Err(e) => {
            forget_connection(remote_node_id52, peer_connections.clone()).await?;
            tracing::error!("error reading chunk: {e:?}");
            Err(eyre::anyhow!("read_chunk error: {e:?}"))
        }
    }? {
        body.extend_from_slice(&v.bytes);
        tracing::trace!(
            "reading body, partial: {}, new body size: {} bytes",
            v.bytes.len(),
            body.len()
        );
    }

    let body = body.freeze();
    tracing::debug!("got {} bytes of body", body.len());

    let mut res = hyper::Response::new(
        http_body_util::Full::new(body)
            .map_err(|e| match e {})
            .boxed(),
    );
    *res.status_mut() = hyper::http::StatusCode::from_u16(r.status)?;
    for (k, v) in r.headers {
        res.headers_mut().insert(
            hyper::http::header::HeaderName::from_bytes(k.as_bytes())?,
            hyper::http::header::HeaderValue::from_bytes(&v)?,
        );
    }

    tracing::info!("all done");
    Ok(res)
}

/// get_stream takes the protocol as well, as every outgoing bi-direction stream must have a
/// protocol. get_stream tries to check if the bidirectional stream is healthy, as simply opening
/// a bidirectional stream, or even simply writing on it does not guarantee that the stream is
/// open. only the read request times out to tell us something is wrong.
///
/// so solve this, we send a protocol message on the stream, and wait for an acknowledgement. if we
/// do not get the ack almost right away on a connection that we got from the cache, we assume the
/// connection is not healthy, and we try to recreate the connection. if it is a fresh connection,
/// then we use a longer timeout.
async fn get_stream(
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
    send.write_all(&serde_json::to_vec(&protocol)?)
        .await?;
    send.write(b"\n").await?;

    let mut recv = ftnet_utils::frame_reader(recv);

    // TODO: use tokio::select!{} to implement timeout here, resilient
    match recv.next().await {
        Some(Ok(v)) => {
            if v != ftnet_utils::ACK {
                forget_connection(remote_node_id52, peer_connections).await?;
                eprintln!("got unexpected message: {v:?}, expected {}", ftnet_utils::ACK);
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

async fn forget_connection(
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
