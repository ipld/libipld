//! While all types and their IPLD representations ultimately dictate how the type is resolved from/writen to blocks of bytes, *how* those bytes may be provided (as well any additional requirements unique to the representation, such as additional blocks, encryption keys, etc) can vary on how and where the type is being used (e.g, in WASM, making partial/range queries, querying/mutating by IPLD selector), etc.
//!
//! Therefore, we create these traits to abstract over how to `Read`, `Write` a type from/to bytes, as well query and mutate a type, while specifically defining for the type it's `Context` requirements for these operations.
use crate::{async_trait, Read, Write};

pub mod context;
pub mod error;
mod implementations;

/// An interface for `Read`ing and `Write`ing a type from/to an execution `Context`.
#[async_trait]
pub trait Representation<R, W>: Sized
where
    R: Read,
    W: Write,
{
    /// `Read` a type from a provided `Context`.
    async fn read<C>(ctx: &mut C) -> Result<Self, error::Error>
    where
        R: 'async_trait,
        W: 'async_trait,
        C: context::Context<R, W> + Send;

    /// `Write` a type to a provided `Context`.
    async fn write<C>(&self, ctx: &mut C) -> Result<(), error::Error>
    where
        R: 'async_trait,
        W: 'async_trait,
        C: context::Context<R, W> + Send;
}

