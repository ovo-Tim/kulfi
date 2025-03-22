pub mod dotftn;
mod read;

#[derive(Debug)]
pub struct Config {
    pub dir: std::path::PathBuf,
    lock_file: std::fs::File,
}

pub use read::ReadError;
