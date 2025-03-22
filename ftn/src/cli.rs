#[derive(clap::Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(clap::Subcommand, Debug)]
pub enum Command {
    #[clap(about = "Start the ftn service.")]
    Start,
    #[clap(about = "Proxy TCP server to a remote ftn service.")]
    TcpProxy {
        id: String,
        #[arg(default_value_t = 2345)]
        port: u16,
    },
}
