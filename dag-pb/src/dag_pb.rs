/// An IPFS MerkleDAG Link
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PbLink {
    /// binary CID (with no multibase prefix) of the target object
    #[prost(bytes = "bytes", tag = "1")]
    pub hash: ::prost::bytes::Bytes,
    /// UTF-8 string name
    #[prost(string, tag = "2")]
    pub name: ::prost::alloc::string::String,
    /// cumulative size of target object
    #[prost(uint64, tag = "3")]
    pub tsize: u64,
}
/// An IPFS MerkleDAG Node
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PbNode {
    /// refs to other objects
    #[prost(message, repeated, tag = "2")]
    pub links: ::prost::alloc::vec::Vec<PbLink>,
    /// opaque user data
    #[prost(bytes = "bytes", tag = "1")]
    pub data: ::prost::bytes::Bytes,
}
