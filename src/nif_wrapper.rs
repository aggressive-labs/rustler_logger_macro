use proc_macro::TokenStream;
use proc_macro_tools::proc_macro2::Span;
use proc_macro_tools::syn::{
    AttributeArgs, Block, FnArg, Ident, Item, Pat, PatIdent, PatType, Type, TypePath, parse,
};
use proc_macro_tools::{Name, parse_macro_input, parse_quote};
use quote::{ToTokens, quote};

/// The attribute used to instrument functions for logging.
pub fn do_log_to_elixir(args: TokenStream, input: TokenStream) -> TokenStream {
    // first make sure we have a valid call (currently, no attributes)
    let args: AttributeArgs = parse_macro_input!(args);
    if !args.is_empty() {
        return quote! {
            compile_error!("log_to_elixir attribute requires no arguments");
        }
        .into();
    };
    // next, make sure we have a function
    let item: Item = parse_macro_input!(input);
    let fn_item = match item {
        Item::Fn(fn_item) => fn_item,
        _ => {
            return quote! {
                compile_error!("log_to_elixir attribute can only be applied to functions")
            }
            .into();
        }
    };

    // now, make sure we are used only once, in the right order, on a nif
    let mut found_nif = false;
    let mut found_log_to_elixir = false;
    for attr in fn_item.attrs.clone() {
        if attr.path.is_ident("nif") {
            found_nif = true;
        }
        if attr.path.is_ident("log_to_elixir") {
            found_log_to_elixir = true;
        }
    }
    let mut compiler_errors = Vec::new();
    if !found_nif {
        compiler_errors.push("log_to_elixir attribute must be applied before nif attribute");
    }
    if found_log_to_elixir {
        compiler_errors.push("log_to_elixir_attribute must only be used once");
    }
    if !compiler_errors.is_empty() {
        let error_text = compiler_errors.join("\n");
        return quote! {
            compile_error!(#error_text);
        }
        .into();
    }

    // find out if we have to inject an arg for the env, or what name is used if
    // it's already there
    let mut env_sym = Ident::new("env", Span::call_site());
    let mut inject_arg = true;

    if let Some(FnArg::Typed(PatType { pat, ty, .. })) = fn_item.sig.inputs.first()
        && let Type::Path(TypePath { path, .. }) = ty.as_ref()
        && path.segments.last().unwrap().ident == "Env"
    {
        if let Pat::Ident(PatIdent {
            ident: env_ident, ..
        }) = pat.as_ref()
        {
            if env_ident.to_string().starts_with("_") {
                return quote! {
                    compile_error!("don't explicitly request and Env and then ignore it")
                }
                .into();
            } else {
                env_sym = env_ident.clone();
                inject_arg = false;
            }
        } else {
            return quote! {
                compile_error!("first parameter of type Env, expected identifier")
            }
            .into();
        };
    }

    // alright, let's mutate the function signature
    let mut func = fn_item.to_owned();

    if inject_arg {
        func.sig.inputs.insert(0, parse_quote!(env: rustler::Env));
    }

    // and let's wrap the function body
    let fn_body = fn_item.block.to_token_stream();

    let fname = fn_item.name();
    let arity = fn_item.sig.inputs.len();

    let new_fn_body: Block = parse(
        quote!(
            {
                rustler_logger::_use_env(
                    #env_sym, ||{
                    let enif_result = std::panic::catch_unwind(|| #fn_body);
                    match enif_result {
                        Ok(result) => result,
                        Err(err) => {
                            rustler_logger::_send_panic_message(#fname, #arity as u32);

                            std::panic::resume_unwind(err);
                        }
                    }
                })
            }
        )
        .into(),
    )
    .unwrap();

    func.block = Box::new(new_fn_body);

    func.into_token_stream().into()
}
