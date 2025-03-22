#[derive(Debug, thiserror::Error)]
pub enum InitError {
    #[error("could not find home directory")]
    NoHomeDir,
    #[error("failed to create directory {0}: {1}")]
    CreateDir(std::path::PathBuf, std::io::Error),
}

/// this function is called on startup, and initializes the .ftn directory if it doesn't exist
pub async fn init_if_required(dir: Option<String>) -> Result<std::path::PathBuf, InitError> {
    let dir = match dir {
        Some(dir) => dir.into(),
        // https://docs.rs/directories/6.0.0/directories/struct.ProjectDirs.html#method.data_dir
        None => match directories::ProjectDirs::from("com", "FifthTry", "ftn") {
            Some(dir) => dir.data_dir().to_path_buf(),
            None => {
                return Err(InitError::NoHomeDir);
            }
        },
    };

    if !dir.exists() {
        match tokio::fs::create_dir_all(&dir).await {
            Ok(_) => (),
            Err(e) => return Err(InitError::CreateDir(dir, e)),
        }
    }

    Ok(dir)
}
