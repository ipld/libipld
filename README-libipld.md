The [Ipld] crate.

*InterPlanetary Linked Data* (IPLD) is a data model to enable decentralized data structures that are universally addressable and linkable, which in turn will enable more decentralized applications; see [ipld.io/docs](https://ipld.io/docs/).

The *Data Model* is decoupled from a particular *Codec* in IPLD; see [ipld.io/glossary](https://ipld.io/glossary/).

# Serde Support

In the rust ecosystem, the IPLD Data Model vs Codec distinction is very similar approach to [serde](https://serde.rs), and this crate can leverage `serde` for codecs via the `serde-codec` feature. The [*serde Data Model*](https://serde.rs/data-model.html#types) and the [*IPLD Data Model*](https://ipld.io/glossary/#data-model) are different: `serde` provides a wider range of data types, while it lacks the [`link` kind](https://ipld.io/glossary/#link) which is essential to IPLD's purpose. This crate uses a work-around by encoding `link` types via opaque byte sequences in the `serde` layer.

This crate began with a separate codec API based on the [codec] module. This API in effect attempts to recreate all of `serde` with support for links which, while cleaner, requires much more effort rather than relying on `serde`.
