// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

#[tokio::main]
async fn main() -> eyre::Result<()> {
    use clap::Parser;

    // run with RUST_LOG="malai=info" to only see our logs when running with the --trace flag
    tracing_subscriber::fmt::init();

    let cli = Cli::parse();


    if let Err(e) = match cli.command {
        Some(Command::Http {
            port,
            host,
            // secure,
            // what_to_do,
        }) => {
            tracing::info!(port, host, verbose = ?cli.verbose, "Exposing HTTP service on kulfi.");
            malai::expose_http(host, port).await
        }
        Some(Command::HttpBridge { proxy_target, port }) => {
            tracing::info!(port, proxy_target, verbose = ?cli.verbose, "Starting HTTP bridge.");
            malai::http_bridge(proxy_target, port).await
        }
        Some(Command::Tcp { port, host }) => {
            tracing::info!(port, host, verbose = ?cli.verbose, "Exposing TCP service on kulfi.");
            malai::expose_tcp(host, port).await
        }
        Some(Command::TcpBridge { proxy_target, port }) => {
            tracing::info!(port, proxy_target, verbose = ?cli.verbose, "Starting TCP bridge.");
            malai::tcp_bridge(proxy_target, port).await
        }
        #[cfg(feature = "ui")]
        None => {
            tracing::info!(verbose = ?cli.verbose, "Starting UI.");
            malai::ui()
        }
        #[cfg(not(feature = "ui"))]
        None => {
            use clap::CommandFactory;
            Cli::command().print_help().map_err(Into::into)
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
    pub command: Option<Command>,

    // adding these two because when we run `cargo tauri dev,` it automatically passes these
    // arguments. need to figure out why and how to disable that, till then this is a workaround
    #[arg(default_value = "true", long, hide = true)]
    no_default_features: bool,
    #[arg(default_value = "auto", long, hide = true)]
    color: String,
}

#[derive(clap::Subcommand, Debug)]
pub enum Command {
    // TODO: add this to the docs when we have ACL
    // By default it allows any peer to connect to the HTTP(s) service. You can pass --what-to-do
    // argument to specify a What To Do service that can be used to add access control."
    #[clap(about = "Expose HTTP Service on kulfi, connect using kulfi or browser")]
    Http {
        port: u16,
        #[arg(
            long,
            default_value = "127.0.0.1",
            help = "Host serving the http service."
        )]
        host: String,
        // #[arg(
        //     long,
        //     default_value_t = false,
        //     help = "Use this if the service is HTTPS"
        // )]
        // secure: bool,
        // #[arg(
        //     long,
        //     help = "The What To Do Service that can be used to add access control."
        // )]
        // this will be the id52 of the identity server that should be consulted
        // what_to_do: Option<String>,
    },
    #[clap(about = "Expose TCP Service on kulfi")]
    Tcp {
        port: u16,
        #[arg(
            long,
            default_value = "127.0.0.1",
            help = "Host serving the TCP service."
        )]
        host: String,
    },
    HttpBridge {
        #[arg(
            long,
            short('t'),
            help = "The id52 to which this bridge will forward incoming HTTP request. By default it forwards to every id52."
        )]
        proxy_target: Option<String>,
        #[arg(
            long,
            short('p'),
            help = "The port on which this bridge will listen for incoming HTTP requests.",
            default_value = "8080"
        )]
        port: u16,
    },
    TcpBridge {
        #[arg(help = "The id52 to which this bridge will forward incoming TCP request.")]
        proxy_target: String,
        #[arg(
            help = "The port on which this bridge will listen for incoming TCP requests.",
            default_value = "8081"
        )]
        port: u16,
    },
}
