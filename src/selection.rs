#[macro_export]
macro_rules! selection {
    ( $s:expr, { $($n:ident),+ } ) => {{
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

#[macro_export]
macro_rules! selection_vec {
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

        $s.into_iter().map(|v| Selection { $($n: v.$n,)* }).collect::<Vec<_>>()
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
        let s2 = selection_vec!(users, { name, age });
        assert_eq!(s2[0].name, "Monty Beaumont".to_string());
        assert_eq!(s2[0].age, 7);
    }
}
