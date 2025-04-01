pub async fn peer_proxy(
    _requesting_id: &str,
    peer_id: &str,
    peer_connections: ftnet::identity::PeerConnections,
    client_pools: ftnet::http::client::ConnectionPools,
    _patch: ftnet::http::RequestPatch,
) -> ftnet::http::Result {
    let (_send, _recv) = get_stream(peer_id, peer_connections, client_pools).await?;

    todo!()
}

async fn get_stream(
    peer_id: &str,
    peer_connections: ftnet::identity::PeerConnections,
    client_pools: ftnet::http::client::ConnectionPools,
) -> eyre::Result<(iroh::endpoint::SendStream, iroh::endpoint::RecvStream)> {
    let mut peers = peer_connections.lock().await;

    let pool = match peers.get(peer_id) {
        Some(v) => v.clone(),
        None => {
            let pool = bb8::Pool::builder()
                .build(ftnet::Identity::from_id52(peer_id, client_pools)?)
                .await?;

            peers.insert(peer_id.to_string(), pool.clone());
            pool
        }
    };

    Ok(pool
        .get()
        .await
        .map_err(|e| {
            eprintln!("failed to get connection: {e:?}");
            eyre::anyhow!("failed to get connection: {e:?}")
        })?
        .open_bi()
        .await?)
}
