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

pub fn read_newline_separated_json<T: serde::de::DeserializeOwned>(
    msg: &[u8],
) -> eyre::Result<(T, &[u8])> {
    let mut i = 0;
    for (j, &b) in msg.iter().enumerate() {
        if b == b'\n' {
            i = j;
        }
    }

    if i == 0 {
        return Err(eyre::eyre!("no newline found in the message: {msg:?}"));
    }

    let header = &msg[..i];
    let rest = &msg[i + 1..];

    Ok((serde_json::from_slice(header)?, rest))
}
