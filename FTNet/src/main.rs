#[tokio::main]
async fn main() -> eyre::Result<()> {
    use clap::Parser;

    // run with RUST_LOG="ftnet=info" to only see our logs when running with the --trace flag
    configure_tracing_subscriber();

    let cli = ftnet::Cli::parse();

    match cli.command {
        ftnet::Command::Start {
            foreground,
            data_dir,
            control_port,
        } => ftnet::start(foreground, data_dir, control_port).await,
        ftnet::Command::TcpProxy { id, port } => {
            println!("Proxying TCP server to remote FTNet service with id: {id}, port: {port}");
            Ok(())
        }
    }
}

fn configure_tracing_subscriber() {
    use tracing_subscriber::layer::SubscriberExt;

    tracing::subscriber::set_global_default(
        tracing_subscriber::registry()
            .with(fastn_observer::Layer::default())
            .with(tracing_subscriber::EnvFilter::from_default_env()),
    )
    .unwrap();
}
