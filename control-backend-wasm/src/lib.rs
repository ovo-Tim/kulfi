#![allow(clippy::derive_partial_eq_without_eq, clippy::get_first)]
#![deny(unused_crate_dependencies)]
#![warn(clippy::used_underscore_binding)]
#![forbid(unsafe_code)]

extern crate self as ftnet_backend;

#[ft_sdk::data]
fn ping() -> ft_sdk::data::Result {
    ft_sdk::data::json(serde_json::json!({ "message": "pong" }))
}

#[ft_sdk::data]
fn v1_what_to_do(
    // node-id for which we have to decide
    _id: ft_sdk::Query<"id">,
) -> ft_sdk::data::Result {
    // TODO: find device id from db and decide if we can do local ProxyPass or ForwardToPeer

    let res = common::WhatToDo::UnknownPeer;

    ft_sdk::data::json(res)
}
