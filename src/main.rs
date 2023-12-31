use clap::Parser;

mod grpc {
    pub mod client;
    mod rpc;
    pub mod server;
    mod util;
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// port number on localhost
    #[arg(short, long, default_value_t = 9090)]
    port: u32,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    grpc::server::run(args.port).await?;
    Ok(())
}
