use super::{Representation, WriteContext};
use crate::async_trait;
use crate::{Error, Write};

/// An empty interface for marking a type as a mutation.
pub trait Mutation {
    type Ok;
}

/// An interface for mutating a `Representation`.
#[async_trait]
pub trait Mutable<R, W, C>: Representation<R, W, C>
where
    W: Write,
    C: WriteContext<W>,
{
    type Mutation: Mutation;

    async fn apply(
        &mut self,
        ctx: &mut C,
        q: Self::Mutation,
    ) -> Result<<<Self as Mutable<R, W, C>>::Mutation as Mutation>::Ok, Error>;
}
