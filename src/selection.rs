#[macro_export]
macro_rules! selection {
    ( $s:ident, { $($n:ident),+ } ) => {{
        #[allow(non_camel_case_types)]
        #[derive(serde::Serialize)]
        struct Selection<$($n,)*> {
            $($n: $n,)*
        }

        impl<$($n: ts_rs::TS + 'static,)*> ts_rs::TS for Selection<$($n,)*> {
            const EXPORT_TO: Option<&'static str> = None;

            fn name() -> String {
                format!("Selection")
            }
            fn dependencies() -> Vec<ts_rs::Dependency> {
                vec![
                    $(ts_rs::Dependency::from_ty::<$n>()),*
                ]
                    .into_iter()
                    .filter_map(|v| v)
                    .collect()
            }

            fn transparent() -> bool {
                true
            }

            fn inline() -> String {
                format!(
                    "{{ {} }}",
                    vec![$((stringify!($n), $n::name())),*]
                        .into_iter()
                        .map(|(name, kind)| { format!("{}: {};", name, kind) })
                        .collect::<Vec<_>>()
                        .join(" ")
                )
            }


        }

        Selection { $($n: $s.$n,)* }
    }};
}
