fn main() -> Result<(), protoc_rust::Error> {
    protoc_rust::run(protoc_rust::Args {
        out_dir: "src/codec/pb/gen",
        input: &["src/codec/pb/gen/dag_pb.proto"],
        ..Default::default()
    })
}
