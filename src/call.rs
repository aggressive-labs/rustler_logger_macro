use proc_macro::TokenStream;
use proc_macro_tools::syn::{Expr, ExprAssign};

use quote::{ToTokens, quote};

pub fn parse_log_call<'a>(
    log_level: &'a Expr,
    mut item_iter: impl Iterator<Item = &'a Expr>,
) -> TokenStream {
    let log_format = match item_iter.next() {
        Some(arg) => arg,
        None => {
            return quote! {
                compile_error!("log macro requires format as second argument");
            }
            .into();
        }
    };

    let mut log_args = Vec::new();
    let mut log_meta = Vec::new();

    for item in item_iter {
        match *item {
            Expr::Assign(ExprAssign {
                ref left,
                ref right,
                ..
            }) => {
                let left_str = left.into_token_stream().to_string();
                let left = quote!(#left_str);
                let right = right.into_token_stream();
                log_meta.push(quote!(.meta(#left, #right)));
            }
            ref other_expr => {
                log_args.push(quote!(.arg(#other_expr)));
            }
        }
    }

    let output = quote! {
        rustler_logger::Log::new(#log_level, #log_format)
            .meta("file", file!())
            .meta("line", line!())
            .meta("column", column!())
            #(#log_args)*
            #(#log_meta)*
            .send()
    };

    output.into()
}
