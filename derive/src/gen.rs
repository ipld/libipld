use proc_macro2::TokenStream;
use quote::quote;
use syn::{Data, Field, Fields};
use synstructure::{BindingInfo, Structure, VariantInfo};

fn field_key(i: usize, field: &Field) -> String {
    field
        .ident
        .as_ref()
        .map(|ident| ident.to_string())
        .unwrap_or_else(|| i.to_string())
}

pub enum VariantRepr {
    Keyed,
    Kinded,
}

pub enum BindingRepr {
    Map,
    List,
}

impl BindingRepr {
    pub fn from_variant(variant: &VariantInfo) -> Self {
        match variant.ast().fields {
            Fields::Named(_) => Self::Map,
            Fields::Unnamed(_) => Self::List,
            Fields::Unit => Self::List,
        }
    }

    pub fn repr(&self, bindings: &[BindingInfo]) -> TokenStream {
        match self {
            Self::Map => {
                let fields = bindings.iter().enumerate().map(|(i, binding)| {
                    let key = field_key(i, binding.ast());
                    quote!(map.insert(IpldKey::from(#key), IpldRef::from(#binding));)
                });
                quote!({
                    let mut map = BTreeMap::new();
                    #(#fields)*
                    IpldRef::OwnedMap(map)
                })
            }
            Self::List => {
                let len = bindings.len();
                let fields = bindings
                    .iter()
                    .map(|binding| quote!(list.push(IpldRef::from(#binding));));
                quote!({
                    let mut list = Vec::with_capacity(#len);
                    #(#fields)*
                    IpldRef::OwnedList(list)
                })
            }
        }
    }

    pub fn parse(&self, variant: &VariantInfo) -> TokenStream {
        match self {
            Self::Map => {
                let construct = variant.construct(|field, i| {
                    let key = field_key(i, field);
                    quote!({
                        if let Some(ipld) = map.remove(&#key.into()) {
                            ipld.try_into()?
                        } else {
                            return Err(IpldError::KeyNotFound);
                        }
                    })
                });
                quote! {
                    if let Ipld::Map(ref mut map) = ipld {
                        Ok(#construct)
                    } else {
                        Err(IpldError::NotMap)
                    }
                }
            }
            Self::List => {
                let len = variant.bindings().len();
                let construct = variant.construct(|_field, i| {
                    quote! {
                        list[#i].clone().try_into()?
                    }
                });
                quote! {
                    if let Ipld::List(list) = ipld {
                        if list.len() != #len {
                            return Err(IpldError::IndexNotFound);
                        }
                        Ok(#construct)
                    } else {
                        Err(IpldError::NotList)
                    }
                }
            }
        }
    }
}

impl VariantRepr {
    pub fn from_structure(s: &Structure) -> Self {
        match &s.ast().data {
            Data::Struct(_) => Self::Kinded,
            Data::Enum(_) => Self::Keyed,
            Data::Union(_) => panic!("unsupported"),
        }
    }

    pub fn repr(&self, variant: &VariantInfo) -> TokenStream {
        let binding = BindingRepr::from_variant(variant);
        let bindings = binding.repr(variant.bindings());
        match self {
            Self::Keyed => {
                let name = variant.ast().ident.to_string();
                quote! {
                    let mut map = BTreeMap::new();
                    map.insert(IpldKey::from(#name), #bindings);
                    IpldRef::OwnedMap(map)
                }
            }
            Self::Kinded => quote!(#bindings),
        }
    }

    pub fn parse(&self, variant: &VariantInfo) -> TokenStream {
        let binding = BindingRepr::from_variant(variant);
        let bindings = binding.parse(variant);
        match self {
            Self::Keyed => {
                let name = variant.ast().ident.to_string();
                quote! {
                    if let Some(ipld) = map.get_mut(&#name.into()) {
                        return {#bindings};
                    }
                }
            }
            Self::Kinded => bindings,
        }
    }
}

pub fn to_ipld(s: &Structure) -> TokenStream {
    let var_repr = VariantRepr::from_structure(s);
    let body = s.each_variant(|var| var_repr.repr(var));

    quote! {
        fn to_ipld<'a>(&'a self) -> IpldRef<'a> {
            match *self {
                #body
            }
        }
    }
}

pub fn from_ipld(s: &Structure) -> TokenStream {
    let var_repr = VariantRepr::from_structure(s);
    let variants: Vec<TokenStream> = s.variants().iter().map(|var| var_repr.parse(var)).collect();
    let body = match var_repr {
        VariantRepr::Keyed => {
            quote! {
                let map = if let Ipld::Map(ref mut map) = ipld {
                    map
                } else {
                    return Err(IpldError::NotMap);
                };
                #(#variants)*
                Err(IpldError::KeyNotFound)
            }
        }
        VariantRepr::Kinded => quote!(#(#variants)*),
    };

    quote! {
       fn from_ipld(mut ipld: libipld::Ipld) -> Result<Self, IpldError> {
           #body
       }
    }
}
