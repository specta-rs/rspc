#[macro_export]
macro_rules! selection {
    ( $s:expr, [{ $($n:ident),+ }] ) => {{
        #[derive(serde::Serialize)]
        #[allow(non_camel_case_types)]
        struct Selection<$($n,)*> {
            $($n: $n,)*
        }

        impl<$($n: ts_rs::TS + 'static,)*> ts_rs::TS for Selection<$($n,)*> {
            const EXPORT_TO: Option<&'static str> = None;

            fn name() -> String {
                format!("Selection")
            }

            fn dependencies(deps: &mut std::collections::HashMap<std::any::TypeId, ts_rs::Dependency>) {
                $(if !deps.contains_key(&std::any::TypeId::of::<$n>()) {
                    if let Some(dep) = ts_rs::Dependency::from_ty::<$n>() {
                        deps.insert(dep.type_id, dep);
                    }

                    $n::dependencies(deps);
                })*
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

        #[allow(non_camel_case_types)]
        $s.into_iter().map(|v| Selection { $($n: v.$n,)* }).collect::<Vec<_>>()
    }};
    ( $s:expr, { $($n:ident),+ } ) => {{
        #[derive(serde::Serialize)]
        #[allow(non_camel_case_types)]
        struct Selection<$($n,)*> {
            $($n: $n,)*
        }

        impl<$($n: ts_rs::TS + 'static,)*> ts_rs::TS for Selection<$($n,)*> {
            const EXPORT_TO: Option<&'static str> = None;

            fn name() -> String {
                format!("Selection")
            }

            fn dependencies(deps: &mut std::collections::HashMap<std::any::TypeId, ts_rs::Dependency>) {
                $(if !deps.contains_key(&std::any::TypeId::of::<$n>()) {
                    if let Some(dep) = ts_rs::Dependency::from_ty::<$n>() {
                        deps.insert(dep.type_id, dep);
                    }

                    $n::dependencies(deps);
                })*
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

        #[allow(non_camel_case_types)]
        Selection { $($n: $s.$n,)* }
    }};
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone)]
    struct User {
        pub id: i32,
        pub name: String,
        pub email: String,
        pub age: i32,
        pub password: String,
    }

    #[test]
    fn test_selection_macros() {
        let user = User {
            id: 1,
            name: "Monty Beaumont".into(),
            email: "monty@otbeaumont.me".into(),
            age: 7,
            password: "password123".into(),
        };

        let s1 = selection!(user.clone(), { name, age });
        assert_eq!(s1.name, "Monty Beaumont".to_string());
        assert_eq!(s1.age, 7);

        let users = vec![user; 3];
        let s2 = selection!(users, [{ name, age }]);
        assert_eq!(s2[0].name, "Monty Beaumont".to_string());
        assert_eq!(s2[0].age, 7);
    }
}
