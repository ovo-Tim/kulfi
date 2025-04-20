use tokio::io::AsyncWriteExt;

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
