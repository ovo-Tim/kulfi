#[tokio::main]
async fn main() {
    use clap::Parser;

    let cli = ftn::Cli::parse();
    println!("cli: {cli:?}");
}
