use std::{
    borrow::Cow,
    collections::{BTreeMap, HashMap},
    fmt,
};

use specta::TypeCollection;

use rspc_core::Procedures;

use crate::{internal::ProcedureKind, types::TypesOrType, Procedure2, State, Types};

/// TODO: Examples exporting types and with `rspc_axum`
pub struct Router2<TCtx = ()> {
    setup: Vec<Box<dyn FnOnce(&mut State) + 'static>>,
    types: TypeCollection,
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
    //     mut procedure: Procedure2<TCtx>,
    // ) -> Self {
    //     self.setup.extend(procedure.setup.drain(..));
    //     self.procedures.insert(vec![key.into()], procedure);
    //     self
    // }

    // TODO: Document the order this is run in for `build`
    // pub fn setup(mut self, func: impl FnOnce(&mut State) + 'static) -> Self {
    //     self.setup.push(Box::new(func));
    //     self
    // }

    // TODO: Yield error if key already exists
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

    // TODO: Yield error if key already exists
    pub fn merge(mut self, mut other: Self) -> Self {
        self.setup.append(&mut other.setup);
        self.procedures.extend(other.procedures.into_iter());
        self
    }

    pub fn build(self) -> Result<(impl Into<Procedures<TCtx>> + Clone + fmt::Debug, Types), ()> {
        let mut state = ();
        for setup in self.setup {
            setup(&mut state);
        }

        let mut procedure_types = BTreeMap::new();
        let procedures = self
            .procedures
            .into_iter()
            .map(|(key, p)| {
                let mut current = &mut procedure_types;
                // TODO: if `key.len()` is `0` we might run into issues here. It shouldn't but probs worth protecting.
                for part in &key[..(key.len() - 1)] {
                    let a = current
                        .entry(part.clone())
                        .or_insert_with(|| TypesOrType::Types(Default::default()));
                    match a {
                        TypesOrType::Type(_) => unreachable!(), // TODO: Confirm this is unreachable
                        TypesOrType::Types(map) => current = map,
                    }
                }
                current.insert(key[key.len() - 1].clone(), TypesOrType::Type(p.ty));

                (get_flattened_name(&key), p.inner)
            })
            .collect::<HashMap<_, _>>();

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

        Ok((
            Impl::<TCtx>(procedures),
            Types {
                types: self.types,
                procedures: procedure_types,
            },
        ))
    }
}

impl<TCtx> fmt::Debug for Router2<TCtx> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let procedure_keys = |kind: ProcedureKind| {
            self.procedures
                .iter()
                .filter(move |(_, p)| p.ty.kind == kind)
                .map(|(k, _)| k.join("."))
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

// TODO: Remove this block with the interop system
impl<TCtx> From<crate::legacy::Router<TCtx>> for Router2<TCtx> {
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

    pub(crate) fn interop_types(&mut self) -> &mut TypeCollection {
        &mut self.types
    }
}

fn get_flattened_name(name: &Vec<Cow<'static, str>>) -> Cow<'static, str> {
    if name.len() == 1 {
        // By cloning we are ensuring we passthrough to the `Cow` to avoid cloning if this is a `&'static str`.
        // Doing `.join` will always produce a new `String` removing the `&'static str` optimization.
        name[0].clone()
    } else {
        name.join(".").to_string().into()
    }
}
