/// this is the tcp proxy.
///
/// the other side has indicated they want to access our TCP device, whose id is specified in the
/// protocol header. we will first check if the remote id is allowed to do that, but the permission
/// system is managed not by Rust code of kulfi, but by the fastn server running as the identity
/// server. this allows fastn code to contain a lot of logic. since fastn code is sandboxed, and
/// something end user can easily modify or get from the fastn app marketplace ecosystem, it is a
/// good place to put as much logic as possible into fastn code.
///
/// fastn server will query database etc., will return the ip:port to connect to.
///
/// we have to decide if one tcp connection is one bidirectional stream as disused in protocol.rs.
/// so we will make one tcp connection from this function, and connect the `send` and `recv` streams
/// to tcp connection's `recv` and `send` side respectively.
pub async fn peer_to_tcp(
    _remote_id: &str,
    addr: &str,
    send: &mut iroh::endpoint::SendStream,
    recv: kulfi_utils::FrameReader,
) -> eyre::Result<()> {
    // todo: call identity server (fastn server running on behalf of identity
    //       /api/v1/identity/{id}/tcp/ with remote_id and id and get the ip:port
    //       to connect to.

    let stream = tokio::net::TcpStream::connect(addr).await?;
    pipe_tcp_stream_over_iroh(stream, send, recv).await
}

pub async fn pipe_tcp_stream_over_iroh(
    stream: tokio::net::TcpStream,
    send: &mut iroh::endpoint::SendStream,
    recv: kulfi_utils::FrameReader,
) -> eyre::Result<()> {
    use tokio::io::AsyncWriteExt;

    let (mut tcp_recv, tcp_send) = tokio::io::split(stream);

    let t = tokio::spawn(async move {
        let mut t = tcp_send;
        t.write_all(recv.read_buffer().as_ref()).await?;
        let mut recv = recv.into_inner();
        tokio::io::copy(&mut recv, &mut t).await
    });

    tokio::io::copy(&mut tcp_recv, send).await?;

    Ok(t.await?.map(|_| ())?)
}

pub async fn tcp_to_peer(
    self_endpoint: iroh::Endpoint,
    stream: tokio::net::TcpStream,
    remote_node_id52: &str,
    peer_connections: kulfi_utils::PeerStreamSenders,
    graceful: kulfi_utils::Graceful,
) -> eyre::Result<()> {
    tracing::info!("peer_proxy: {remote_node_id52}");

    let (mut send, recv) = kulfi_utils::get_stream(
        self_endpoint,
        kulfi_utils::Protocol::Tcp,
        remote_node_id52.to_string(),
        peer_connections.clone(),
        graceful,
    )
    .await?;

    tracing::info!("wrote protocol");

    kulfi_utils::pipe_tcp_stream_over_iroh(stream, &mut send, recv).await
}
