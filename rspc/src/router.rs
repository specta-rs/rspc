use std::{
    borrow::{Borrow, Cow},
    collections::{BTreeMap, HashMap},
    fmt,
    panic::Location,
    sync::Arc,
};

use specta::TypeCollection;

use rspc_procedure::Procedures;

use crate::{
    modern::procedure::ErasedProcedure, types::TypesOrType, Procedure2, ProcedureKind, State, Types,
};

/// TODO: Examples exporting types and with `rspc_axum`
pub struct Router2<TCtx = ()> {
    setup: Vec<Box<dyn FnOnce(&mut State) + 'static>>,
    types: TypeCollection,
    procedures: BTreeMap<Vec<Cow<'static, str>>, ErasedProcedure<TCtx>>,
    errors: Vec<DuplicateProcedureKeyError>,
}

impl<TCtx> Default for Router2<TCtx> {
    fn default() -> Self {
        Self {
            setup: Default::default(),
            types: Default::default(),
            procedures: Default::default(),
            errors: vec![],
        }
    }
}

impl<TCtx> Router2<TCtx> {
    pub fn new() -> Self {
        Self::default()
    }

    #[cfg(feature = "unstable")]
    #[track_caller]
    pub fn procedure(
        mut self,
        key: impl Into<Cow<'static, str>>,
        procedure: impl Into<ErasedProcedure<TCtx>>,
    ) -> Self {
        let key = key.into();

        if let Some((_, original)) = self.procedures.iter().find(|(k, _)| k[0] == key) {
            self.errors.push(DuplicateProcedureKeyError {
                path: vec![key],
                original: original.ty.location,
                duplicate: Location::caller().clone(),
            });
        } else {
            let mut procedure = procedure.into();
            self.setup.extend(procedure.setup.drain(..));
            self.procedures.insert(vec![key], procedure);
        }

        self
    }

    // TODO: Document the order this is run in for `build`
    #[cfg(feature = "unstable")]
    pub fn setup(mut self, func: impl FnOnce(&mut State) + 'static) -> Self {
        self.setup.push(Box::new(func));
        self
    }

    #[track_caller]
    pub fn nest(mut self, prefix: impl Into<Cow<'static, str>>, mut other: Self) -> Self {
        let prefix = prefix.into();

        if let Some((_, original)) = self.procedures.iter().find(|(k, _)| k[0] == prefix) {
            self.errors.push(DuplicateProcedureKeyError {
                path: vec![prefix],
                original: original.ty.location,
                duplicate: Location::caller().clone(),
            });
        } else {
            self.setup.append(&mut other.setup);
            self.errors.extend(other.errors.into_iter().map(|e| {
                let mut path = vec![prefix.clone()];
                path.extend(e.path);
                DuplicateProcedureKeyError { path, ..e }
            }));
            self.procedures
                .extend(other.procedures.into_iter().map(|(k, v)| {
                    let mut key = vec![prefix.clone()];
                    key.extend(k);
                    (key, v)
                }));
        }

        self
    }

    #[track_caller]
    pub fn merge(mut self, mut other: Self) -> Self {
        let error_count = self.errors.len();

        for (k, original) in other.procedures.iter() {
            if let Some(new) = self.procedures.get(k) {
                self.errors.push(DuplicateProcedureKeyError {
                    path: k.clone(),
                    original: original.ty.location,
                    duplicate: new.ty.location,
                });
            }
        }

        if self.errors.len() > error_count {
            self.setup.append(&mut other.setup);
            self.procedures.extend(other.procedures.into_iter());
            self.errors.extend(other.errors);
        }

        self
    }

    pub fn build(self) -> Result<(Procedures<TCtx>, Types), Vec<DuplicateProcedureKeyError>> {
        self.build_with_state_inner(State::default())
    }

    // #[cfg(feature = "unstable")]
    // pub fn build_with_state(
    //     self,
    //     state: State,
    // ) -> Result<(Procedures<TCtx>, Types), Vec<DuplicateProcedureKeyError>> {
    //     self.build_with_state_inner(state)
    // }

    fn build_with_state_inner(
        self,
        mut state: State,
    ) -> Result<(Procedures<TCtx>, Types), Vec<DuplicateProcedureKeyError>> {
        if self.errors.len() > 0 {
            return Err(self.errors);
        }

        for setup in self.setup {
            setup(&mut state);
        }
        let state = Arc::new(state);

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

                (get_flattened_name(&key), (p.inner)(state.clone()))
            })
            .collect::<HashMap<_, _>>();

        Ok((
            Procedures::new(procedures, state),
            // TODO: Get rid of this and have `rspc-tracing` mount it
            // .with_logger(|event| println!("{event:?}")),
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
    type Item = (&'a Vec<Cow<'static, str>>, &'a ErasedProcedure<TCtx>);
    type IntoIter =
        std::collections::btree_map::Iter<'a, Vec<Cow<'static, str>>, ErasedProcedure<TCtx>>;

    fn into_iter(self) -> Self::IntoIter {
        self.procedures.iter()
    }
}

#[cfg(not(feature = "nolegacy"))]
impl<TCtx> From<crate::legacy::Router<TCtx>> for Router2<TCtx> {
    fn from(router: crate::legacy::Router<TCtx>) -> Self {
        crate::interop::legacy_to_modern(router)
    }
}

#[cfg(not(feature = "nolegacy"))]
impl<TCtx> Router2<TCtx> {
    pub(crate) fn interop_procedures(
        &mut self,
    ) -> &mut BTreeMap<Vec<Cow<'static, str>>, ErasedProcedure<TCtx>> {
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

pub struct DuplicateProcedureKeyError {
    path: Vec<Cow<'static, str>>,
    original: Location<'static>,
    duplicate: Location<'static>,
}

impl fmt::Debug for DuplicateProcedureKeyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(
            f,
            "Duplicate procedure at path {:?}. Original: {}:{}:{} Duplicate: {}:{}:{}",
            self.path,
            self.original.file(),
            self.original.line(),
            self.original.column(),
            self.duplicate.file(),
            self.duplicate.line(),
            self.duplicate.column()
        )
    }
}

impl fmt::Display for DuplicateProcedureKeyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}

impl std::error::Error for DuplicateProcedureKeyError {}
