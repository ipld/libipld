use crate::{async_trait, BlockError, Cid, IpldIndex, Read, Write};

/// An execution context for `Representation`s to `Read`/`Write` themselves to byte streams..
///
/// A `Representation::Context` is intended to provide the `Representation` implementation a way of specifying required operations for manipulating a Dag before reading from/after writing to a byte stream.
pub trait Context<R: Read, W: Write> {
    fn reader(&mut self) -> &mut R;
    fn writer(&mut self) -> &mut W;
}

/// An execution context for `Representation`s to `Read`/`Write` themselves from/to blocks.
#[async_trait]
pub trait BlockContext<R: Read, W: Write>: Context<R, W> {
    fn reader(&mut self, cid: &Cid) -> &mut R;
    fn writer(&mut self, old_cid: &Cid) -> &mut W;
    async fn flush(&mut self, w: &mut W) -> Result<Cid, BlockError>;
}

/// An execution context for recursing into a dag `Representation`.
pub trait RecursiveContext<R: Read, W: Write>: Context<R, W> {
    fn path(&self) -> &[&IpldIndex];
    fn push(&mut self, segment: &IpldIndex);
    fn pop(&mut self) -> IpldIndex;
}
