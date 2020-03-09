fn main() {
    prost_build::compile_protos(&["src/dag_pb.proto"], &["src"]).unwrap();
}
