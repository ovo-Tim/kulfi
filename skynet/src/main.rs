#[tokio::main]
async fn main() -> eyre::Result<()> {
    use clap::Parser;

    // run with RUST_LOG="skynet=info" to only see our logs when running with the --trace flag
    tracing_subscriber::fmt::init();

    let cli = Cli::parse();

    match cli.command {
        Command::ExposeHttp { port, secure } => {
            tracing::info!("Exposing HTTP service on FTNet with port: {port}, secure: {secure}.");
        }
    }

    Ok(())
}

#[derive(clap::Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Cli {
    #[arg(long, global = true)]
    pub trace: bool,

    #[command(subcommand)]
    pub command: Command,
}

#[derive(clap::Subcommand, Debug)]
pub enum Command {
    #[clap(about = "Expose HTTP Service on FTNet, connect using FTNet")]
    ExposeHttp {
        port: u16,
        #[arg(
            long,
            default_value_t = false,
            help = "Use this if the service is HTTPS"
        )]
        secure: bool,
    },
}
