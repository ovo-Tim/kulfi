#[tokio::main]
async fn main() -> eyre::Result<()> {
    use clap::Parser;

    // run with RUST_LOG="skynet=info" to only see our logs when running with the --trace flag
    tracing_subscriber::fmt::init();

    let cli = Cli::parse();

    if let Err(e) = match cli.command {
        Command::ExposeHttp {
            port,
            secure,
            what_to_do,
        } => {
            tracing::info!(port, secure, what_to_do, verbose = ?cli.verbose, "Exposing HTTP service on FTNet.");
            skynet::expose_http(port, secure, what_to_do).await
        }
    } {
        tracing::error!("Error: {e}");
        return Err(e);
    }

    Ok(())
}

#[derive(clap::Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Cli {
    #[command(flatten)]
    verbose: clap_verbosity_flag::Verbosity,

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
