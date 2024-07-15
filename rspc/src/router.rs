use std::{
    borrow::Cow,
    collections::BTreeMap,
    fmt,
    path::{Path, PathBuf},
};

use specta::{Language, TypeMap};
use specta_util::TypeCollection;

use crate::{
    procedure::{Procedure, UnbuiltProcedure},
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

        let mut types = TypeMap::default();
        self.types.collect(&mut types);
        let procedures = self
            .procedures
            .into_iter()
            .map(|(key, procedure)| (key.clone(), procedure.build(key, &mut state, &mut types)))
            .collect();

        // TODO: Customise the files header. It should says rspc not Specta!

        // TODO: Generate the massive `Procedures` types

        for export in self.exports {
            export(types.clone()).unwrap(); // TODO: Error
        }

        Ok(BuiltRouter {
            state,
            types,
            procedures,
        })
    }
}

pub struct BuiltRouter<TCtx> {
    pub state: State,
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
