use crate::{async_trait, BTreeMap, CborError, ReadCbor};
use crate::{
    context::{PopElement, PushElement},
    encode::{write_null, write_u64},
    Context, Error, Read, Representation, Write,
};

#[async_trait]
impl<R, W, T> Representation<R, W> for Option<T>
where
    R: Read + Unpin + Send,
    W: Write + Unpin + Send,
    T: Representation<R, W> + Sync,
{
    #[inline]
    async fn read<C>(ctx: &mut C) -> Result<Self, Error>
    where
        R: 'async_trait,
        W: 'async_trait,
        C: Context<R, W> + Send,
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
    async fn write<C>(&self, ctx: &mut C) -> Result<(), Error>
    where
        R: 'async_trait,
        W: 'async_trait,
        C: Context<R, W> + Send,
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
impl<R, W, T> Representation<R, W> for Vec<T>
where
    R: Read + Unpin + Send,
    W: Write + Unpin + Send,
    T: Representation<R, W> + Send + Sync,
{
    #[inline]
    default async fn read<C>(ctx: &mut C) -> Result<Self, Error>
    where
        R: 'async_trait,
        W: 'async_trait,
        C: Context<R, W> + Send,
    {
        let major = u8::read_cbor(ctx.reader()).await?;
        let len = read_list_len(ctx.reader(), major).await?;
        let mut list: Self = Vec::with_capacity(len);
        for idx in 0..len {
            ctx.try_apply(PushElement::new(&idx));
            list.push(T::read(ctx).await?);
            ctx.try_apply(PopElement);
        }
        Ok(list)
    }

    #[inline]
    default async fn write<C>(&self, ctx: &mut C) -> Result<(), Error>
    where
        R: 'async_trait,
        W: 'async_trait,
        C: Context<R, W> + Send,
    {
        write_u64(ctx.writer(), 4, self.len() as u64).await?;
        for (idx, value) in self.iter().enumerate() {
            ctx.try_apply(PushElement::new(&idx));
            T::write(value, ctx).await?;
            ctx.try_apply(PopElement);
        }
        Ok(())
    }
}

// NOTE: why 'static?
#[async_trait]
impl<R, W, T> Representation<R, W> for BTreeMap<String, T>
where
    R: Read + Unpin + Send,
    W: Write + Unpin + Send,
    T: 'static + Representation<R, W> + Send + Sync,
{
    default async fn read<C>(ctx: &mut C) -> Result<Self, Error>
    where
        R: 'async_trait,
        W: 'async_trait,
        C: Context<R, W> + Send,
    {
        let major = u8::read(ctx).await?;
        let len = read_map_len(ctx.reader(), major).await?;
        let mut map: Self = BTreeMap::new();
        for _ in 0..len {
            let key = String::read(ctx).await?;
            ctx.try_apply(PushElement::new(&key));
            map.insert(key, T::read(ctx).await?);
            ctx.try_apply(PopElement);
        }
        Ok(map)
    }

    default async fn write<C>(&self, ctx: &mut C) -> Result<(), Error>
    where
        R: 'async_trait,
        W: 'async_trait,
        C: Context<R, W> + Send,
    {
        write_u64(ctx.writer(), 5, self.len() as u64).await?;
        for (key, value) in self {
            String::write(key, ctx).await?;
            ctx.try_apply(PushElement::new(&key.as_str()));
            T::write(value, ctx).await?;
            ctx.try_apply(PopElement);
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
