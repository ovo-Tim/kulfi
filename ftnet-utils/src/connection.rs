/// PeerConnections stores the iroh connections for every peer.
///
/// when a connection is broken, etc., we remove the connection from the map.
pub type PeerConnections = std::sync::Arc<
    tokio::sync::Mutex<std::collections::HashMap<String, iroh::endpoint::Connection>>,
>;

/// IDMap stores the fastn port and the endpoint for every identity
///
/// why is it a Vec and not a HasMap? the incoming requests contain the first few characters of id
/// and not the full id. the reason for this is we want to use <id>.localhost.direct as well, and
/// subdomain can be max 63 char long, and our ids are 64 chars. if we use <id>.ftnet, then this
/// will not be a problem. we still do prefix match instead of exact match just to be sure.
///
/// since the number of identities will be small, a prefix match is probably going to be the same
/// speed as the hash map exact lookup.
pub type IDMap = std::sync::Arc<tokio::sync::Mutex<Vec<(String, (u16, iroh::endpoint::Endpoint))>>>;
