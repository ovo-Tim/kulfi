#[tokio::main]
async fn main() {
    use clap::Parser;

    let cli = ftnet::Cli::parse();
    println!("cli: {cli:?}");
}
