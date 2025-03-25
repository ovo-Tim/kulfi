#[tokio::main]
async fn main() {
    use clap::Parser;

    let cli = ftnet::Cli::parse();
    println!("{cli:?}");
    if let Err(e) = match cli.command {
        ftnet::Command::Start {
            foreground,
            data_dir,
        } => ftnet::start(foreground, data_dir).await,
        ftnet::Command::TcpProxy { id, port } => {
            println!("Proxying TCP server to remote ftn service with id: {id}, port: {port}");
            Ok(())
        }
    } {
        eprintln!("error: {e}");
        std::process::exit(1);
    }
}
