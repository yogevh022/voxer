use syn::{Expr, Token};
use syn::parse::{Parse, ParseStream};

pub(crate) struct AttributeArg {
    pub(crate) key: syn::Ident,
    pub(crate) eq: Token![=],
    pub(crate) value: Expr,
}

impl Parse for AttributeArg {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(Self {
            key: input.parse()?,
            eq: input.parse()?,
            value: input.parse()?,
        })
    }
}
pub(crate) struct AttributeArgs(pub Vec<AttributeArg>);

impl Parse for AttributeArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut args = Vec::new();
        while !input.is_empty() {
            args.push(input.parse()?);
        }
        Ok(Self(args))
    }
}