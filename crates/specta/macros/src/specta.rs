// inspired by https://github.com/tauri-apps/tauri/blob/2901145c497299f033ba7120af5f2e7ead16c75a/core/tauri-macros/src/command/handler.rs

use proc_macro2::Ident;
use quote::{format_ident, quote};
use syn::{parse_macro_input, FnArg, ItemFn, Visibility};

pub fn attribute(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    if !cfg!(feature = "function") {
        panic!("Please enable the 'function' feature on the Specta crate to work with Functions.");
    }

    let function = parse_macro_input!(item as ItemFn);
    let wrapper = format_command_wrapper(&function.sig.ident);

    let maybe_macro_export = match &function.vis {
        Visibility::Public(_) => quote!(#[macro_export]),
        _ => Default::default(),
    };

    let function_name = &function.sig.ident;

    let arg_names = function.sig.inputs.iter().map(|input| match input {
        FnArg::Receiver(_) => unreachable!("Commands cannot take 'self'"),
        FnArg::Typed(arg) => &arg.pat,
    });

    let arg_signatures = function.sig.inputs.iter().map(|_| quote!(_));

    quote! {
        #function

        #maybe_macro_export
        #[doc(hidden)]
        macro_rules! #wrapper {
            (@name) => { stringify!(#function_name) };
            (@arg_names) => { &[#(stringify!(#arg_names)),* ] };
            (@signature) => { fn(#(#arg_signatures),*) -> _ };
        }
    }
    .into()
}

fn format_command_wrapper(function: &Ident) -> Ident {
    format_ident!("__specta__{}", function)
}
