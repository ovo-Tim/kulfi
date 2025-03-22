#[derive(clap::Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(clap::Subcommand, Debug)]
pub enum Command {
    #[clap(about = "Start the ftnet service.")]
    Start,
    #[clap(about = "Connect to a remote ftnet service.")]
    Proxy { id: String },
}
