use eyre::WrapErr;

pub fn mkdir(parent: &std::path::Path, name: &str) -> eyre::Result<std::path::PathBuf> {
    let path = parent.join(name);

    std::fs::create_dir_all(&path)
        .wrap_err_with(|| format!("failed to create {name}: {path:?}"))?;
    Ok(path)
}

fn keyring_entry(id: &str) -> eyre::Result<keyring::Entry> {
    keyring::Entry::new("FTNet", id)
        .wrap_err_with(|| format!("failed to create keyring Entry for {id}"))
}

pub fn save_secret(secret_key: &iroh::SecretKey) -> eyre::Result<()> {
    let public = secret_key.public().to_string();
    Ok(keyring_entry(public.as_str())?.set_secret(&secret_key.to_bytes())?)
}

pub fn get_secret(id: &str) -> eyre::Result<iroh::SecretKey> {
    let entry = keyring_entry(id)?;
    let secret = entry
        .get_secret()
        .wrap_err_with(|| format!("keyring: secret not found for {id}"))?;

    if secret.len() != 32 {
        return Err(eyre::anyhow!(
            "keyring: secret has invalid length: {}",
            secret.len()
        ));
    }

    let bytes: [u8; 32] = secret.try_into().unwrap(); // unwrap ok as already asserted
    Ok(iroh::SecretKey::from_bytes(&bytes))
}

pub type FrameReader =
    tokio_util::codec::FramedRead<iroh::endpoint::RecvStream, tokio_util::codec::LinesCodec>;

pub fn frame_reader(recv: iroh::endpoint::RecvStream) -> FrameReader {
    tokio_util::codec::FramedRead::new(recv, tokio_util::codec::LinesCodec::new())
}

pub fn id52_to_public_key(id: &str) -> eyre::Result<iroh::PublicKey> {
    use eyre::WrapErr;

    let bytes = data_encoding::BASE32_DNSSEC.decode(id.as_bytes())?;
    if bytes.len() != 32 {
        return Err(eyre::anyhow!(
            "read: id has invalid length: {}",
            bytes.len()
        ));
    }

    let bytes: [u8; 32] = bytes.try_into().unwrap(); // unwrap ok as already asserted

    iroh::PublicKey::from_bytes(&bytes).wrap_err_with(|| "failed to parse id to public key")
}

pub fn public_key_to_id52(key: &iroh::PublicKey) -> String {
    data_encoding::BASE32_DNSSEC.encode(key.as_bytes())
}
