pub const PONG: &[u8] = b"pong\n";

pub async fn ping(conn: &iroh::endpoint::Connection) -> eyre::Result<()> {
    let (mut send_stream, mut recv_stream) = conn.open_bi().await?;
    send_stream
        .write_all(&serde_json::to_vec(&ftnet::Protocol::Ping)?)
        .await?;
    println!("finished stream, waiting for reply");
    let msg = recv_stream
        .read_to_end(10)
        .await
        .inspect_err(|e| eprintln!("failed to read: {e}"))?;
    println!("got {msg:?}, {PONG:?}");
    if msg != PONG {
        return Err(eyre::anyhow!("expected {PONG:?}, got {msg:?}"));
    }
    println!("sent ping, finishing stream");
    send_stream.finish()?;
    Ok(())
}
