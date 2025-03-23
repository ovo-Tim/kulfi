mod bb8;
mod create;
mod read;
mod run;

#[derive(Debug)]
pub struct Identity {
    pub public_key: iroh::PublicKey,
}
