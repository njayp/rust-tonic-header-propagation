use tonic::{metadata::MetadataMap, transport::Endpoint, Request, Status};

use super::{
    rpc::{demo_client::DemoClient, EmptyMessage, ForwardPingRequest},
    util::{merge_metadata, print_metadata},
};

/// This function will get called on each outbound request. Returning a
/// `Status` here will cancel the request and have that status returned to
/// the caller.
///
/// extract metadata map from extensions if it exists,
/// then merge it into request metadata
fn intercept(mut req: Request<()>) -> Result<Request<()>, Status> {
    let blankmap = &MetadataMap::default();
    // extract metadata map from extensions
    let metadata = req
        .extensions()
        .get::<MetadataMap>()
        .unwrap_or(blankmap)
        .clone();

    merge_metadata(req.metadata_mut(), &metadata);
    Ok(req)
}

pub async fn ping_with_request(
    port: u32,
    request: Request<EmptyMessage>,
) -> Result<(), Box<dyn std::error::Error>> {
    let channel = Endpoint::from_shared(port_to_url(port))?.connect().await?;
    let mut client = DemoClient::with_interceptor(channel, intercept);
    let response = client.ping(request).await?;
    print_metadata(response.metadata());
    Ok(())
}

pub async fn ping_with_demo_header(port: u32) -> Result<(), Box<dyn std::error::Error>> {
    let mut request = tonic::Request::new(EmptyMessage {});
    add_demo_header(&mut request);
    ping_with_request(port, request).await
}

pub async fn fwd_ping_with_demo_header(
    port: u32,
    fwd_port: u32,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut request = tonic::Request::new(ForwardPingRequest { port: fwd_port });
    add_demo_header(&mut request);

    let channel = Endpoint::from_shared(port_to_url(port))?.connect().await?;
    let mut client = DemoClient::with_interceptor(channel, intercept);
    let response = client.forward_ping(request).await?;
    print_metadata(response.metadata());
    Ok(())
}

fn add_demo_header<T>(request: &mut Request<T>) {
    request
        .metadata_mut()
        .insert("x-demo-key", "x-demo-value".parse().unwrap());
}

fn port_to_url(port: u32) -> String {
    format!("http://[::1]:{}", port)
}
