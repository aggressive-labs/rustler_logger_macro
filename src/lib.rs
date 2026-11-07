//! Macros to support [`rustler_logger`](https://github.com/aggressive-labs/rustler_logger).
use proc_macro::TokenStream;
use proc_macro_tools::syn::ExprPath;
use proc_macro_tools::{parse_macro_input, parse_quote};
use quote::quote;

mod arg;
mod call;
mod nif_wrapper;

use arg::ArgList;
use call::*;
use nif_wrapper::*;

/// wrapper for `nif` functions needed to set up logging to BEAM
///
/// ```rust
/// use rustler_logger::*;
///
/// #[log_to_elixir]
/// #[nif]
/// fn my_nif(){
///     info!("my NIF can log, because it's empowered by `log_to_elixir`")
/// }
/// ```
#[proc_macro_attribute]
pub fn log_to_elixir(args: TokenStream, input: TokenStream) -> TokenStream {
    do_log_to_elixir(args, input)
}

/// send a log entry to the BEAM VM
///
/// ```ignore
/// use rustler_logger::*;
///
/// log!(Info, "access denied (username=~p)", "alice")
/// ```
#[proc_macro]
pub fn log(args: TokenStream) -> TokenStream {
    let macro_args: ArgList = parse_macro_input!(args);
    let mut item_iter = macro_args.items.iter();

    let log_level = match item_iter.next() {
        Some(arg) => arg,
        None => {
            return quote! {
                compile_error!("log macro requires log level as first argument");
            }
            .into();
        }
    };
    parse_log_call(log_level, item_iter)
}

/// send a `trace` level log entry to the BEAM VM
///
/// ```ignore
/// use rustler_logger::*;
///
/// trace!("access denied (username=~p)", "alice")
/// ```
///
/// NOTE: Elixir doesn't have a "trace" level, so we use "debug" instead.
#[proc_macro]
pub fn trace(args: TokenStream) -> TokenStream {
    debug(args)
}

/// send a `debug` level log entry to the BEAM VM
///
/// ```ignore
/// use rustler_logger::*;
///
/// debug!("access denied (username=~p)", "alice")
/// ```
#[proc_macro]
pub fn debug(args: TokenStream) -> TokenStream {
    let macro_args: ArgList = parse_macro_input!(args);
    let item_iter = macro_args.items.iter();
    let log_level: ExprPath = parse_quote! { rustler_logger::LogLevel::Debug };
    let log_level_expr = parse_quote! { #log_level };
    parse_log_call(&log_level_expr, item_iter)
}

/// send an `info` level log entry to the BEAM VM
///
/// ```ignore
/// use rustler_logger::*;
///
/// info!("access denied (username=~p)", "alice")
/// ```
#[proc_macro]
pub fn info(args: TokenStream) -> TokenStream {
    let macro_args: ArgList = parse_macro_input!(args);
    let item_iter = macro_args.items.iter();
    let log_level: ExprPath = parse_quote! { rustler_logger::LogLevel::Info };
    let log_level_expr = parse_quote! { #log_level };
    parse_log_call(&log_level_expr, item_iter)
}

/// send a `notice` level log entry to the BEAM VM
///
/// ```ignore
/// use rustler_logger::*;
///
/// notice!("access denied (username=~p)", "alice")
/// ```
#[proc_macro]
pub fn notice(args: TokenStream) -> TokenStream {
    let macro_args: ArgList = parse_macro_input!(args);
    let item_iter = macro_args.items.iter();
    let log_level: ExprPath = parse_quote! { rustler_logger::LogLevel::Notice };
    let log_level_expr = parse_quote! { #log_level };
    parse_log_call(&log_level_expr, item_iter)
}

/// send a `warning` level log entry to the BEAM VM
///
/// ```ignore
/// use rustler_logger::*;
///
/// warn!("access denied (username=~p)", "alice")
/// ```
///
/// NOTE: The actual level used is `warning`. This is provided as a convenience.
#[proc_macro]
pub fn warn(args: TokenStream) -> TokenStream {
    warning(args)
}

/// send a `warning` level log entry to the BEAM VM
///
/// ```ignore
/// use rustler_logger::*;
///
/// warning!("access denied (username=~p)", "alice")
/// ```
#[proc_macro]
pub fn warning(args: TokenStream) -> TokenStream {
    let macro_args: ArgList = parse_macro_input!(args);
    let item_iter = macro_args.items.iter();
    let log_level: ExprPath = parse_quote! { rustler_logger::LogLevel::Warning };
    let log_level_expr = parse_quote! { #log_level };
    parse_log_call(&log_level_expr, item_iter)
}

/// send an `error` level log entry to the BEAM VM
///
/// ```ignore
/// use rustler_logger::*;
///
/// error!("access denied (username=~p)", "alice")
/// ```
#[proc_macro]
pub fn error(args: TokenStream) -> TokenStream {
    let macro_args: ArgList = parse_macro_input!(args);
    let item_iter = macro_args.items.iter();
    let log_level: ExprPath = parse_quote! { rustler_logger::LogLevel::Error };
    let log_level_expr = parse_quote! { #log_level };
    parse_log_call(&log_level_expr, item_iter)
}

/// send a `critical` level log entry to the BEAM VM
///
/// ```ignore
/// use rustler_logger::*;
///
/// critical!("access denied (username=~p)", "alice")
/// ```
#[proc_macro]
pub fn critical(args: TokenStream) -> TokenStream {
    let macro_args: ArgList = parse_macro_input!(args);
    let item_iter = macro_args.items.iter();
    let log_level: ExprPath = parse_quote! { rustler_logger::LogLevel::Critical };
    let log_level_expr = parse_quote! { #log_level };
    parse_log_call(&log_level_expr, item_iter)
}

/// send an `alert` level log entry to the BEAM VM
///
/// ```ignore
/// use rustler_logger::*;
///
/// alert!("access denied (username=~p)", "alice")
/// ```
#[proc_macro]
pub fn alert(args: TokenStream) -> TokenStream {
    let macro_args: ArgList = parse_macro_input!(args);
    let item_iter = macro_args.items.iter();
    let log_level: ExprPath = parse_quote! { rustler_logger::LogLevel::Alert };
    let log_level_expr = parse_quote! { #log_level };
    parse_log_call(&log_level_expr, item_iter)
}

/// send a `fatal` level log entry to the BEAM VM
///
/// ```ignore
/// use rustler_logger::*;
///
/// fatal!("access denied (username=~p)", "alice")
/// ```
///
/// NOTE: Elixir doesn't have a "fatal" level, so we use "emergency" instead.
#[proc_macro]
pub fn fatal(args: TokenStream) -> TokenStream {
    emergency(args)
}

/// send a `emergency` level log entry to the BEAM VM
///
/// ```ignore
/// use rustler_logger::*;
///
/// emergency!("access denied (username=~p)", "alice")
/// ```
#[proc_macro]
pub fn emergency(args: TokenStream) -> TokenStream {
    let macro_args: ArgList = parse_macro_input!(args);
    let item_iter = macro_args.items.iter();
    let log_level: ExprPath = parse_quote! { rustler_logger::LogLevel::Emergency };
    let log_level_expr = parse_quote! { #log_level };
    parse_log_call(&log_level_expr, item_iter)
}
