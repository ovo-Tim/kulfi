pub async fn ping(conn: &iroh::endpoint::Connection) -> eyre::Result<()> {
    let (mut send_stream, mut recv_stream) = conn.open_bi().await?;
    send_stream.write_all(b"ping\n").await?;
    send_stream.finish()?;
    let msg = recv_stream.read_to_end(10).await?;
    if msg != b"pong\n" {
        return Err(eyre::anyhow!("expected pong, got {msg:?}"));
    }
    Ok(())
}
