use crate::utils;
use crate::{IDMap, PeerConnections, Protocol};

pub async fn peer_to_peer(
    req: hyper::Request<hyper::body::Incoming>,
    self_id52: &str,
    remote_node_id52: &str,
    peer_connections: PeerConnections,
    _patch: ftnet_common::RequestPatch,
    id_map: IDMap,
) -> crate::http::ProxyResult {
    use http_body_util::BodyExt;
    use tokio_stream::StreamExt;

    tracing::info!("peer_proxy: {remote_node_id52}");

    let (mut send, recv) =
        get_stream(self_id52, remote_node_id52, peer_connections, id_map).await?;

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
        Some(v) => serde_json::from_str(&v?)?,
        None => {
            tracing::error!("failed to read from incoming connection");
            return Err(eyre::anyhow!("failed to read from incoming connection"));
        }
    };

    tracing::info!("got response header: {r:?}");

    let mut body = recv.read_buffer().to_vec();
    let mut recv = recv.into_inner();

    let mut buf = Vec::with_capacity(1024 * 64);

    while let Some(v) = recv.read(&mut buf).await? {
        if v == 0 {
            tracing::info!("finished reading body");
            break;
        }
        body.extend_from_slice(&buf);
        buf.truncate(0);
    }

    tracing::info!("got body");

    let mut res = hyper::Response::new(
        http_body_util::Full::new(hyper::body::Bytes::from(body))
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
    let conn = get_connection(self_id52, remote_node_id52, id_map, peer_connections).await?;
    // TODO: this is where we can check if the connection is healthy or not. if we fail to get the
    //       bidirectional stream, probably we should try to recreate connection.
    Ok(conn.open_bi().await?)
}

async fn get_connection(
    self_id52: &str,
    remote_node_id52: &str,
    id_map: IDMap,
    peer_connections: PeerConnections,
) -> eyre::Result<iroh::endpoint::Connection> {
    let connections = peer_connections.lock().await;
    let connection = connections.get(remote_node_id52).map(ToOwned::to_owned);

    // we drop the connections mutex guard so that we do not hold lock across await point.
    drop(connections);

    if let Some(conn) = connection {
        return Ok(conn);
    }

    let ep = get_endpoint(self_id52, id_map).await?;
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

    let mut connections = peer_connections.lock().await;
    connections.insert(remote_node_id52.to_string(), conn.clone());

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
