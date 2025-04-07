#[tokio::main]
async fn main() -> eyre::Result<()> {
    use clap::Parser;

    // run with RUST_LOG="skynet=info" to only see our logs when running with the --trace flag
    tracing_subscriber::fmt::init();

    let cli = Cli::parse();

    match cli.command {
        Command::ExposeHttp {
            port,
            secure,
            what_to_do,
        } => {
            tracing::info!(
                action = "Exposing HTTP service on FTNet.",
                port = port,
                secure = secure,
                what_to_do = what_to_do
            );
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
    #[clap(
        about = "Expose HTTP Service on FTNet, connect using FTNet.",
        long_about = r#"
Expose HTTP Service on FTNet, connect using FTNet.

By default it allows any peer to connecto to the HTTP(s) service. You can pass --what-to-do
argument to specify a What To Do service that can be used to add access control."#
    )]
    ExposeHttp {
        port: u16,
        #[arg(
            long,
            default_value_t = false,
            help = "Use this if the service is HTTPS"
        )]
        secure: bool,
        #[arg(
            long,
            help = "The What To Do Service that can be used to add access control."
        )]
        what_to_do: Option<String>,
    },
}
