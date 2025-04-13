pub mod dot_kulfi;
mod identities;
mod read;

#[derive(Debug)]
pub struct Config {
    pub dir: std::path::PathBuf,
    lock_file: std::fs::File,
}
