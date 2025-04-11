/// PeerConnections stores the iroh connections for every peer.
///
/// when a connection is broken, etc., we remove the connection from the map.
pub type PeerStreamSenders =
std::sync::Arc<tokio::sync::Mutex<std::collections::HashMap<RemoteID52, StreamRequestSender>>>;

type Stream = (iroh::endpoint::SendStream, ftnet_utils::FrameReader);
type StreamResult = eyre::Result<Stream>;
type ReplyChannel = tokio::sync::oneshot::Sender<StreamResult>;
#[expect(unused)]
type ReplyChannelReceiver = tokio::sync::oneshot::Receiver<StreamResult>;
type RemoteID52 = String;

type StreamRequest = (
    iroh::Endpoint,
    ftnet_utils::Protocol,
    ReplyChannel,
);

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
    remote_node_id52: &RemoteID52,
    peer_stream_senders: PeerStreamSenders,
) -> eyre::Result<(iroh::endpoint::SendStream, ftnet_utils::FrameReader)> {
    let stream_request_sender =
        get_stream_request_sender(remote_node_id52, peer_stream_senders.clone()).await?;

    let (reply_channel, receiver) = tokio::sync::oneshot::channel();

    stream_request_sender
        .send((
            self_endpoint,
            protocol,
            reply_channel,
        ))
        .await?;

    receiver.await?
}

async fn get_stream_request_sender(
    remote_node_id52: &RemoteID52,
    peer_stream_senders: PeerStreamSenders,
) -> eyre::Result<StreamRequestSender> {
    let mut peer_stream_senders = peer_stream_senders.lock().await;

    if let Some(sender) = peer_stream_senders.get(remote_node_id52) {
        return Ok(sender.clone());
    }

    let (sender, receiver) = tokio::sync::mpsc::channel(1);
    peer_stream_senders.insert(remote_node_id52.clone(), sender.clone());

    let remote_node_id52 = remote_node_id52.to_owned();

    tokio::spawn(async move { connection_manager_worker(receiver, remote_node_id52).await });

    Ok(sender)
}

async fn connection_manager_worker(_receiver: StreamRequestReceiver, _remote_node_id52: RemoteID52) {
    todo!()
}
