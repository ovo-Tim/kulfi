pub const PONG: &[u8] = b"pong\n";

pub async fn ping(conn: &iroh::endpoint::Connection) -> eyre::Result<()> {
    let (mut send_stream, mut recv_stream) = conn.open_bi().await?;
    send_stream
        .write_all(&serde_json::to_vec(&ftnet::Protocol::Ping)?)
        .await?;
    send_stream.write_all("\n".as_bytes()).await?;
    tracing::info!("sent ping, waiting for reply");
    let msg = recv_stream
        .read_to_end(1000)
        .await
        .inspect_err(|e| tracing::error!("failed to read: {e}"))?;
    tracing::info!("got {msg:?}, {PONG:?}");
    if msg != PONG {
        return Err(eyre::anyhow!("expected {PONG:?}, got {msg:?}"));
    }
    tracing::info!("got reply, finishing stream");
    send_stream.finish()?;
    tracing::info!("finished stream");
    Ok(())
}
