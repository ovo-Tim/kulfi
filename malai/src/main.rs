#[tokio::main]
async fn main() -> eyre::Result<()> {
    use clap::Parser;

    // run with RUST_LOG="malai=info" to only see our logs when running with the --trace flag
    tracing_subscriber::fmt::init();
    // configure_tracing_subscriber();

    let cli = Cli::parse();

    match cli.command {
        Command::Start {
            foreground,
            data_dir,
            control_port,
        } => {
            let data_dir = match data_dir {
                Some(dir) => dir.into(),
                // https://docs.rs/directories/6.0.0/directories/struct.ProjectDirs.html#method.data_dir
                None => match directories::ProjectDirs::from("com", "FifthTry", "malai") {
                    Some(dir) => dir.data_dir().to_path_buf(),
                    None => {
                        return Err(eyre::anyhow!(
                            "dot_malai init failed: can not find data dir when dir is not provided"
                        ));
                    }
                },
            };

            malai::start(foreground, data_dir, control_port).await
        }
        Command::TcpProxy { id, port } => {
            tracing::info!(
                "Proxying TCP server to remote malai service with id: {id}, port: {port}"
            );
            Ok(())
        }
    }
}

#[expect(dead_code)]
fn configure_tracing_subscriber() {
    use tracing_subscriber::layer::SubscriberExt;

    tracing::subscriber::set_global_default(
        tracing_subscriber::registry()
            .with(fastn_observer::Layer::default())
            .with(tracing_subscriber::EnvFilter::from_default_env()),
    )
    .unwrap();
}

#[derive(clap::Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,

    #[arg(long, global = true)]
    pub trace: bool,
}

#[derive(clap::Subcommand, Debug)]
pub enum Command {
    #[clap(about = "Start the malai service.")]
    Start {
        #[arg(default_value_t = false, short = 'f')]
        foreground: bool,
        #[arg(long, short = 'd')]
        data_dir: Option<String>,
        #[arg(default_value_t = 80, long, short = 'p')]
        control_port: u16,
    },
    #[clap(about = "Proxy TCP server to a remote malai service.")]
    TcpProxy {
        id: String,
        #[arg(default_value_t = 2345)]
        port: u16,
    },
}
