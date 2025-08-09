pub fn mkdir(parent: &std::path::Path, name: &str) -> eyre::Result<std::path::PathBuf> {
    use eyre::WrapErr;
    let path = parent.join(name);

    std::fs::create_dir_all(&path)
        .wrap_err_with(|| format!("failed to create {name}: {path:?}"))?;
    Ok(path)
}

// Deprecated: Use kulfi_id52::PublicKey::from_str instead
pub fn id52_to_public_key(id: &str) -> eyre::Result<kulfi_id52::PublicKey> {
    use std::str::FromStr;
    kulfi_id52::PublicKey::from_str(id)
        .map_err(|e| eyre::anyhow!("{}", e))
}

// Deprecated: Use kulfi_id52::PublicKey::to_string instead
pub fn public_key_to_id52(key: &kulfi_id52::PublicKey) -> String {
    key.to_string()
}
