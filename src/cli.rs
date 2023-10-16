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
    /// if this flag is set, forward ping to this port on localhost
    #[arg(short, long, default_value_t = 0)]
    fwd: u32,

    /// port number on localhost
    #[arg(short, long, default_value_t = 9090)]
    port: u32,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    if args.fwd != 0 {
        grpc::client::fwd_ping_with_demo_header(args.port, args.fwd).await?;
    } else {
        grpc::client::ping_with_demo_header(args.port).await?;
    }
    Ok(())
}
