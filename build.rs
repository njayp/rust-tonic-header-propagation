fn main() -> Result<(), Box<dyn std::error::Error>> {
    // TODO compile all .proto files in dir
    let proto_file = "./proto/demo.proto";

    tonic_build::configure()
        .build_client(true)
        .build_server(true)
        .out_dir("./src/grpc")
        .compile(&[proto_file], &["proto"])?;
    Ok(())
}
