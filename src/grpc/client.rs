use tonic::Request;

use super::{
    rpc::{demo_client::DemoClient, EmptyMessage, ForwardPingRequest},
    util::print_metadata,
};

pub async fn ping_with_request(
    port: u32,
    request: Request<EmptyMessage>,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut client = DemoClient::connect(port_to_url(port)).await?;
    let response = client.ping(request).await?;
    print_metadata(response.metadata());
    Ok(())
}

pub async fn ping_with_demo_header(port: u32) -> Result<(), Box<dyn std::error::Error>> {
    let mut request = tonic::Request::new(EmptyMessage {});

    // add custom header
    request
        .metadata_mut()
        .insert("x-demo", "x-demo-value".parse().unwrap());

    ping_with_request(port, request).await
}

pub async fn fwd_ping_with_demo_header(
    port: u32,
    fwd_port: u32,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut client = DemoClient::connect(port_to_url(port)).await?;
    let mut request = tonic::Request::new(ForwardPingRequest { port: fwd_port });

    // add custom header
    request
        .metadata_mut()
        .insert("x-demo", "x-demo-value".parse().unwrap());

    let response = client.forward_ping(request).await?;
    print_metadata(response.metadata());
    Ok(())
}

fn port_to_url(port: u32) -> String {
    format!("http://[::1]:{}", port)
}
