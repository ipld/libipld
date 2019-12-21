use super::{ReadContext, Representation};
use crate::async_trait;
use crate::{Error, Read};

/// An interface for marking a type as a query.
pub trait Query {
    type Ok;
}

/// An interface for querying a `Representation`.
#[async_trait]
pub trait Queryable<R, W, C>: Representation<R, W, C>
where
    R: Read,
    C: ReadContext<R>,
{
    type Query: Query;

    async fn query(
        &self,
        ctx: &mut C,
        q: Self::Query,
    ) -> Result<<<Self as Queryable<R, W, C>>::Query as Query>::Ok, Error>;
}
