fn main() {
    tonic_build::compile_protos("proto/hello.proto").unwrap();
    tonic_build::compile_protos("proto/counter.proto").unwrap();
}