//! The identity folder
//!
//! The identity folder is stored in $FTN/identities/<identity-id>.
//!
//! The identity-id is the public key of the identity, it is a 64 character long string.
//!
//! The private key is stored in the platform specific keychain, and the public key is used as the
//! identity-id.
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
    pub async fn create(identities_folder: &std::path::Path) -> eyre::Result<Self> {
        use eyre::WrapErr;

        let public_key = {
            let mut rng = rand::rngs::OsRng;
            let secret_key = iroh::SecretKey::generate(&mut rng);
            // we do not want to keep secret key in memory, only in keychain
            ftn::utils::save_secret(&secret_key)
                .wrap_err("failed to store secret key to keychain")?;
            secret_key.public()
        };

        let now = std::time::SystemTime::now();
        let unixtime = now
            .duration_since(std::time::UNIX_EPOCH)
            .wrap_err("failed to get unix time")?
            .as_secs();
        let tmp_dir = identities_folder.join(format!("temp-{public_key}-{unixtime}"));

        ftn::utils::mkdir(&tmp_dir, "package")?;
        ftn::utils::mkdir(&tmp_dir, "devices")?;
        ftn::utils::mkdir(&tmp_dir, "logs")?;

        let dir = identities_folder.join(public_key.to_string());
        std::fs::rename(&tmp_dir, dir).wrap_err("failed to rename tmp_dir to dir")?;

        Ok(Self {
            id: public_key.to_string(),
            fastn_port: 8000,
        })
    }
}
