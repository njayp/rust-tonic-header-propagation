use std::{
    pin::Pin,
    task::{Context, Poll},
    time::Duration,
};

use hyper::Body;
use tonic::{body::BoxBody, transport::Server, Request, Response, Status};
use tower::{Layer, Service};

use super::{
    client::ping_with_request,
    rpc::{
        demo_server::{Demo, DemoServer},
        EmptyMessage, ForwardPingRequest,
    },
    util::print_metadata,
};

struct DemoServerImpl {}

#[tonic::async_trait]
impl Demo for DemoServerImpl {
    async fn ping(&self, request: Request<EmptyMessage>) -> Result<Response<EmptyMessage>, Status> {
        print_metadata(request.metadata());
        let response = Response::new(EmptyMessage {});
        return Ok(response);
    }

    async fn forward_ping(
        &self,
        request: Request<ForwardPingRequest>,
    ) -> Result<Response<EmptyMessage>, Status> {
        print_metadata(request.metadata());

        // create new request
        let mut forward_request = Request::new(EmptyMessage {});
        // store request metadata in new request
        forward_request
            .extensions_mut()
            .insert(request.metadata().clone());
        // send forward request
        match ping_with_request(request.get_ref().port, forward_request).await {
            Ok(()) => (),
            Err(err) => {
                // handle upstream error
                println!("error: {}", err.as_ref())
            }
        }

        let response = Response::new(EmptyMessage {});
        return Ok(response);
    }
}

pub async fn run(port: u32) -> Result<(), Box<dyn std::error::Error>> {
    // The stack of middleware that our service will be wrapped in
    let layer = tower::ServiceBuilder::new()
        // Apply middleware from tower
        .timeout(Duration::from_secs(30))
        // Apply our own middleware
        .layer(MyMiddlewareLayer::default())
        .into_inner();

    Server::builder()
        .layer(layer)
        .add_service(DemoServer::new(DemoServerImpl {}))
        .serve(format!("[::1]:{}", port).parse()?)
        .await?;

    Ok(())
}

// Tower boilerplate for creating middleware
#[derive(Debug, Clone, Default)]
struct MyMiddlewareLayer;

impl<S> Layer<S> for MyMiddlewareLayer {
    type Service = MyMiddleware<S>;

    fn layer(&self, service: S) -> Self::Service {
        MyMiddleware { inner: service }
    }
}

#[derive(Debug, Clone)]
struct MyMiddleware<S> {
    inner: S,
}

type BoxFuture<'a, T> = Pin<Box<dyn std::future::Future<Output = T> + Send + 'a>>;

impl<S> Service<hyper::Request<Body>> for MyMiddleware<S>
where
    S: Service<hyper::Request<Body>, Response = hyper::Response<BoxBody>> + Clone + Send + 'static,
    S::Future: Send + 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: hyper::Request<Body>) -> Self::Future {
        // This is necessary because tonic internally uses `tower::buffer::Buffer`.
        // See https://github.com/tower-rs/tower/issues/547#issuecomment-767629149
        // for details on why this is necessary
        let clone = self.inner.clone();
        let mut inner = std::mem::replace(&mut self.inner, clone);

        Box::pin(async move {
            // copy request header map for later use
            let req_header_map = &req.headers().clone();
            let mut response = inner.call(req).await?;
            // propagate "-x" headers from copied request header map to response header map
            for (key, value) in req_header_map.iter() {
                if key.to_string().starts_with("x-") {
                    response.headers_mut().insert(key, value.clone());
                }
            }

            Ok(response)
        })
    }
}
