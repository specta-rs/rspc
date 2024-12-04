use std::{borrow::Cow, collections::BTreeMap, fmt};

use specta::{datatype::DataType, NamedType, SpectaID, Type, TypeMap};

use rspc_core::Procedures;

use crate::{internal::ProcedureKind, Procedure2, State};

/// TODO: Examples exporting types and with `rspc_axum`
pub struct Router2<TCtx = ()> {
    setup: Vec<Box<dyn FnOnce(&mut State) + 'static>>,
    types: TypeMap,
    procedures: BTreeMap<Vec<Cow<'static, str>>, Procedure2<TCtx>>,
}

impl<TCtx> Default for Router2<TCtx> {
    fn default() -> Self {
        Self {
            setup: Default::default(),
            types: Default::default(),
            procedures: Default::default(),
        }
    }
}

impl<TCtx> Router2<TCtx> {
    pub fn new() -> Self {
        Self::default()
    }

    // TODO: Enforce unique across all methods (query, subscription, etc). Eg. `insert` should yield error if key already exists.
    // pub fn procedure(
    //     mut self,
    //     key: impl Into<Cow<'static, str>>,
    //     procedure: UnbuiltProcedure<TCtx>,
    // ) -> Self {
    //     let name = key.into();
    //     self.procedures.insert(name, procedure);
    //     self
    // }

    // TODO: Document the order this is run in for `build`
    // pub fn setup(mut self, func: impl FnOnce(&mut State) + 'static) -> Self {
    //     self.setup.push(Box::new(func));
    //     self
    // }

    pub fn nest(mut self, prefix: impl Into<Cow<'static, str>>, mut other: Self) -> Self {
        self.setup.append(&mut other.setup);

        let prefix = prefix.into();

        self.procedures
            .extend(other.procedures.into_iter().map(|(mut k, v)| {
                k.push(prefix.clone());
                (k, v)
            }));

        self
    }

    pub fn merge(mut self, mut other: Self) -> Self {
        self.setup.append(&mut other.setup);
        self.procedures.extend(other.procedures.into_iter());
        self
    }

    pub fn build(
        mut self,
    ) -> Result<(impl Into<Procedures<TCtx>> + Clone + fmt::Debug, TypeMap), ()> {
        let mut state = ();
        for setup in self.setup {
            setup(&mut state);
        }

        let (types, procedures): (Vec<_>, BTreeMap<_, _>) = self
            .procedures
            .into_iter()
            .map(|(key, p)| (construct_bindings_type(&key, &p), (key, p.inner)))
            .unzip();

        {
            #[derive(Type)]
            struct Procedures;

            let s = literal_object(
                "Procedures".into(),
                Some(Procedures::sid()),
                types.into_iter(),
            );
            let mut ndt = Procedures::definition_named_data_type(&mut self.types);
            ndt.inner = s.into();
            self.types.insert(Procedures::sid(), ndt);
        }

        struct Impl<TCtx>(Procedures<TCtx>);
        impl<TCtx> Into<Procedures<TCtx>> for Impl<TCtx> {
            fn into(self) -> Procedures<TCtx> {
                self.0
            }
        }
        impl<TCtx> Clone for Impl<TCtx> {
            fn clone(&self) -> Self {
                Self(self.0.clone())
            }
        }
        impl<TCtx> fmt::Debug for Impl<TCtx> {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "{:?}", self.0)
            }
        }

        Ok((Impl::<TCtx>(procedures), self.types))
    }
}

impl<TCtx> fmt::Debug for Router2<TCtx> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let procedure_keys = |kind: ProcedureKind| {
            self.procedures
                .iter()
                .filter(move |(_, p)| p.kind() == kind)
                .map(|(k, _)| k.join("::"))
                .collect::<Vec<_>>()
        };

        f.debug_struct("Router")
            .field("queries", &procedure_keys(ProcedureKind::Query))
            .field("mutations", &procedure_keys(ProcedureKind::Mutation))
            .field(
                "subscriptions",
                &procedure_keys(ProcedureKind::Subscription),
            )
            .finish()
    }
}

impl<'a, TCtx> IntoIterator for &'a Router2<TCtx> {
    type Item = (&'a Vec<Cow<'static, str>>, &'a Procedure2<TCtx>);
    type IntoIter = std::collections::btree_map::Iter<'a, Vec<Cow<'static, str>>, Procedure2<TCtx>>;

    fn into_iter(self) -> Self::IntoIter {
        self.procedures.iter()
    }
}

// TODO: Probally using `DataTypeFrom` stuff cause we shouldn't be using `specta::internal`
fn literal_object(
    name: Cow<'static, str>,
    sid: Option<SpectaID>,
    fields: impl Iterator<Item = (Cow<'static, str>, DataType)>,
) -> DataType {
    specta::internal::construct::r#struct(
        name,
        sid,
        Default::default(),
        specta::internal::construct::struct_named(
            fields
                .into_iter()
                .map(|(name, ty)| {
                    (
                        name.into(),
                        specta::internal::construct::field(false, false, None, "".into(), Some(ty)),
                    )
                })
                .collect(),
            None,
        ),
    )
    .into()
}

fn construct_bindings_type<TCtx>(
    key: &[Cow<'static, str>],
    p: &Procedure2<TCtx>,
) -> (Cow<'static, str>, DataType) {
    if key.len() == 1 {
        (
            key[0].clone(),
            literal_object(
                "".into(),
                None,
                vec![
                    ("input".into(), p.input.clone()),
                    ("result".into(), p.result.clone()),
                    ("error".into(), p.error.clone()),
                ]
                .into_iter(),
            ),
        )
    } else {
        (
            key[0].clone(),
            literal_object(
                "".into(),
                None,
                vec![construct_bindings_type(&key[1..], p)].into_iter(),
            ),
        )
    }
}

// TODO: Remove this block with the interop system
impl<TCtx> From<crate::legacy::Router<TCtx, ()>> for Router2<TCtx> {
    fn from(router: crate::legacy::Router<TCtx>) -> Self {
        crate::interop::legacy_to_modern(router)
    }
}

// TODO: Remove this block with the interop system
impl<TCtx> Router2<TCtx> {
    pub(crate) fn interop_procedures(
        &mut self,
    ) -> &mut BTreeMap<Vec<Cow<'static, str>>, Procedure2<TCtx>> {
        &mut self.procedures
    }

    pub(crate) fn interop_types(&mut self) -> &mut TypeMap {
        &mut self.types
    }
}
