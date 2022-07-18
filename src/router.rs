use std::{
    any::{Any, TypeId},
    collections::BTreeSet,
    fs,
    io::Write,
    marker::PhantomData,
    path::{Path, PathBuf},
    pin::Pin,
};

use futures::Stream;
use serde_json::Value;

use crate::{
    ConcreteArg, ExecError, Key, KeyDefinition, Operation, SubscriptionOperation, TSDependency,
};

/// TODO
pub struct Router<
    TCtx = (),
    TMeta = (),
    TQueryKey = &'static str,
    TMutationKey = &'static str,
    TSubscriptionKey = &'static str,
    TRootCtx = (),
> where
    TCtx: Send + Sync + 'static,
    TMeta: Send + Sync + 'static,
    TQueryKey: KeyDefinition,
    TMutationKey: KeyDefinition,
    TSubscriptionKey: KeyDefinition,
    TRootCtx: Send + Sync + 'static,
{
    pub(crate) query: Operation<TQueryKey, TCtx>,
    pub(crate) mutation: Operation<TMutationKey, TCtx>,
    pub(crate) subscription: SubscriptionOperation<TSubscriptionKey, TRootCtx>,
    pub(crate) phantom: PhantomData<TMeta>,
}

impl<TCtx, TMeta, TQueryKey, TMutationKey, TSubscriptionKey, TRootCtx>
    Router<TCtx, TMeta, TQueryKey, TMutationKey, TSubscriptionKey, TRootCtx>
where
    TCtx: Send + Sync + 'static,
    TMeta: Send + Sync + 'static,
    TQueryKey: KeyDefinition,
    TMutationKey: KeyDefinition,
    TSubscriptionKey: KeyDefinition,
    TRootCtx: Send + Sync + 'static,
{
    pub async fn exec_query<TArg, TKey>(
        &self,
        ctx: TCtx,
        key: TKey,
        arg: TArg,
    ) -> Result<Value, ExecError>
    where
        TArg: Send + Sync + 'static,
        TKey: Key<TQueryKey, TArg>,
    {
        let definition = self
            .query
            .get(key.to_val())
            .ok_or(ExecError::OperationNotFound)?;
        let arg = match TypeId::of::<TArg>() == TypeId::of::<Value>() {
            // SAFETY: We check the TypeId's match before `transmute_copy`. We use this method as I couldn't implement a trait which wouldn't overlap to abstract this into.
            true => {
                // We are using runtime specialization because I could not come up with a trait which wouldn't overlap to abstract this into.
                let v = (&mut Some(arg) as &mut dyn Any)
                    .downcast_mut::<Option<Value>>()
                    .unwrap()
                    .take()
                    .unwrap();
                ConcreteArg::Value(v)
            }
            false => ConcreteArg::Unknown(Box::new(arg)),
        };

        definition(ctx, arg)?.await
    }

    #[allow(dead_code)]
    pub(crate) async fn exec_query_unsafe(
        &self,
        ctx: TCtx,
        key: String,
        arg: Value,
    ) -> Result<Value, ExecError> {
        let definition = self
            .query
            .get(TQueryKey::from_str(key)?)
            .ok_or(ExecError::OperationNotFound)?;
        definition(ctx, ConcreteArg::Value(arg))?.await
    }

    pub async fn exec_mutation<TArg, TKey>(
        &self,
        ctx: TCtx,
        key: TKey,
        arg: TArg,
    ) -> Result<Value, ExecError>
    where
        TArg: Send + Sync + 'static,
        TKey: Key<TMutationKey, TArg>,
    {
        let definition = self
            .mutation
            .get(key.to_val())
            .ok_or(ExecError::OperationNotFound)?;
        let arg = match TypeId::of::<TArg>() == TypeId::of::<Value>() {
            true => {
                // We are using runtime specialization because I could not come up with a trait which wouldn't overlap to abstract this into.
                let v = (&mut Some(arg) as &mut dyn Any)
                    .downcast_mut::<Option<Value>>()
                    .unwrap()
                    .take()
                    .unwrap();
                ConcreteArg::Value(v)
            }
            false => ConcreteArg::Unknown(Box::new(arg)),
        };

        definition(ctx, arg)?.await
    }

    pub async fn exec_subscription<TArg, TKey>(
        &self,
        ctx: TRootCtx,
        key: TKey,
        arg: TArg,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<Value, ExecError>> + Send>>, ExecError>
    where
        TArg: Send + Sync + 'static,
        TKey: Key<TSubscriptionKey, TArg>,
    {
        let definition = self
            .subscription
            .get(key.to_val())
            .ok_or(ExecError::OperationNotFound)?;
        let arg = match TypeId::of::<TArg>() == TypeId::of::<Value>() {
            true => {
                // We are using runtime specialization because I could not come up with a trait which wouldn't overlap to abstract this into.
                let v = (&mut Some(arg) as &mut dyn Any)
                    .downcast_mut::<Option<Value>>()
                    .unwrap()
                    .take()
                    .unwrap();
                ConcreteArg::Value(v)
            }
            false => ConcreteArg::Unknown(Box::new(arg)),
        };

        Ok(definition(ctx, arg)?)
    }

    #[allow(dead_code)]
    pub(crate) async fn exec_mutation_unsafe(
        &self,
        ctx: TCtx,
        key: String,
        arg: Value,
    ) -> Result<Value, ExecError> {
        let definition = self
            .mutation
            .get(TMutationKey::from_str(key)?)
            .ok_or(ExecError::OperationNotFound)?;
        definition(ctx, ConcreteArg::Value(arg))?.await
    }

    #[allow(dead_code)]
    pub(crate) async fn exec_subscription_unsafe(
        &self,
        ctx: TRootCtx,
        key: String,
        arg: Value,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<Value, ExecError>> + Send>>, ExecError> {
        let definition = self
            .subscription
            .get(TSubscriptionKey::from_str(key)?)
            .ok_or(ExecError::OperationNotFound)?;

        Ok(definition(ctx, ConcreteArg::Value(arg))?)
    }

    // TODO: Don't use `Box<Error>` as return type.
    pub fn export_ts<TPath: AsRef<Path>>(
        &self,
        export_path: TPath,
        // TODO: New error type
    ) -> Result<(), Box<dyn std::error::Error>> {
        let export_path = PathBuf::from(export_path.as_ref());

        fs::create_dir_all(&export_path)?;
        let file_path = export_path.clone().join("index.ts");

        let header = vec![
            "/* --------------------------------------------------------------- */",
            "/* AUTOGENERATED BY [rspc](https://github.com/oscartbeaumont/rspc) */",
            "/*          [DO NOT EDIT THE FOLLOWING SECTION MANUALLY]           */",
            "/* --------------------------------------------------------------- */",
        ]
        .join("\n");
        let footer = vec![
            "/* --------------------------------------------------------------- */",
            "/*                [YOU CAN EDIT AFTER THIS LINE]                   */",
            "/* --------------------------------------------------------------- */",
        ]
        .join("\n");

        let (mut lines, begin_marker) = {
            if file_path.exists() {
                let content = fs::read_to_string(&file_path).unwrap();
                let mut lines = content
                    .split('\n')
                    .map(ToString::to_string)
                    .collect::<Vec<String>>();

                if lines.len() == 1 && lines[0].is_empty() {
                    (vec![], None)
                } else {
                    let begin_marker = lines
                        .iter()
                        .position(|line| {
                            line.contains("[DO NOT EDIT THE FOLLOWING SECTION MANUALLY]")
                        })
                        .unwrap();
                    let end_marker = lines
                        .iter()
                        .position(|line| line.contains("[YOU CAN EDIT AFTER THIS LINE]"))
                        .unwrap();

                    lines.drain((begin_marker - 2)..(end_marker + 2));
                    (lines, Some(begin_marker - 2))
                }
            } else {
                (Default::default(), None)
            }
        };

        let mut buf = vec![];

        writeln!(buf, "\nimport * as rspc from '@rspc/client'")?;

        let mut dependencies = BTreeSet::<TSDependency>::new();

        let mut query_buf = Vec::new();
        self.query
            .export_ts(&mut dependencies, &mut query_buf, export_path.clone())?;

        let mut mutation_buf = Vec::new();
        self.mutation
            .export_ts(&mut dependencies, &mut mutation_buf, export_path.clone())?;

        let mut subscription_buf = Vec::new();
        self.subscription
            .export_ts(&mut dependencies, &mut subscription_buf, export_path)?;

        for dep in dependencies.into_iter() {
            writeln!(
                buf,
                "import type {{ {} }} from {:?};",
                dep.ts_name.clone(),
                format!("./{}", dep.ts_name)
            )?;
        }

        writeln!(
            buf,
            "\nexport interface Operations {{ queries: Queries, mutations: Mutations, subscriptions: Subscriptions }}"
        )?;

        write!(buf, "\nexport type Queries =")?;
        buf.write_all(&query_buf)?;
        writeln!(buf, ";")?;

        write!(buf, "\nexport type Mutations =")?;
        buf.write_all(&mutation_buf)?;
        writeln!(buf, ";")?;

        write!(buf, "\nexport type Subscriptions =")?;
        buf.write_all(&subscription_buf)?;
        writeln!(buf, ";")?;

        writeln!(
            buf,
            "\nexport type TransportKind = \"fetch\" | \"websocket\";"
        )?;

        writeln!(
            buf,
            "\n\n{}",
            vec![
                "/** create new rspc client with address and kind",
                " * @param {string} address - rspc enspoint address",
                " * @param {TransportKind} kind - {@link TransportKind} to use",
                " * */"
            ]
            .join("\n")
        )?;
        writeln!(
            buf,
            "{}",
            vec![
                "export const createClient = (kind: TransportKind, address: string) =>",
                "  rspc.createClient<Operations>({",
                "    transport:",
                "      kind === \"fetch\"",
                "        ? new rspc.FetchTransport(address)",
                "        : new rspc.WebsocketTransport(address)",
                "  });"
            ]
            .join("\n")
        )?;

        if let Some(marker) = begin_marker {
            lines.insert(marker, header);
            lines.insert(marker + 1, String::from_utf8(buf)?);
            lines.insert(marker + 2, footer);
        } else {
            lines.push(header);
            lines.push(String::from_utf8(buf)?);
            lines.push(footer);
        }

        fs::write(&file_path, lines.join("\n"))?;

        Ok(())
    }
}
