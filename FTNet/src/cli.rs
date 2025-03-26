#[derive(clap::Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(clap::Subcommand, Debug)]
pub enum Command {
    #[clap(about = "Start the FTNet service.")]
    Start {
        #[arg(default_value_t = false, short = 'f')]
        foreground: bool,
        data_dir: Option<String>,
    },
    #[clap(about = "Proxy TCP server to a remote FTNet service.")]
    TcpProxy {
        id: String,
        #[arg(default_value_t = 2345)]
        port: u16,
    },
}
