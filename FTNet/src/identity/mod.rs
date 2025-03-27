mod bb8;
mod create;
mod read;
mod run;

pub use bb8::get_endpoint;

#[derive(Debug)]
pub struct Identity {
    pub id: String,
    pub public_key: iroh::PublicKey,
}
