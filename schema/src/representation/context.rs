//! Execution contexts for `Representation`s to `Read`/`Write` themselves from/to bytes and query/mutate themselves by specializing their implementation around specific `State` changes.
//!
//! While a `Representation` defines how a type traverses it's fields and maps them to bytes or blocks, the `Context` determines what happens with the bytes when encountering nested types, links, etc, before writing to or after reading from the byte stream.
//!
//! For example:
//!     - An `impl Context for EncryptedContext` can provide a byte stream that encrypts bytes written from a type/decrypts bytes read into a type. Later, a `Representation` can be provided with an `EncyptedContext` initialized with a key, transparently encrypting/decrypting the provided byte streams.
//!     - Additionally, we can define an `impl State for Encrypted<R, W>: Context<R, W>` and a type whose `Representation` implementation could derive an encryption/decryption key from within the type, ensuring that the type can only be stored in ciphertext.
use crate::{async_trait, Cid, Error, IpldIndex, Read, Write};
use derive_more::{Constructor, From};

/// A state change operation to be applied to the `Context`.
pub trait State {
    /// Return value of the state change operation.
    type Result;
}

/// Traversing into a list element / map value.
#[derive(Constructor, From)]
pub struct PushElement<'a, I: Into<IpldIndex<'a>>>(&'a I);
impl<'a, I: Into<IpldIndex<'a>>> State for PushElement<'a, I> {
    type Result = bool;
}

/// Finishing traversal into a list element / map value.
pub struct PopElement;
impl State for PopElement {
    type Result = ();
}

/// Resolving a `Cid` into a dag / block.
#[derive(Constructor, From)]
pub struct ResolveLink<'a>(&'a Cid);
impl<'a> State for ResolveLink<'a> {
    type Result = bool;
}

/// Finishing resolving a `Cid` into a dag / block.
#[derive(Constructor, From)]
pub struct WriteLink<'a>(&'a Cid);
impl<'a> State for WriteLink<'a> {
    type Result = Result<Cid, Error>;
}

/// An execution context for `Representation`s to `Read`/`Write` themselves from/to bytes by signalling `State` changes to the `Context`.
#[async_trait]
pub trait Context<R: Read, W: Write> {
    /// Provides a `Read`.
    fn reader(&mut self) -> &mut R;

    /// Provides a `Write`.
    fn writer(&mut self) -> &mut W;

    /// Attempts to apply the current `State`, triggering optional side-effects within `Context`, allowing it to drive the `Representation` operation.
    async fn try_apply<S: State>(&mut self, state: S) -> S::Result;
}


    /// Informs the `Context` that the end of the value at the current `IpldIndex` has been reached.
    fn pop(&mut self) -> IpldIndex;
}
