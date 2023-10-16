# Propagating Headers When Using GRPC in Rust

## Motivation

[gRPC](https://grpc.io/docs/what-is-grpc/introduction/) is a remarkable communication protocol, known for its speed, versatility, and efficiency. Developed by Google, it excels in high-performance scenarios, offers support for multiple languages, simplifies development, and enhances security and scalability. With clear interface definitions and powerful features, gRPC stands as a top choice for building modern, reliable distributed systems.

Many useful Kubernetes tools, such as OpenTelemetry and Telepresence, require header propagation to function correctly. This resitory shows how to propigate headers in Rust using the [tower](https://docs.rs/tower/latest/tower/) middleware crate within the [tonic](https://docs.rs/tonic/latest/tonic/) grpc crate.

This demo repository can be found [here](https://github.com/njayp/rust-tonic-header-propagation).

## Prerequisites

- Rust (rustup 1.26, rustc 1.73)
- Protoc (libprotoc 24.4)

## Scaffolding

A simple ping service is used for this demonstration.

```
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

## Propagating Headers from Request to Response

`tower` is a versatile open-source Rust library that provides essential components for building high-performance, composable network services. It emphasizes modularity and efficiency, making it an excellent choice for creating robust, asynchronous systems capable of handling large volumes of requests. With an active community and continuous improvements, `tower` is a key tool in the Rust ecosystem for crafting reliable and efficient network services.

`tower` Boilerplate looks like this.

```
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

```
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
```

Adding this middleware to the `tonic` server is simple.

```
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
```

## Propagating Headers from Call to Call

When a server needs to make additional `gRPC` calls, headers should be propagated from the original call to the new call. This can be done during the creation of the new request.

```
fn merge_metadata(metadata_into: &mut MetadataMap, metadata_from: &MetadataMap) {
    for key_and_value in metadata_from.iter() {
        match key_and_value {
            KeyAndValueRef::Ascii(key, value) => {
                if key.to_string().starts_with("x-") {
                    metadata_into.insert(key, value.to_owned());
                }
            }
            default => (),
        }
    }
}

...

    async fn forward_ping(
        &self,
        request: Request<ForwardPingRequest>,
    ) -> Result<Response<EmptyMessage>, Status> {
        print_metadata(request.metadata());

        // create new request
        let mut forward_request = Request::new(EmptyMessage {});
        // propagate metadata to new request
        merge_metadata(forward_request.metadata_mut(), request.metadata());
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

## Acknowledgements

- [Building gRPC APIs with Rust](https://konghq.com/blog/engineering/building-grpc-apis-with-rust)
- [GitHub Tonic Examples](https://github.com/hyperium/tonic/blob/master/examples/src/tower/server.rs)
