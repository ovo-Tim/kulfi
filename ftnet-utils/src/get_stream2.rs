/// PeerConnections stores the iroh connections for every peer.
///
/// when a connection is broken, etc., we remove the connection from the map.
#[expect(unused)]
pub type PeerStreamSenders =
std::sync::Arc<tokio::sync::Mutex<std::collections::HashMap<String, StreamRequestSender>>>;

type Stream = (iroh::endpoint::SendStream, ftnet_utils::FrameReader);
type StreamResult = eyre::Result<Stream>;
#[expect(unused)]
type ReplyChannel = tokio::sync::oneshot::Sender<StreamResult>;
type ReplyChannelReceiver = tokio::sync::oneshot::Receiver<StreamResult>;
type RemoteID52 = String;

type StreamRequest = (
    iroh::Endpoint,
    RemoteID52,
    ftnet_utils::Protocol,
    ReplyChannelReceiver,
);

type StreamRequestSender = tokio::sync::mpsc::Sender<StreamRequest>;
#[expect(unused)]
type StreamRequestReceiver = tokio::sync::mpsc::Receiver<StreamRequest>;

/// get_stream takes the protocol as well, as every outgoing bi-direction stream must have a
/// protocol. get_stream tries to check if the bidirectional stream is healthy, as simply opening
/// a bidirectional stream, or even simply writing on it does not guarantee that the stream is
/// open. only the read request times out to tell us something is wrong.
///
/// so solve this, we send a protocol message on the stream, and wait for an acknowledgement. if we
/// do not get the ack almost right away on a connection that we got from the cache, we assume the
/// connection is not healthy, and we try to recreate the connection. if it is a fresh connection,
/// then we use a longer timeout.
#[expect(unused)]
pub async fn get_stream(
    _self_endpoint: iroh::Endpoint,
    _protocol: ftnet_utils::Protocol,
    _remote_node_id52: &str,
    _peer_connections: ftnet_utils::PeerStreamSenders,
) -> eyre::Result<(iroh::endpoint::SendStream, ftnet_utils::FrameReader)> {
    todo!()
}
