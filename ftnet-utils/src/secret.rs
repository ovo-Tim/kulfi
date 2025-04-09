use eyre::WrapErr;

/// A simple secret store to manage [iroh::SecretKey]
pub trait SecretStore {
    /// Get the secret key from the underlying store
    fn get(&self) -> eyre::Result<iroh::SecretKey>;
    /// Save the secret key to the underlying store
    fn save(&self, secret_key: &iroh::SecretKey) -> eyre::Result<()>;
    /// Generate a new secret key and save it to the underlying store
    /// returns the public key portion
    fn generate(rng: impl rand_core::CryptoRngCore) -> eyre::Result<iroh::PublicKey>;
}

#[cfg(feature = "keyring")]
pub struct KeyringSecretStore {
    id52: String,
}

#[cfg(feature = "keyring")]
impl SecretStore for KeyringSecretStore {
    fn get(&self) -> eyre::Result<iroh::SecretKey> {
        let entry = self.keyring_entry()?;

        let secret = entry
            .get_secret()
            .wrap_err_with(|| format!("keyring: secret not found for {}", self.id52))?;

        if secret.len() != 32 {
            return Err(eyre::anyhow!(
                "keyring: secret has invalid length: {}",
                secret.len()
            ));
        }

        let bytes: [u8; 32] = secret.try_into().expect("already checked for length");
        Ok(iroh::SecretKey::from_bytes(&bytes))
    }

    fn save(&self, secret_key: &iroh::SecretKey) -> eyre::Result<()> {
        Ok(self.keyring_entry()?.set_secret(&secret_key.to_bytes())?)
    }

    fn generate(mut rng: impl rand_core::CryptoRngCore) -> eyre::Result<iroh::PublicKey> {
        let secret_key = iroh::SecretKey::generate(&mut rng);
        // we do not want to keep secret key in memory, only in keychain
        let store = Self::new(ftnet_utils::public_key_to_id52(&secret_key.public()));
        store
            .save(&secret_key)
            .wrap_err_with(|| "failed to store secret key to keychain")?;

        Ok(secret_key.public())
    }
}

#[cfg(feature = "keyring")]
impl KeyringSecretStore {
    pub fn new(id52: String) -> Self {
        KeyringSecretStore { id52 }
    }

    fn keyring_entry(&self) -> eyre::Result<keyring::Entry> {
        keyring::Entry::new("FTNet", self.id52.as_str())
            .wrap_err_with(|| format!("failed to create keyring Entry for {}", self.id52))
    }
}
