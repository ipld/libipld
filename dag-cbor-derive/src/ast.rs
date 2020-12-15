use proc_macro2::Span;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SchemaType {
    Struct(Struct),
    Union(Union),
    Enum(Enum),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Struct {
    pub name: syn::Ident,
    pub fields: Vec<StructField>,
    pub repr: StructRepr,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StructField {
    pub name: syn::Member,
    pub rename: Option<String>,
    pub nullable: bool,
    pub optional: bool,
    pub implicit: Option<syn::Expr>,
    pub default: Option<syn::Expr>,
}

impl StructField {
    pub fn new(member: syn::Member) -> Self {
        Self {
            name: member,
            rename: None,
            nullable: false,
            optional: false,
            implicit: None,
            default: None,
        }
    }
}

impl From<syn::Ident> for StructField {
    fn from(ident: syn::Ident) -> Self {
        Self::new(syn::Member::Named(ident))
    }
}

impl From<usize> for StructField {
    fn from(i: usize) -> Self {
        Self::new(syn::Member::Unnamed(syn::Index {
            index: i as _,
            span: Span::call_site(),
        }))
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum StructRepr {
    Map,
    Tuple,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Union {
    pub name: syn::Ident,
    pub variants: Vec<Struct>,
    pub repr: UnionRepr,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum UnionRepr {
    Keyed,
    Kinded,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Enum {
    pub name: syn::Ident,
    pub values: Vec<EnumValue>,
    pub repr: EnumRepr,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EnumValue {
    pub name: syn::Ident,
    pub rename: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum EnumRepr {
    String,
    Int,
}
