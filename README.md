> # ⛔️ DEPRECATED: [libipld](https://github.com/ipld/libipld) has been superseded by [ipld-core](https://github.com/ipld/rust-ipld-core)
> The migration to `ipld-core` should be straight forward. If you run into problems during the migration or need help, feel free to [open a bug report on the `ipld-core` repo](https://github.com/ipld/rust-ipld-core/issues).


# Rust IPLD library

> Basic rust ipld library supporting `dag-cbor`, `dag-json` and `dag-pb` formats.

Originally authored by [@dvc94ch](https://github.com/dvc94ch) as a part of the [ipfs-rust](https://github.com/ipfs-rust/) project.

The `Ipld` enum from the `libipld-core` crate is the central piece that most of the users of this library use.

The codec implementations use custom traits. In order to be more compatible with the rest of the Rust ecosystem, it's *strongly recommended*, to use new implementations, that use [Serde](https://serde.rs/) as a basis instead. Currently, the list of implementations is limited, please let us know if you crate one and we'll add it to the list:

 - DAG-CBOR: https://github.com/ipld/serde_ipld_dagcbor

## Community

For chats with the developers and the community: Join us in any of these (bridged) locations:
  - On Matrix: [#ipld:ipfs.io](https://matrix.to/#/#ipld:ipfs.io)
  - On Discord: join the [IPLD community on IPFS Discord](https://discord.gg/xkUC8bqSCP).

## License

Dual licensed under MIT or Apache License (Version 2.0). See [LICENSE-MIT](./LICENSE-MIT) and [LICENSE-APACHE](./LICENSE-APACHE) for more details.
