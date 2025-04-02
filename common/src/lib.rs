/// our fastn identity service can tell us to modify the request in some ways
/// TODO: make this smallvec to reduce heap allocations
pub type RequestPatch = Vec<RequestPatchItem>;

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub enum RequestPatchItem {
    AddHeader { name: String, value: String },
    DeleteHeader { name: String },
    AddCookie { name: String, value: String },
    DeleteCookie { name: String },
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub enum WhatToDo {
    ForwardToPeer {
        peer_id: String,
        patch: RequestPatch,
    },
    ProxyPass {
        port: u16,
        extra_headers: RequestPatch,
    },
    UnknownPeer,
}
