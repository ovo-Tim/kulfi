#![allow(clippy::derive_partial_eq_without_eq, clippy::get_first)]
#![deny(unused_crate_dependencies)]
#![warn(clippy::used_underscore_binding)]
#![forbid(unsafe_code)]

extern crate self as ftnet_backend;

#[ft_sdk::data]
fn ping() -> ft_sdk::data::Result {
    ft_sdk::data::json(serde_json::json!({ "message": "pong" }))
}
