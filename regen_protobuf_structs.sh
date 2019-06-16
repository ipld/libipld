#!/usr/bin/env sh

protoc --rust_out src/codec/pb src/codec/pb/dag_pb.proto
