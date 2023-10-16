# rust-tonic-header-propagation

## Usage

This repo is a demo for header propagation in rust using `tonic` and `tower`. Start two servers and send requests to see this in action.

Start servers on 9090 and 9091

```cmd
cargo run --bin server
```

In a second terminal

```cmd
cargo run --bin server -- port 9091
```

Use the cli to send a request to 9090 and see the headers propagated to the response

```cmd
cargo run --bin cli
```

Use the cli to send a request to 9090 that is forwarded to 9091 and see the headers propagated to the forwarded request

```cmd
cargo run --bin cli -- --fwd 9091
```
