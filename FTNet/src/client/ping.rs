pub const PING: &[u8] = b"ping\n";
pub const PONG: &[u8] = b"pong\n";

pub async fn ping(conn: &iroh::endpoint::Connection) -> eyre::Result<()> {
    let (mut send_stream, mut recv_stream) = conn.open_bi().await?;
    send_stream.write_all(PING).await?;
    send_stream.finish()?;
    let msg = recv_stream.read_to_end(10).await?;
    if msg != PONG {
        return Err(eyre::anyhow!("expected {PONG:?}, got {msg:?}"));
    }
    Ok(())
}
