/// UDP proxy over iroh bidirectional streams.
///
/// Since iroh provides reliable, ordered byte streams (like TCP), we need to frame UDP datagrams
/// to preserve their boundaries. We use a simple length-prefix protocol:
///   - 2 bytes (u16 big-endian): datagram length
///   - N bytes: datagram payload
///
/// The maximum UDP datagram size we support is 65535 bytes (u16::MAX).

/// Receive a local UDP datagram and forward it over the iroh stream (framed with length prefix).
pub async fn peer_to_udp(
    addr: &str,
    mut send: iroh::endpoint::SendStream,
    mut recv: iroh::endpoint::RecvStream,
) -> eyre::Result<()> {
    let socket = tokio::net::UdpSocket::bind("0.0.0.0:0").await?;
    socket.connect(addr).await?;

    let recv_task = {
        let socket = std::sync::Arc::new(socket);
        let socket_for_send = socket.clone();

        // iroh stream -> local UDP socket
        let t = tokio::spawn(async move {
            loop {
                match read_framed_datagram(&mut recv).await {
                    Ok(data) => {
                        if let Err(e) = socket_for_send.send(&data).await {
                            tracing::error!("failed to send UDP datagram to local: {e:?}");
                            break;
                        }
                    }
                    Err(e) => {
                        tracing::trace!("iroh stream ended: {e:?}");
                        break;
                    }
                }
            }
        });

        // local UDP socket -> iroh stream
        let mut buf = vec![0u8; 65535];
        loop {
            match socket.recv(&mut buf).await {
                Ok(n) => {
                    if let Err(e) = write_framed_datagram(&mut send, &buf[..n]).await {
                        tracing::error!("failed to write framed datagram to iroh: {e:?}");
                        break;
                    }
                }
                Err(e) => {
                    tracing::error!("failed to recv from local UDP socket: {e:?}");
                    break;
                }
            }
        }

        send.finish()?;
        t
    };

    let _ = recv_task.await;
    Ok(())
}

/// Accept UDP datagrams on a local port and forward them over iroh to a remote peer.
pub async fn udp_to_peer(
    header: crate::ProtocolHeader,
    self_endpoint: iroh::Endpoint,
    socket: std::sync::Arc<tokio::net::UdpSocket>,
    client_addr: std::net::SocketAddr,
    data: Vec<u8>,
    remote_node_id52: &str,
    peer_connections: crate::PeerStreamSenders,
    graceful: crate::Graceful,
) -> eyre::Result<()> {
    tracing::info!("udp_to_peer: {remote_node_id52}");

    let (mut send, mut recv) = crate::get_stream(
        self_endpoint,
        header,
        remote_node_id52.to_string(),
        peer_connections,
        graceful,
    )
    .await?;

    tracing::info!("got stream for UDP");

    // Send the initial datagram
    write_framed_datagram(&mut send, &data).await?;

    // iroh stream -> local UDP socket (responses back to client)
    let socket_for_recv = socket.clone();
    let recv_task = tokio::spawn(async move {
        loop {
            match read_framed_datagram(&mut recv).await {
                Ok(data) => {
                    if let Err(e) = socket_for_recv.send_to(&data, client_addr).await {
                        tracing::error!("failed to send UDP response to client: {e:?}");
                        break;
                    }
                }
                Err(e) => {
                    tracing::trace!("iroh recv stream ended: {e:?}");
                    break;
                }
            }
        }
    });

    // local UDP socket -> iroh stream (subsequent datagrams from same client)
    // Note: this function handles the ongoing session for one client address.
    // The caller is responsible for routing subsequent datagrams from the same
    // client_addr to this session via a channel if needed.

    // For now, we just wait for the recv task to finish (the remote side closes the stream)
    let _ = recv_task.await;
    send.finish()?;

    Ok(())
}

/// Write a length-prefixed datagram to the iroh send stream.
pub async fn write_framed_datagram(
    send: &mut iroh::endpoint::SendStream,
    data: &[u8],
) -> eyre::Result<()> {
    let len = u16::try_from(data.len())
        .map_err(|_| eyre::anyhow!("datagram too large: {} bytes", data.len()))?;
    send.write_all(&len.to_be_bytes()).await?;
    send.write_all(data).await?;
    Ok(())
}

/// Read a length-prefixed datagram from the iroh recv stream.
pub async fn read_framed_datagram(recv: &mut iroh::endpoint::RecvStream) -> eyre::Result<Vec<u8>> {
    let mut len_buf = [0u8; 2];
    recv.read_exact(&mut len_buf)
        .await
        .map_err(|e| eyre::anyhow!("failed to read datagram length: {e}"))?;
    let len = u16::from_be_bytes(len_buf) as usize;
    let mut buf = vec![0u8; len];
    recv.read_exact(&mut buf)
        .await
        .map_err(|e| eyre::anyhow!("failed to read datagram body: {e}"))?;
    Ok(buf)
}
