# axum-grpc-example
A lightweight example of using Axum with gRPC.

### Building
Ensure you have protoc installed in your environment:
```shell
sudo apt update && \
sudo apt install -y protobuf-compiler
```
Build and generate protobuf classes from `hello.proto` with cargo build:
```shell
cargo build
```
You should see a generated file in `target/debug/build/axum-grpc-example-<guid>/out/hello.rs`

### Running
Run the service with:
```shell
cargo run
```

With the service up, check the REST endpoint with:
```shell
curl http://127.0.0.1:3000/health
```

Check the gRPC endpoint with any gRPC client, we suggest [evans](https://github.com/ktr0731/evans):
```shell
evans --proto proto/hello.proto repl
package hello
service Greeter
call SayHello
<enter your name>
```