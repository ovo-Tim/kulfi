use crate::utils;
use crate::{IDMap, PeerConnections, Protocol};

pub async fn peer_to_peer(
    req: hyper::Request<hyper::body::Incoming>,
    self_id52: &str,
    remote_node_id52: &str,
    peer_connections: PeerConnections,
    _patch: ftnet_sdk::RequestPatch,
    id_map: IDMap,
) -> crate::http::ProxyResult {
    use http_body_util::BodyExt;
    use tokio_stream::StreamExt;

    tracing::info!("peer_proxy: {remote_node_id52}");

    let (mut send, recv) = get_stream(
        self_id52,
        remote_node_id52,
        peer_connections.clone(),
        id_map,
    )
    .await?;

    tracing::info!("got stream");
    send.write_all(&serde_json::to_vec(&Protocol::Identity)?)
        .await?;
    send.write(b"\n").await?;

    tracing::info!("wrote protocol");

    let (head, body) = req.into_parts();
    send.write_all(&serde_json::to_vec(&crate::http::Request::from(head))?)
        .await?;
    send.write_all("\n".as_bytes()).await?;
    tracing::info!("sent request header");

    let mut body = http_body_util::BodyDataStream::new(body);
    while let Some(v) = body.next().await {
        send.write_all(&v?).await?;
    }

    tracing::info!("sent body");

    let mut recv = crate::utils::frame_reader(recv);
    let r: crate::http::Response = match recv.next().await {
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
            tracing::error!("failed to get bidirectional stream: {e:?}");
            Err(eyre::anyhow!("failed to get bidirectional stream: {e:?}"))
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

async fn get_stream(
    self_id52: &str,
    remote_node_id52: &str,
    peer_connections: PeerConnections,
    id_map: IDMap,
) -> eyre::Result<(iroh::endpoint::SendStream, iroh::endpoint::RecvStream)> {
    tracing::trace!("getting stream");
    let conn = get_connection(
        self_id52,
        remote_node_id52,
        id_map,
        peer_connections.clone(),
    )
    .await?;
    // TODO: this is where we can check if the connection is healthy or not. if we fail to get the
    //       bidirectional stream, probably we should try to recreate connection.
    tracing::trace!("getting stream - got connection");
    match conn.open_bi().await {
        Ok(v) => Ok(v),
        Err(e) => {
            tracing::trace!("get-stream forgetting connection: {e}");
            forget_connection(remote_node_id52, peer_connections).await?;
            tracing::error!("failed to get bidirectional stream: {e:?}");
            Err(eyre::anyhow!("failed to get bidirectional stream: {e:?}"))
        }
    }
}

async fn forget_connection(
    remote_node_id52: &str,
    peer_connections: PeerConnections,
) -> eyre::Result<()> {
    tracing::trace!("forgetting connection");
    let mut connections = peer_connections.lock().await;
    connections.remove(remote_node_id52);
    tracing::trace!("forgot connection");
    Ok(())
}

async fn get_connection(
    self_id52: &str,
    remote_node_id52: &str,
    id_map: IDMap,
    peer_connections: PeerConnections,
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

    tracing::trace!("getting ep");
    let ep = get_endpoint(self_id52, id_map).await?;
    tracing::trace!("got ep");

    let conn = match ep
        .connect(
            utils::id52_to_public_key(remote_node_id52)?,
            crate::APNS_IDENTITY,
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

async fn get_endpoint(self_id52: &str, id_map: IDMap) -> eyre::Result<iroh::endpoint::Endpoint> {
    let map = id_map.lock().await;

    for (id, (_port, ep)) in map.iter() {
        if id == self_id52 {
            return Ok(ep.clone());
        }
    }

    tracing::error!("no entry for {self_id52} in the id_map: {id_map:?}");
    Err(eyre::anyhow!(
        "no entry for {self_id52} in the id_map: {id_map:?}"
    ))
}
