pub async fn peer_proxy(
    _requesting_id: &str,
    peer_id: &str,
    peer_connections: ftnet::identity::PeerConnections,
    client_pools: ftnet::http::client::ConnectionPools,
    _patch: ftnet::http::RequestPatch,
) -> ftnet::http::Result {
    let _conn = get_connection(peer_id, peer_connections, client_pools).await?;

    todo!()
}

async fn get_connection(
    peer_id: &str,
    peer_connections: ftnet::identity::PeerConnections,
    client_pools: ftnet::http::client::ConnectionPools,
) -> eyre::Result<bb8::Pool<ftnet::Identity>> {
    let mut peers = peer_connections.lock().await;

    match peers.get(peer_id) {
        Some(v) => Ok(v.clone()),
        None => {
            let pool = bb8::Pool::builder()
                .build(ftnet::Identity::from_id52(peer_id, client_pools)?)
                .await?;

            peers.insert(peer_id.to_string(), pool.clone());
            Ok(pool)
        }
    }
}
