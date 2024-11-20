use std::{
    borrow::Cow,
    collections::{BTreeMap, HashMap},
    fmt,
    path::{Path, PathBuf},
    sync::Arc,
};

use specta::{DataType, Language, Type, TypeMap};
use specta_util::TypeCollection;

use super::{
    procedure::{Procedure, ProcedureKind, UnbuiltProcedure},
    State,
};

pub struct Router<TCtx = ()> {
    setup: Vec<Box<dyn FnOnce(&mut State) + 'static>>,
    types: TypeCollection,
    procedures: BTreeMap<Cow<'static, str>, UnbuiltProcedure<TCtx>>,
    exports: Vec<Box<dyn FnOnce(TypeMap) -> Result<(), Box<dyn std::error::Error>>>>,
}

impl<TCtx> Default for Router<TCtx> {
    fn default() -> Self {
        Self {
            setup: Default::default(),
            types: Default::default(),
            procedures: Default::default(),
            exports: Default::default(),
        }
    }
}

impl<TCtx> fmt::Debug for Router<TCtx> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("Router").field(&self.procedures).finish()
    }
}

impl<TCtx> Router<TCtx> {
    pub fn new() -> Router<TCtx> {
        Self::default()
    }

    pub fn procedure(
        mut self,
        key: impl Into<Cow<'static, str>>,
        procedure: UnbuiltProcedure<TCtx>,
    ) -> Self {
        let name = key.into();
        self.procedures.insert(name, procedure);
        self
    }

    pub fn merge(mut self, prefix: impl Into<Cow<'static, str>>, mut other: Self) -> Self {
        self.setup.append(&mut other.setup);

        let prefix = prefix.into();
        let prefix = if prefix.is_empty() {
            Cow::Borrowed("")
        } else {
            format!("{prefix}.").into()
        };

        self.procedures.extend(
            other
                .procedures
                .into_iter()
                .map(|(k, v)| (format!("{prefix}{k}").into(), v)),
        );

        self
    }

    // TODO: Document the order this is run in for `build`
    pub fn setup(mut self, func: impl FnOnce(&mut State) + 'static) -> Self {
        self.setup.push(Box::new(func));
        self
    }

    pub fn ext(mut self, types: TypeCollection) -> Self {
        self.types = types;
        self
    }

    // TODO: Docs - that this is delayed until `Router::build` which means `Language::Error` type has to be erased.
    pub fn export_to(
        mut self,
        language: impl Language + 'static,
        path: impl Into<PathBuf>,
    ) -> Self {
        let path = path.into();
        self.exports.push(Box::new(move |types| {
            language
                .export(types)
                .and_then(|result| std::fs::write(path, result).map_err(Into::into))
                .map_err(Into::into)
        }));
        self
    }

    pub fn build(self) -> Result<BuiltRouter<TCtx>, ()> {
        self.build_with_state(State::default())
    }

    pub fn build_with_state(self, mut state: State) -> Result<BuiltRouter<TCtx>, ()> {
        // TODO: Return errors on duplicate procedure names or restricted names

        for setup in self.setup {
            setup(&mut state);
        }

        let mut type_map = TypeMap::default();
        self.types.collect(&mut type_map);
        let procedures: BTreeMap<Cow<'static, str>, _> = self
            .procedures
            .into_iter()
            .map(|(key, procedure)| (key.clone(), procedure.build(key, &mut state, &mut type_map)))
            .collect();

        {
            struct Procedure {
                kind: String,
                input: DataType,
                result: DataType,
                error: DataType,
            }

            enum ProcedureOrProcedures {
                Procedure(Procedure),
                Procedures(HashMap<Cow<'static, str>, ProcedureOrProcedures>),
            }

            impl Into<specta::DataType> for Procedure {
                fn into(self) -> specta::DataType {
                    specta::DataType::Struct(specta::internal::construct::r#struct(
                        "".into(),
                        None,
                        vec![],
                        specta::internal::construct::struct_named(
                            vec![
                                (
                                    "kind".into(),
                                    specta::internal::construct::field(
                                        false,
                                        false,
                                        None,
                                        Default::default(),
                                        Some(specta::DataType::Literal(
                                            specta::datatype::LiteralType::String(self.kind),
                                        )),
                                    ),
                                ),
                                (
                                    "input".into(),
                                    specta::internal::construct::field(
                                        false,
                                        false,
                                        None,
                                        Default::default(),
                                        Some(self.input),
                                    ),
                                ),
                                (
                                    "result".into(),
                                    specta::internal::construct::field(
                                        false,
                                        false,
                                        None,
                                        Default::default(),
                                        Some(self.result),
                                    ),
                                ),
                                (
                                    "error".into(),
                                    specta::internal::construct::field(
                                        false,
                                        false,
                                        None,
                                        Default::default(),
                                        Some(self.error),
                                    ),
                                ),
                            ],
                            None,
                        ),
                    ))
                }
            }

            impl Into<specta::DataType> for ProcedureOrProcedures {
                fn into(self) -> specta::DataType {
                    match self {
                        Self::Procedure(procedure) => procedure.into(),
                        Self::Procedures(procedures) => {
                            specta::DataType::Struct(specta::internal::construct::r#struct(
                                "".into(),
                                None,
                                vec![],
                                specta::internal::construct::struct_named(
                                    procedures
                                        .into_iter()
                                        .map(|(key, value)| {
                                            (
                                                key,
                                                specta::internal::construct::field(
                                                    false,
                                                    false,
                                                    None,
                                                    Default::default(),
                                                    Some(value.into()),
                                                ),
                                            )
                                        })
                                        .collect(),
                                    None,
                                ),
                            ))
                        }
                    }
                }
            }

            let mut types: HashMap<Cow<'static, str>, ProcedureOrProcedures> = Default::default();

            {
                for (key, procedure) in &procedures {
                    let mut procedures_map = &mut types;

                    let path = key.split(".").collect::<Vec<_>>();
                    let Some((key, path)) = path.split_last() else {
                        panic!("how is this empty");
                    };

                    for segment in path {
                        let ProcedureOrProcedures::Procedures(nested_procedures_map) =
                            procedures_map
                                .entry(segment.to_string().into())
                                .or_insert(ProcedureOrProcedures::Procedures(Default::default()))
                        else {
                            panic!();
                        };

                        procedures_map = nested_procedures_map;
                    }

                    procedures_map.insert(
                        key.to_string().into(),
                        ProcedureOrProcedures::Procedure(Procedure {
                            kind: match procedure.kind() {
                                ProcedureKind::Query => "query",
                                ProcedureKind::Mutation => "mutation",
                                ProcedureKind::Subscription => "subscription",
                            }
                            .to_string(),
                            input: procedure.ty().input.clone(),
                            result: procedure.ty().result.clone(),
                            error: DataType::Any,
                        }),
                    );
                }
            }

            #[derive(specta::Type)]
            struct Procedures;

            let mut named_type =
                <Procedures as specta::NamedType>::definition_named_data_type(&mut type_map);

            named_type.inner = ProcedureOrProcedures::Procedures(types).into();

            type_map.insert(<Procedures as specta::NamedType>::sid(), named_type);
        }

        // TODO: Customise the files header. It should says rspc not Specta!

        for export in self.exports {
            export(type_map.clone()).unwrap(); // TODO: Error
        }

        Ok(BuiltRouter {
            state: Arc::new(state),
            types: type_map,
            procedures,
        })
    }
}

#[derive(Clone)]
pub struct BuiltRouter<TCtx> {
    pub state: Arc<State>,
    pub types: TypeMap,
    pub procedures: BTreeMap<Cow<'static, str>, Procedure<TCtx>>,
}

impl<TCtx> fmt::Debug for BuiltRouter<TCtx> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("BuiltRouter")
            // TODO
            .finish()
    }
}

impl<TCtx> BuiltRouter<TCtx> {
    pub fn export<L: Language>(&self, language: L) -> Result<String, L::Error> {
        language.export(self.types.clone())
    }

    pub fn export_to<L: Language>(
        &self,
        language: L,
        path: impl AsRef<Path>,
    ) -> Result<(), L::Error> {
        std::fs::write(path, self.export(language)?).map_err(Into::into)
    }
}
