// inspired by https://github.com/tauri-apps/tauri/blob/2901145c497299f033ba7120af5f2e7ead16c75a/core/tauri-macros/src/command/handler.rs

use quote::quote;
use syn::{parse_macro_input, FnArg, ItemFn, Visibility};

use crate::utils::format_fn_wrapper;

pub fn attribute(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    if !cfg!(feature = "function") {
        panic!("Please enable the 'function' feature on the Specta crate to work with Functions.");
    }

    let function = parse_macro_input!(item as ItemFn);
    let wrapper = format_fn_wrapper(&function.sig.ident);

    let visibility = &function.vis;
    let maybe_macro_export = match &visibility {
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

        // allow the macro to be resolved with the same path as the function
        #[allow(unused_imports)]
        #visibility use #wrapper;
    }
    .into()
}
