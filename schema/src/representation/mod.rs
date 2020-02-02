//! While all types and their IPLD representations ultimately dictate how the type is resolved from/writen to blocks of bytes, *how* those bytes may be provided (as well any additional requirements unique to the representation, such as additional blocks, encryption keys, etc) can vary on how and where the type is being used (e.g, in WASM, making partial/range queries, querying/mutating by IPLD selector), etc.
//!
//! Therefore, we create these traits to abstract over how to `Read`, `Write` a type from/to bytes, as well query and mutate a type, while specifically defining for the type it's `Context` requirements for these operations.

pub mod context;

use crate::{dev::*, Error};
use context::Context;
//use libipld_base::codec::Codec;

/// An interface for `Encode`ing and `Decode`ing an IPLD Representation.
///
/// Types that have `Representation`s generally follow the same few steps when
/// encoding (in reverse for decoding):
///     - pre-processing, i.e.:
///         fetching codecs
///         generating signatures
///         converting bytes to hex
///     - (? optionally) conversion of the type to an Ipld-like
///         helpful for ensuring canonicalization
///     - serializing the Ipld-like type with a provided Codec
/// decoding:
///     - pre-processing, i.e.:
///         fetching blocks
///     - deserializing either:
///         - to an Ipld-like type, then conversion to native type
///         - to a native type directly
///
/// The supplied execution `Context` provides `Codec` to use, and can also:
///     - dictate which fields to `Read`/`Write`,
///     - provide a source/sink of bytes for a particular `Cid`/`Block`
#[async_trait]
pub trait Representation: Sized {
    //    /// Encodes a type to a provided `Context`.
    //    ///
    //    /// By default, creates an IPLD data type representation from the type, then
    //    /// encodes the `Ipld` with the provided `Codec`.
    //    async fn encode<Ctx>(&self, ctx: &Ctx) -> Result<Option<Cid>, Error>
    //    where
    //        Ctx: Context,
    //    {
    ////        let dag = self.to_ipld(ctx).await?;
    ////        ctx.codec().encode(dag)?
    //    }
    //
    //    /// `Read` a type from a provided `Context`.
    //    async fn decode<Ctx>(bytes: &[u8], ctx: &Ctx) -> Result<Self, Error>
    //    where
    //        Ctx: Context,
    //    {
    ////        let dag = ctx.codec().decode(bytes).await?;
    ////        Self::from_ipld(dag, ctx)
    //    }

    //    /// `Read` a type from a provided `Context`.
    //    async fn read_with_ctx<NewCtx>(ctx: &Ctx) -> Result<Self, Error>
    //    where
    //        NewCtx: FromContext<Ctx>,
    //        Self: Representation<NewCtx>;

    //    /// `Write` a type to a provided `Context`.
    //    async fn write_with_ctx<NewCtx>(&self, ctx: &Ctx) -> Result<(), Error>
    //    where
    //        Co: 'async_trait,
    //        R: 'async_trait,
    //        W: 'async_trait,
    //        NewCtx: FromContext<Ctx>;
}
