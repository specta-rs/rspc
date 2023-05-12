#[macro_export]
macro_rules! selection {
    ( $s:expr, { $($n:ident),+ } ) => {{
        #[allow(non_camel_case_types)]
        mod selection {
            // Due to macro expansion order `$crate` can't go in the Specta attribute but when hardcoding `rspc` macro hygiene get's angry.
            // This hack is to avoid both issues.
            mod _rspc {
                pub use $crate::*;
            }

            #[derive(serde::Serialize, specta::Type)]
            #[specta(inline)]
            pub struct Selection<$($n,)*> {
                $(pub $n: $n),*
            }
        }
        use selection::Selection;
        #[allow(non_camel_case_types)]
        Selection { $($n: $s.$n,)* }
    }};
    ( $s:expr, [{ $($n:ident),+ }] ) => {{
        #[allow(non_camel_case_types)]
        mod selection {
            #[derive(serde::Serialize, specta::Type)]
            #[specta(inline)]
            pub struct Selection<$($n,)*> {
                $(pub $n: $n,)*
            }
        }
        use selection::Selection;
        #[allow(non_camel_case_types)]
        $s.into_iter().map(|v| Selection { $($n: v.$n,)* }).collect::<Vec<_>>()
    }};
}
