pub fn mkdir(parent: &std::path::Path, name: &str) -> eyre::Result<std::path::PathBuf> {
    use eyre::WrapErr;
    let path = parent.join(name);

    std::fs::create_dir_all(&path)
        .wrap_err_with(|| format!("failed to create {name}: {path:?}"))?;
    Ok(path)
}

// Deprecated: Use PublicKey::from_str instead
pub fn id52_to_public_key(id: &str) -> eyre::Result<crate::PublicKey> {
    use std::str::FromStr;
    crate::PublicKey::from_str(id)
}

// Deprecated: Use PublicKey::to_string instead
pub fn public_key_to_id52(key: &crate::PublicKey) -> String {
    key.to_string()
}
