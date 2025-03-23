//! The identity folder
//!
//! The identity folder is stored in $FTN/identities/<identity-id>.
//!
//! The identity-id is the public key of the identity, it is a 64 character long string.
//!
//! The folder contains a file named `private.key`, which contains the private key. The private
//! key's corresponding public key is the identity-id. The file is stored in PEM format, but we are
//! planning to store in Apple KeyChain or Windows Credential Manager in the future.
//!
//! This folder contains the `db.sqlite` file which corresponds to the DB for the fastn package for
//! this identity.
//!
//! `package` is the folder that contains the fastn package for this identity.
//!
//! `devices` is the folder that contains the device drivers for this identity. The structure of
//! this folder is described in `device/read.rs` (TODO).
//!
//! `logs` is the folder that contains the logs for this identity. This contains fastn access logs
//! and other device access logs etc.
impl ftn::Identity {
    pub async fn read(_path: &std::path::Path) -> eyre::Result<Self> {
        todo!()
    }
}
