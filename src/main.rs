mod grpc {
    pub mod client;
    mod rpc;
    pub mod server;
    mod util;
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    //grpc::server::run(9090).await?;
    grpc::client::ping_with_demo_header(9090).await?;
    Ok(())
}
