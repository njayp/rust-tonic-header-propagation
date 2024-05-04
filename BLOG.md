Oct 27, 2023

## Propagating Headers When Using GRPC in Rust

### Motivation
Many useful Kubernetes tools, such as OpenTelemetry and [Telepresence](https://www.getambassador.io/products/telepresence), require header propagation to function. Today, I wanted to share with you my methods of tackling this challenge. Let's dive into [gRPC](https://grpc.io/docs/what-is-grpc/introduction/), [Tower](https://docs.rs/tower/latest/tower/), and [Tonic](https://docs.rs/tonic/latest/tonic/)! 

This repository for this demo can be found [here](https://github.com/njayp/rust-tonic-header-propagation).

### Prerequisites

- Rust (rustup 1.26, rustc 1.73)
- Protoc (libprotoc 24.4)

### Scaffolding

A simple ping service is used for this demonstration.

```proto
service Demo {
    rpc Ping(EmptyMessage) returns (EmptyMessage);
    rpc ForwardPing(ForwardPingRequest) returns (EmptyMessage);
}

message EmptyMessage {
}

message ForwardPingRequest {
    uint32 port = 1;
}
```

### Propagating Headers from Request to Response

`tower` is a versatile open-source Rust library that provides essential components for building high-performance, composable network services. It emphasizes modularity and efficiency, making it an excellent choice for creating robust, asynchronous systems capable of handling large volumes of requests.

`tower` middleware boilerplate looks like this.

```rust
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
            // Do extra async work here...
            let response = inner.call(req).await?;

            Ok(response)
        })
    }
}
```

Header propagation needs to be added to the call function.

```rust
    fn call(&mut self, req: hyper::Request<Body>) -> Self::Future {
        // This is necessary because tonic internally uses `tower::buffer::Buffer`.
        // See https://github.com/tower-rs/tower/issues/547#issuecomment-767629149
        // for details on why this is necessary
        let clone = self.inner.clone();
        let mut inner = std::mem::replace(&mut self.inner, clone);

        Box::pin(async move {
            // copy request header map
            let req_header_map = &req.headers().clone();
            let mut response = inner.call(req).await?;
            // propagate "x-" headers from copied request header map to response header map
            for (key, value) in req_header_map.iter() {
                if key.to_string().starts_with("x-") {
                    response.headers_mut().insert(key, value.clone());
                }
            }

            Ok(response)
        })
    }
```

Adding this middleware to the `tonic` server is simple.

```rust
    let layer = tower::ServiceBuilder::new()
        .timeout(Duration::from_secs(30))
        // Apply our own middleware
        .layer(MyMiddlewareLayer::default())
        .into_inner();

    Server::builder()
        .layer(layer)
        .add_service(DemoServer::new(DemoServerImpl {}))
        .serve(format!("[::1]:{}", port).parse()?)
        .await?;
```

### Propagating Headers from Call to Call

When a server needs to make additional `gRPC` calls, headers should be propagated from the original call to the new call. There are several ways to do this. This method takes advantage of [interceptors](https://docs.rs/tonic/latest/tonic/service/interceptor/trait.Interceptor.html) from the `tonic` crate. First, in the server code, store the metadata of the original request in the extensions of the new request.

```rust
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
```

Create an interceptor function that performs header propagation on the request using the metadata stored in the request's extension map.

```rust
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
```

Add the interceptor function to the client.

```rust
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
```

### More Resources
I hope you found this deep dive into `Tonic` and `gRPC` helpful! Now, you're ready to create some proper header propagation in your own [Telepresence](https://www.getambassador.io/products/telepresence) instance.

- [Building gRPC APIs with Rust](https://konghq.com/blog/engineering/building-grpc-apis-with-rust)
- [GitHub Tonic Examples](https://github.com/hyperium/tonic/blob/master/examples/src/tower/server.rs)