/// PeerConnections stores the iroh connections for every peer.
///
/// when a connection is broken, etc., we remove the connection from the map.
pub type PeerStreamSenders = std::sync::Arc<
    tokio::sync::Mutex<std::collections::HashMap<(SelfID52, RemoteID52), StreamRequestSender>>,
>;

type Stream = (iroh::endpoint::SendStream, ftnet_utils::FrameReader);
type StreamResult = eyre::Result<Stream>;
type ReplyChannel = tokio::sync::oneshot::Sender<StreamResult>;
#[expect(unused)]
type ReplyChannelReceiver = tokio::sync::oneshot::Receiver<StreamResult>;
type RemoteID52 = String;
type SelfID52 = String;

type StreamRequest = (ftnet_utils::Protocol, ReplyChannel);

type StreamRequestSender = tokio::sync::mpsc::Sender<StreamRequest>;
type StreamRequestReceiver = tokio::sync::mpsc::Receiver<StreamRequest>;

/// get_stream tries to check if the bidirectional stream is healthy, as simply opening
/// a bidirectional stream, or even simply writing on it does not guarantee that the stream is
/// open. only the read request times out to tell us something is wrong. this is why get_stream
/// takes the protocol as well, as every outgoing bi-direction stream must have a protocol. it
/// sends the protocol and waits for an ack. if the ack is not received within a certain time, it
/// assumes the connection is not healthy, and tries to recreate the connection.
///
/// for managing connection, we use a spawned task. this task listens for incoming stream requests
/// and manages the connection as part of the task local data.
#[expect(unused)]
pub async fn get_stream(
    self_endpoint: iroh::Endpoint,
    protocol: ftnet_utils::Protocol,
    remote_node_id52: RemoteID52,
    peer_stream_senders: PeerStreamSenders,
) -> eyre::Result<(iroh::endpoint::SendStream, ftnet_utils::FrameReader)> {
    let stream_request_sender =
        get_stream_request_sender(self_endpoint, remote_node_id52, peer_stream_senders).await;

    let (reply_channel, receiver) = tokio::sync::oneshot::channel();

    stream_request_sender
        .send((protocol, reply_channel))
        .await?;

    receiver.await?
}

async fn get_stream_request_sender(
    self_endpoint: iroh::Endpoint,
    remote_node_id52: RemoteID52,
    peer_stream_senders: PeerStreamSenders,
) -> StreamRequestSender {
    let self_id52 = ftnet_utils::public_key_to_id52(&self_endpoint.node_id());
    let mut senders = peer_stream_senders.lock().await;

    if let Some(sender) = senders.get(&(self_id52.clone(), remote_node_id52.clone())) {
        return sender.clone();
    }

    // TODO: figure out if the mpsc::channel is the right size
    let (sender, receiver) = tokio::sync::mpsc::channel(1);
    senders.insert(
        (self_id52.clone(), remote_node_id52.clone()),
        sender.clone(),
    );
    drop(senders);

    tokio::spawn(async move {
        connection_manager(
            receiver,
            self_id52,
            self_endpoint,
            remote_node_id52,
            peer_stream_senders,
        )
        .await;
    });

    sender
}

async fn connection_manager(
    mut receiver: StreamRequestReceiver,
    self_id52: SelfID52,
    self_endpoint: iroh::Endpoint,
    remote_node_id52: RemoteID52,
    peer_stream_senders: PeerStreamSenders,
) {
    let e = match connection_manager_(&mut receiver, self_endpoint, remote_node_id52.clone()).await
    {
        Ok(()) => return,
        Err(e) => e,
    };

    // what is our error handling strategy?
    //
    // since an error has just occurred on our connection, it is best to cancel all concurrent
    // tasks that depend on this connection, and let the next task recreate the connection, this
    // way things are clean.
    //
    // we can try to keep the concurrent tasks open, and retry connection, but it increases the
    // complexity of implementation, and it is not worth it for now.
    //
    // also note that connection_manager() and it's caller, get_stream(), are called to create the
    // initial stream only, this error handling strategy will work for concurrent requests that are
    // waiting for the stream to be created. the tasks that already got the stream will not be
    // affected by this. tho, since something wrong has happened with the connection, they will
    // eventually fail too.
    tracing::error!("connection manager worker error: {e:?}");

    // once we close the receiver, any tasks that have gotten access to the corresponding sender
    // will fail when sending.
    receiver.close();

    // send an error to all the tasks that are waiting for stream for this receiver.
    while let Some((_protocol, reply_channel)) = receiver.recv().await {
        if reply_channel
            .send(Err(eyre::anyhow!("failed to create connection: {e:?}")))
            .is_err()
        {
            tracing::error!("failed to send error reply: {e:?}");
        }
    }

    // cleanup the peer_stream_senders map, so no future tasks will try to use this.
    let mut senders = peer_stream_senders.lock().await;
    senders.remove(&(self_id52.clone(), remote_node_id52.clone()));
}

async fn connection_manager_(
    receiver: &mut StreamRequestReceiver,
    self_endpoint: iroh::Endpoint,
    remote_node_id52: RemoteID52,
) -> eyre::Result<()> {
    let conn = match self_endpoint
        .connect(
            ftnet_utils::id52_to_public_key(&remote_node_id52)?,
            ftnet_utils::APNS_IDENTITY,
        )
        .await
    {
        Ok(v) => v,
        Err(e) => {
            tracing::error!("failed to create connection: {e:?}");
            return Err(eyre::anyhow!("failed to create connection: {e:?}"));
        }
    };

    // TODO: if we do not get any task on receiver, we should send ping pong for the keep alive
    //       duration. hint: use tokio::select!{} here
    while let Some((protocol, reply_channel)) = receiver.recv().await {
        handle_request(&conn, protocol, reply_channel).await?;
    }

    Ok(())
}

async fn handle_request(
    conn: &iroh::endpoint::Connection,
    _protocol: ftnet_utils::Protocol,
    _reply_channel: ReplyChannel,
) -> eyre::Result<()> {
    let (_send, _recv) = match conn.open_bi().await {
        Ok(v) => v,
        Err(e) => {
            tracing::error!("failed to open_bi: {e:?}");
            return Err(eyre::anyhow!("failed to open_bi: {e:?}"));
        }
    };

    todo!()
}
