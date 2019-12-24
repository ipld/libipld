use crate::{async_trait, BTreeMap, CborError, ReadCbor};
use crate::{
    encode::{write_null, write_u64},
    Error, Read, ReadContext, RecursiveContext, Representation, Write, WriteContext,
};

#[async_trait]
impl<R, W, C, T> Representation<R, W, C> for Option<T>
where
    R: Read + Unpin + Send,
    W: Write + Unpin + Send,
    C: ReadContext<R> + WriteContext<W> + Send,
    T: Representation<R, W, C> + Sync,
{
    #[inline]
    async fn read(ctx: &mut C) -> Result<Self, Error>
    where
        R: 'async_trait,
        W: 'async_trait,
        C: 'async_trait,
    {
        match u8::read(ctx).await? {
            0xf6 => Ok(None),
            0xf7 => Ok(None),
            _ => match T::read(ctx).await {
                Ok(t) => Ok(Some(t)),
                Err(Error::Cbor(CborError::UnexpectedCode)) => Ok(None),
                Err(err) => Err(err),
            },
        }
    }

    #[inline]
    async fn write(&self, ctx: &mut C) -> Result<(), Error>
    where
        R: 'async_trait,
        W: 'async_trait,
        C: 'async_trait,
    {
        match self {
            Some(value) => T::write(value, ctx).await,
            None => {
                write_null(ctx.writer()).await?;
                Ok(())
            }
        }
    }
}

#[async_trait]
impl<R, W, C, T> Representation<R, W, C> for Vec<T>
where
    R: Read + Unpin + Send,
    W: Write + Unpin + Send,
    C: ReadContext<R> + WriteContext<W> + RecursiveContext + Send,
    T: Representation<R, W, C> + Send + Sync,
{
    #[inline]
    default async fn read(ctx: &mut C) -> Result<Self, Error>
    where
        R: 'async_trait,
        W: 'async_trait,
        C: 'async_trait,
    {
        let major = u8::read_cbor(ctx.reader()).await?;
        let len = read_list_len(ctx.reader(), major).await?;
        let mut list: Self = Vec::with_capacity(len);
        for idx in 0..len {
            ctx.push(idx);
            list.push(T::read(ctx).await?);
            ctx.pop();
        }
        Ok(list)
    }

    #[inline]
    default async fn write(&self, ctx: &mut C) -> Result<(), Error>
    where
        R: 'async_trait,
        W: 'async_trait,
        C: 'async_trait,
    {
        write_u64(ctx.writer(), 4, self.len() as u64).await?;
        for (idx, value) in self.iter().enumerate() {
            ctx.push(idx);
            T::write(value, ctx).await?;
            ctx.pop();
        }
        Ok(())
    }
}

// TODO: why 'static?
#[async_trait]
impl<R, W, C, T> Representation<R, W, C> for BTreeMap<String, T>
where
    R: Read + Unpin + Send,
    W: Write + Unpin + Send,
    C: ReadContext<R> + WriteContext<W> + RecursiveContext + Send,
    T: 'static + Representation<R, W, C> + Send + Sync,
{
    ///
    default async fn read(ctx: &mut C) -> Result<Self, Error>
    where
        R: 'async_trait,
        W: 'async_trait,
        C: 'async_trait,
    {
        let major = u8::read(ctx).await?;
        let len = read_map_len(ctx.reader(), major).await?;
        let mut map: Self = BTreeMap::new();
        for _ in 0..len {
            let key = String::read(ctx).await?;
            ctx.push(format!("{}", key));
            map.insert(key, T::read(ctx).await?);
            ctx.pop();
        }
        Ok(map)
    }

    ///
    default async fn write(&self, ctx: &mut C) -> Result<(), Error>
    where
        R: 'async_trait,
        W: 'async_trait,
        C: 'async_trait,
    {
        write_u64(ctx.writer(), 5, self.len() as u64).await?;
        for (key, value) in self {
            String::write(key, ctx).await?;
            ctx.push(format!("{}", key));
            T::write(value, ctx).await?;
            ctx.pop();
        }
        Ok(())
    }
}

#[inline]
pub(crate) async fn read_list_len<R>(r: &mut R, major: u8) -> Result<usize, Error>
where
    R: Read + Unpin + Send,
{
    let len = match major {
        0x80..=0x97 => major as usize - 0x80,
        0x98 => u8::read_cbor(r).await? as usize,
        0x99 => u16::read_cbor(r).await? as usize,
        0x9a => u32::read_cbor(r).await? as usize,
        0x9b => {
            let len = u64::read_cbor(r).await?;
            if len > usize::max_value() as u64 {
                return Err(Error::Cbor(CborError::LengthOutOfRange));
            }
            len as usize
        }
        _ => return Err(Error::Cbor(CborError::UnexpectedCode)),
    };
    Ok(len)
}

#[inline]
pub(crate) async fn read_map_len<R>(r: &mut R, major: u8) -> Result<usize, Error>
where
    R: Read + Unpin + Send,
{
    let len = match major {
        0xa0..=0xb7 => major as usize - 0xa0,
        0xb8 => u8::read_cbor(r).await? as usize,
        0xb9 => u16::read_cbor(r).await? as usize,
        0xba => u32::read_cbor(r).await? as usize,
        0xbb => {
            let len = u64::read_cbor(r).await?;
            if len > usize::max_value() as u64 {
                return Err(Error::Cbor(CborError::LengthOutOfRange));
            }
            len as usize
        }
        _ => return Err(Error::Cbor(CborError::UnexpectedCode)),
    };
    Ok(len)
}
