use tonic::{transport::Server, Request, Response, Status};

use super::{
    client::ping_with_ctx,
    interceptors::{save_context, Context},
    rpc::{
        demo_server::{Demo, DemoServer},
        EmptyMessage, ForwardPingRequest,
    },
    util::{merge_metadata, print_metadata},
};

struct DemoServerImpl {}

#[tonic::async_trait]
impl Demo for DemoServerImpl {
    async fn ping(&self, request: Request<EmptyMessage>) -> Result<Response<EmptyMessage>, Status> {
        print_metadata(request.metadata());
        // propagate headers to response
        let mut response = Response::new(EmptyMessage {});
        merge_metadata(response.metadata_mut(), request.metadata());
        return Ok(response);
    }

    async fn forward_ping(
        &self,
        request: Request<ForwardPingRequest>,
    ) -> Result<Response<EmptyMessage>, Status> {
        print_metadata(request.metadata());

        // fwd request
        let mut fwd_request = Request::new(EmptyMessage {});
        // pass context from previous request to new request
        let context = request.extensions().get::<Context>().unwrap().clone();
        fwd_request.extensions_mut().insert(context);
        // fwd ping
        match ping_with_ctx(request.get_ref().port, fwd_request).await {
            Ok(()) => (),
            Err(err) => {
                println!("error: {}", err.to_string())
            }
        }

        // propagate headers to response
        let mut response = Response::new(EmptyMessage {});
        merge_metadata(response.metadata_mut(), request.metadata());
        return Ok(response);
    }
}

pub async fn run(port: u32) -> Result<(), Box<dyn std::error::Error>> {
    Server::builder()
        .add_service(DemoServer::with_interceptor(
            DemoServerImpl {},
            save_context,
        ))
        .serve(port_to_addr(port).parse()?)
        .await?;

    Ok(())
}

fn port_to_addr(port: u32) -> String {
    format!("[::1]:{}", port)
}
