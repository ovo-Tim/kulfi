pub async fn tcp_to_peer(
    self_endpoint: iroh::Endpoint,
    _stream: tokio::net::TcpStream,
    remote_node_id52: &str,
    peer_connections: kulfi_utils::PeerStreamSenders,
    graceful: kulfi_utils::Graceful,
) -> eyre::Result<()> {
    tracing::info!("peer_proxy: {remote_node_id52}");

    let (_send, _recv) = kulfi_utils::get_stream(
        self_endpoint,
        kulfi_utils::Protocol::Http,
        remote_node_id52.to_string(),
        peer_connections.clone(),
        graceful,
    )
    .await?;

    tracing::info!("wrote protocol");

    todo!()
}
