use proc_macro_tools::syn::parse::{Parse, ParseStream, Result};
use proc_macro_tools::syn::punctuated::Punctuated;
use proc_macro_tools::syn::{Expr, Token};

#[doc(hidden)]
#[derive(Debug)]
pub struct ArgList {
    pub items: Punctuated<Expr, Token![,]>,
}

#[doc(hidden)]
impl Parse for ArgList {
    fn parse(input: ParseStream) -> Result<Self> {
        let items = Punctuated::parse_terminated(input)?;
        Ok(ArgList { items })
    }
}
