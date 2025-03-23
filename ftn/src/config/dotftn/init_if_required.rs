/// this function is called on startup, and initializes the .ftn directory if it doesn't exist
pub async fn init_if_required(dir: Option<String>) -> eyre::Result<std::path::PathBuf> {
    let dir = match dir {
        Some(dir) => dir.into(),
        // https://docs.rs/directories/6.0.0/directories/struct.ProjectDirs.html#method.data_dir
        None => match directories::ProjectDirs::from("com", "FifthTry", "ftn") {
            Some(dir) => dir.data_dir().to_path_buf(),
            None => {
                return Err(eyre::anyhow!("can not find data dir"));
            }
        },
    };

    if !dir.exists() {
        tokio::fs::create_dir_all(&dir).await?;
    }

    Ok(dir)
}
