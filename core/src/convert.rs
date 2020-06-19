//! Conversion to and from ipld.
use crate::cid::Cid;
use crate::ipld::Ipld;
use crate::multihash::MultihashCode;
use std::collections::BTreeMap;
use std::convert::TryFrom;

macro_rules! derive_to_ipld_prim {
    ($enum:ident, $ty:ty, $fn:ident) => {
        impl<C, H> From<$ty> for Ipld<C, H>
        where
            C: Into<u64> + TryFrom<u64> + Copy,
            H: MultihashCode,
        {
            fn from(t: $ty) -> Self {
                Ipld::$enum(t.$fn() as _)
            }
        }
    };
}

macro_rules! derive_to_ipld {
    ($enum:ident, $ty:ty, $($fn:ident),*) => {
        impl<C, H> From<$ty> for Ipld<C, H>
        where
            C: Into<u64> + TryFrom<u64> + Copy,
            H: MultihashCode,
        {
            fn from(t: $ty) -> Self {
                Ipld::$enum(t$(.$fn())*)
            }
        }
    };
}

macro_rules! derive_to_ipld_generic {
   ($enum:ident, $ty:ty, $($fn:ident),*) => {
       impl<C, H> From<$ty> for Ipld<C, H>
       where
           C: Into<u64> + TryFrom<u64> + Copy,
           H: MultihashCode,
       {
           fn from(t: $ty) -> Self {
               Ipld::$enum(t$(.$fn())*)
           }
       }
   };
}

derive_to_ipld!(Bool, bool, clone);
derive_to_ipld_prim!(Integer, i8, clone);
derive_to_ipld_prim!(Integer, i16, clone);
derive_to_ipld_prim!(Integer, i32, clone);
derive_to_ipld_prim!(Integer, i64, clone);
derive_to_ipld_prim!(Integer, i128, clone);
derive_to_ipld_prim!(Integer, isize, clone);
derive_to_ipld_prim!(Integer, u8, clone);
derive_to_ipld_prim!(Integer, u16, clone);
derive_to_ipld_prim!(Integer, u32, clone);
derive_to_ipld_prim!(Integer, u64, clone);
derive_to_ipld_prim!(Integer, usize, clone);
derive_to_ipld_prim!(Float, f32, clone);
derive_to_ipld_prim!(Float, f64, clone);
derive_to_ipld!(String, String, into);
derive_to_ipld!(String, &str, to_string);
derive_to_ipld!(Bytes, Box<[u8]>, into_vec);
derive_to_ipld!(Bytes, Vec<u8>, into);
derive_to_ipld!(Bytes, &[u8], to_vec);
derive_to_ipld!(List, Vec<Ipld<C, H>>, into);
derive_to_ipld!(Map, BTreeMap<String, Ipld<C, H>>, to_owned);
derive_to_ipld_generic!(Link, Cid<C, H>, clone);
derive_to_ipld_generic!(Link, &Cid<C, H>, to_owned);
