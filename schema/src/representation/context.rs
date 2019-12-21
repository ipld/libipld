//! Execution contexts for `Representation`s to `Read`/`Write` themselves from/to bytes and query/mutate themselves by specializing their implementation around specific `Context` requirements.
//!
//! While a `Representation` defines how a type traverses it's fields and maps them to bytes or blocks, the provided `Context` determines what happens with the bytes before writing/after reading from the byte stream.
//!
//! For example:
//!     - An `impl Context for EncryptedContext` can provide a byte stream that encrypts bytes written from a type/decrypts bytes read into a type. Later, a `Representation<C: Context>` can be provided with an `EncyptedContext` initialized with a key, transparently encrypting/decrypting the provided byte streams.
//!     - Additionally, we can define a `trait EncryptedContext<R, W>: Context<R, W>` and a type whose `Representation` implementation could derive an encryption/decryption key from within the type, ensuring that the type can only be stored in ciphertext.
use crate::{async_trait, BlockError, Cid, IpldIndex, Read, Write};

/// An execution context for `Representation`s to `Read` themselves from bytes by specializing their implementation around specific `Context` requirements.
pub trait ReadContext<R: Read> {
    /// Provides a `Read`.
    fn reader(&mut self) -> &mut R;
}

/// An execution context for `Representation`s to `Write` themselves to bytes by specializing their implementation around specific `Context` requirements.
pub trait WriteContext<W: Write> {
    /// Provides a `Write`.
    fn writer(&mut self) -> &mut W;
}

/// An execution context for `Representation`s to `Read` themselves from a separate separate block's byte stream by `Cid`.
pub trait BlockReadContext<R: Read>: ReadContext<R> {
    /// Informs the `Context` that a link has been reached, and allows the `Context` to determine whether or not it should be resolved.
    fn should_read(&mut self, cid: &Cid) -> bool;
}

/// An execution context for `Representation`s to `Write` themselves to a separate block's byte stream, producing a new block and `Cid`.
#[async_trait]
pub trait BlockWriteContext<W: Write>: WriteContext<W> {
    /// Informs the `Context` that a link has been reached, allowing the `Context` to determine whether or not the inner dag should be written to a separate block.
    fn start(&mut self) -> bool;

    /// Informs the `Context` that the inner dag is finished writing (optionally providing the `Cid` of the dag we're overwriting), and that the `Context` should produce it's `Cid`.
    async fn end(&mut self, old_cid: Option<&Cid>) -> Result<Cid, BlockError>;
}

/// An execution context for recursing into a dag `Representation`.
pub trait RecursiveContext {
    /// Informs the `Context` that an value at an `IpldIndex` has been reached.
    fn push<'a, T: Into<IpldIndex<'a>>>(&mut self, idx: T) -> bool;

    /// Informs the `Context` that the end of the value at the current `IpldIndex` has been reached.
    fn pop(&mut self) -> IpldIndex;
}
