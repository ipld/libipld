use proc_macro2::TokenStream;

#[derive(Clone, Debug)]
pub struct TokenStreamEq(pub TokenStream);

impl std::ops::Deref for TokenStreamEq {
    type Target = TokenStream;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl PartialEq for TokenStreamEq {
    fn eq(&self, other: &Self) -> bool {
        self.0.to_string() == other.0.to_string()
    }
}

impl Eq for TokenStreamEq {}
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
    pub pat: TokenStreamEq,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StructField {
    pub name: syn::Member,
    pub rename: Option<String>,
    pub nullable: bool,
    pub optional: bool,
    pub implicit: Option<syn::Expr>,
    pub default: Option<syn::Expr>,
    pub binding: syn::Ident,
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
    pub pat: TokenStreamEq,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum EnumRepr {
    String,
    Int,
}
