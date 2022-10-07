fn main() {
    prost_build::Config::new()
        .bytes(&["."])
        .compile_protos(&["src/dag_pb.proto"], &["src"])
        .unwrap();
}
