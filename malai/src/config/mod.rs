pub mod dot_malai;
mod identities;
mod read;

#[derive(Debug)]
pub struct Config {
    pub dir: std::path::PathBuf,
    lock_file: std::fs::File,
}
