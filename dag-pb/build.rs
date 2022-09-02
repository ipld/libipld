fn main() {
    std::env::set_var("PROTOC", protobuf_src::protoc());
    prost_build::compile_protos(&["src/dag_pb.proto"], &["src"]).unwrap();
}
