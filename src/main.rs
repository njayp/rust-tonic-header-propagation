mod grpc {
    pub mod client;
    mod interceptors;
    mod rpc;
    pub mod server;
    mod util;
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    grpc::server::run(9091).await?;
    grpc::client::fwd_ping_with_demo_header(9090, 9091).await?;
    Ok(())
}
