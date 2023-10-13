use core::clone::Clone;
use tonic::{transport::Endpoint, Request};

use super::{
    interceptors::{apply_context, Context},
    rpc::{demo_client::DemoClient, EmptyMessage, ForwardPingRequest},
    util::print_metadata,
};

pub async fn ping_with_ctx<T: 'static>(
    port: u32,
    req: Request<T>,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut fwd_req = Request::new(EmptyMessage {});
    // pass context from previous request to new request
    fwd_req
        .extensions_mut()
        .insert(req.extensions().get::<Context>().unwrap().clone());
    let channel = Endpoint::from_shared(port_to_url(port))?.connect().await?;
    let mut client = DemoClient::with_interceptor(channel, apply_context);
    let responce = client.ping(fwd_req).await?;
    print_metadata(responce.metadata());
    Ok(())
}

pub async fn ping_with_demo_header(port: u32) -> Result<(), Box<dyn std::error::Error>> {
    let mut client = DemoClient::connect(port_to_url(port)).await?;
    let mut request = tonic::Request::new(EmptyMessage {});

    // add custom header
    request
        .metadata_mut()
        .insert("x-demo", "x-demo-value".parse().unwrap());

    let response = client.ping(request).await?;
    print_metadata(response.metadata());
    Ok(())
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
