// inspired by https://github.com/tauri-apps/tauri/blob/2901145c497299f033ba7120af5f2e7ead16c75a/core/tauri-macros/src/command/handler.rs

use proc_macro2::Ident;
use quote::{format_ident, quote};
use syn::{
    parse::{Parse, ParseBuffer},
    parse_macro_input, FnArg, ItemFn, Path, PathSegment, Token, Visibility,
};

pub fn command(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
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
    format_ident!("__specta__cmd__{}", function)
}

fn path_to_command(path: &mut Path) -> &mut PathSegment {
    path.segments
        .last_mut()
        .expect("parsed syn::Path has no segment")
}

pub struct Handler {
    paths: Vec<Path>,
    wrappers: Vec<Path>,
}

impl Parse for Handler {
    fn parse(input: &ParseBuffer<'_>) -> syn::Result<Self> {
        let paths = input.parse_terminated::<Path, Token![,]>(Path::parse)?;

        // parse the command names and wrappers from the passed paths
        let wrappers = paths
            .iter()
            .map(|path| {
                let mut wrapper = path.clone();
                let last = path_to_command(&mut wrapper);

                // the name of the actual command function
                let command = last.ident.clone();

                // set the path to the command function wrapper
                last.ident = format_command_wrapper(&command);

                wrapper
            })
            .collect();

        Ok(Self {
            paths: paths.into_iter().collect(), // remove punctuation separators
            wrappers,
        })
    }
}

impl From<Handler> for proc_macro::TokenStream {
    fn from(Handler { paths, wrappers }: Handler) -> Self {
        quote::quote!({
            let mut type_map = ::specta::TypeDefs::default();

            (
                vec![#(
                    ::specta::export_command_datatype(
                        #paths as #wrappers!(@signature),
                        #wrappers!(@name),
                        &mut type_map,
                        #wrappers!(@arg_names)
                    )
                ),*],
                type_map,
            )
        })
        .into()
    }
}
