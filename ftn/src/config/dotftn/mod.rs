//! The $FTN folder
//!
//! The location of this folder is platform-specific, on Linux it is either
//! $HOME/.local/share/ftn or $XDG_DATA_HOME/ftn, on MacOS it is $HOME/Library/Application
//! Support/com.FifthTry.ftn and on Windows: {FOLDERID_RoamingAppData}\ftn\data which is usually
//! C:\Users\Alice\AppData\Roaming\FifthTry\ftn\data.
//!
//! The folder contains a lock file, `$FTN/ftn.lock, which is used to ensure only one instance
//! of `ftn` is running.
//!
//! The folder contains more folders like `identities`, `logs` and maybe `config.json` etc. in
//! the future.
//!
//! The identities folder is the most interesting one, it contains one folder for every identity
//! that exists on this machine. The content of single `identity` folder is described
//! in `identity/create.rs`.

pub const LOCK_FILE: &str = "ftn.lock";

mod init_if_required;
mod lock;

pub use init_if_required::init_if_required;
pub use lock::{exclusive, lock_file};
