use std::{
    any::TypeId,
    borrow::Cow,
    collections::{BTreeMap, HashMap},
    sync::Arc,
};

pub use httpz::http::Method;
use httpz::{http::Response, Endpoint, GenericEndpoint, HttpEndpoint, Request};
use include_dir::{include_dir, Dir};
use mime_guess::mime;
use openapiv3::{
    Info, MediaType, Operation, Parameter, ParameterData, ParameterSchemaOrContent, PathItem,
    Paths, QueryStyle, ReferenceOr, Responses, Schema, Server, StatusCode,
};
use specta::openapi::to_openapi;

use crate::{
    integrations::httpz_extractors::{TCtxFunc, TCtxFuncResult},
    internal::{BuiltProcedureBuilder, ProcedureKind},
    ExecKind, Router,
};

static SWAGGER_UI: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/src/plugins/openapi/swagger-ui");

#[derive(Debug, Clone)]
pub struct OpenAPIEndpoint {
    pub procedure_name: &'static str,
    pub procedure_type: ExecKind,
    pub arg_schema: ReferenceOr<Schema>,
    pub result_schema: ReferenceOr<Schema>,
}

#[derive(Default)]
pub struct OpenAPIContext {
    endpoints: HashMap<(Method, &'static str /* OpenAPI URL */), OpenAPIEndpoint>,
}

#[derive(Default)]
pub struct OpenAPIConfig {
    pub title: Cow<'static, str>,
    // pub description: &'static str,
    pub version: Cow<'static, str>,
    pub base_url: Cow<'static, str>,
}

pub trait OpenAPI<TResolver> {
    fn openapi(self, method: Method, url: &'static str) -> BuiltProcedureBuilder<TResolver>;
}

impl<TResolver> OpenAPI<TResolver> for BuiltProcedureBuilder<TResolver> {
    #[allow(clippy::unwrap_used, clippy::panic)] // TODO: Remove once this feature is stable
    fn openapi(self, method: Method, url: &'static str) -> Self {
        {
            let mut data = self.data.write().unwrap();

            let ctx = data
                .entry(TypeId::of::<OpenAPIContext>())
                .or_insert_with(|| {
                    Box::<OpenAPIContext>::default()
                        as Box<dyn std::any::Any + Send + Sync + 'static>
                })
                .downcast_mut::<OpenAPIContext>()
                .unwrap();

            if ctx.endpoints.contains_key(&(method.clone(), url)) {
                panic!(
                    "OpenAPI endpoint already with method '{}' and url '{}'",
                    method, url,
                );
            }

            println!(
                "A {:?} {:?}",
                self.typedef.inline_input, self.typedef.inline_result
            );

            ctx.endpoints.insert(
                (method, url),
                OpenAPIEndpoint {
                    procedure_name: self.name,
                    procedure_type: match self.kind {
                        ProcedureKind::Query => ExecKind::Query,
                        ProcedureKind::Mutation => ExecKind::Mutation,
                        ProcedureKind::Subscription => panic!("TODO"),
                    },
                    arg_schema: to_openapi(&self.typedef.inline_input),
                    result_schema: to_openapi(&self.typedef.inline_result),
                },
            );
        }

        self
    }
}

impl<TCtx> Router<TCtx>
where
    TCtx: Send + Sync + 'static,
{
    #[allow(clippy::unwrap_used, clippy::panic)] // TODO: Remove lint override once this feature is stable
    pub fn openapi_doc(&self, config: OpenAPIConfig) -> openapiv3::OpenAPI {
        let mut data = self.data.write().unwrap();

        let ctx = data
            .entry(TypeId::of::<OpenAPIContext>())
            .or_insert_with(|| {
                Box::<OpenAPIContext>::default() as Box<dyn std::any::Any + Send + Sync + 'static>
            })
            .downcast_mut::<OpenAPIContext>()
            .unwrap();

        openapiv3::OpenAPI {
            openapi: "3.0.3".into(),
            info: Info {
                title: config.title.into(),
                // description: Some(config.description), // TODO
                version: config.version.into(),
                ..Default::default()
            },
            servers: vec![Server {
                url: config.base_url.into(),
                ..Default::default()
            }],
            paths: Paths {
                paths: ctx
                    .endpoints
                    .iter()
                    .map(
                        |(
                            (method, url),
                            OpenAPIEndpoint {
                                procedure_name,
                                procedure_type,
                                arg_schema,
                                result_schema,
                                ..
                            },
                        )| {
                            (
                                url.to_string(),
                                ReferenceOr::Item({
                                    let operation = Operation {
                                        operation_id: Some(format!(
                                            "{}-{}",
                                            match procedure_type {
                                                ExecKind::Query => "query",
                                                ExecKind::Mutation => "mutation",
                                            },
                                            procedure_name
                                        )),
                                        parameters: match procedure_type {
                                            ExecKind::Query => {
                                                vec![ReferenceOr::Item(Parameter::Query {
                                                    parameter_data: ParameterData {
                                                        name: "todo".to_string(), // TODO
                                                        description: None,        // TODO
                                                        required: true, // TODO: If null or not
                                                        deprecated: None, // TODO
                                                        format: ParameterSchemaOrContent::Schema(
                                                            arg_schema.clone(),
                                                        ),
                                                        example: None, // TODO
                                                        examples: Default::default(), // TODO
                                                        explode: None,
                                                        extensions: Default::default(),
                                                    },
                                                    allow_reserved: false, // TODO: What should this be?
                                                    style: QueryStyle::Form, // TODO: What should this be?
                                                    allow_empty_value: Some(false),
                                                })]
                                            }
                                            ExecKind::Mutation => vec![],
                                        },
                                        request_body: match procedure_type {
                                            ExecKind::Query => None,
                                            ExecKind::Mutation => None, // TODO
                                        },
                                        responses: Responses {
                                            // default: todo!(),
                                            responses: [(
                                                StatusCode::Code(200),
                                                ReferenceOr::Item(openapiv3::Response {
                                                    content: [(
                                                        "application/json".to_string(),
                                                        MediaType {
                                                            schema: Some(result_schema.clone()),
                                                            // Some(ReferenceOr::Item(Schema {
                                                            //     schema_data: SchemaData {
                                                            //         // nullable: todo!(),
                                                            //         // read_only: todo!(),
                                                            //         // write_only: todo!(),
                                                            //         // deprecated: todo!(),
                                                            //         // external_docs: todo!(),
                                                            //         example: Some(json!({
                                                            //             "todo": "demo",
                                                            //         })),
                                                            //         // title: todo!(),
                                                            //         // description: todo!(),
                                                            //         // discriminator: todo!(),
                                                            //         // default: todo!(),
                                                            //         // extensions: todo!(),
                                                            //         ..Default::default()
                                                            //     },
                                                            //     schema_kind: SchemaKind::Type(
                                                            //         Type::Object(ObjectType {
                                                            //             properties: [(
                                                            //                 "todo".to_string(),
                                                            //                 ReferenceOr::Item(
                                                            //                     Box::new(Schema {
                                                            //                         schema_data: Default::default(),
                                                            //                         schema_kind: SchemaKind::Type(Type::String(StringType {
                                                            //                             // format: todo!(),
                                                            //                             // pattern: todo!(),
                                                            //                             // enumeration: todo!(),
                                                            //                             // min_length: todo!(),
                                                            //                             // max_length: todo!(),
                                                            //                             ..Default::default()
                                                            //                         })),
                                                            //                     }),
                                                            //                 ),
                                                            //             )]
                                                            //             .into_iter()
                                                            //             .collect(),
                                                            //             // required: todo!(),
                                                            //             // additional_properties: todo!(),
                                                            //             // min_properties: todo!(),
                                                            //             // max_properties: todo!(),
                                                            //             ..Default::default()
                                                            //         }),
                                                            //     ),
                                                            // })),
                                                            // example: todo!(),
                                                            // examples: todo!(),
                                                            // encoding: todo!(),
                                                            // extensions: todo!(),
                                                            ..Default::default()
                                                        },
                                                    )]
                                                    .into_iter()
                                                    .collect(),
                                                    ..Default::default()
                                                }),
                                            )]
                                            .into_iter()
                                            .collect(),
                                            ..Default::default()
                                        },
                                        ..Default::default()
                                    };

                                    match *method {
                                        Method::GET => PathItem {
                                            get: Some(operation),
                                            ..Default::default()
                                        },
                                        Method::POST => PathItem {
                                            post: Some(operation),
                                            ..Default::default()
                                        },
                                        Method::PUT => PathItem {
                                            put: Some(operation),
                                            ..Default::default()
                                        },
                                        Method::DELETE => PathItem {
                                            delete: Some(operation),
                                            ..Default::default()
                                        },
                                        Method::HEAD => PathItem {
                                            head: Some(operation),
                                            ..Default::default()
                                        },
                                        Method::OPTIONS => PathItem {
                                            options: Some(operation),
                                            ..Default::default()
                                        },
                                        Method::PATCH => PathItem {
                                            patch: Some(operation),
                                            ..Default::default()
                                        },
                                        Method::TRACE => PathItem {
                                            trace: Some(operation),
                                            ..Default::default()
                                        },
                                        _ => panic!("TODO"),
                                    }
                                }),
                            )
                        },
                    )
                    .collect(),
                ..Default::default()
            },
            // TODO: Because of OpenAPI's lack of generics this isn't possible.
            // components: Some(Components {
            //     schemas: self
            //         .typ_store()
            //         .iter()
            //         .map(|(k, v)| {
            //             println!("{:?}", v);
            //             (k.to_string(), to_openapi(v))
            //         })
            //         .collect(),
            //     ..Default::default()
            // }),
            ..Default::default()
        }
    }

    #[allow(clippy::unwrap_used)] // TODO: Remove lint override once this feature is stable
    pub fn openapi_endpoint<
        TCtxFnMarker: Send + Sync + 'static,
        TCtxFn: TCtxFunc<TCtx, TCtxFnMarker>,
    >(
        self: Arc<Self>,
        config: OpenAPIConfig,
        ctx_fn: TCtxFn,
    ) -> Endpoint<impl HttpEndpoint> {
        let doc = self.openapi_doc(config);
        let openapi =
            Arc::new(serde_json::to_string(&doc).expect("Could not serialize OpenAPI config!"));
        let endpoints = Arc::new({
            let mut data = self.data.write().unwrap();

            let ctx = data
                .entry(TypeId::of::<OpenAPIContext>())
                .or_insert_with(|| {
                    Box::<OpenAPIContext>::default()
                        as Box<dyn std::any::Any + Send + Sync + 'static>
                })
                .downcast_mut::<OpenAPIContext>()
                .unwrap();

            ctx.endpoints
                .iter()
                .map(|((_method, url), v)| (format!("/api{}", url), v.clone()))
                .collect::<BTreeMap<_, _>>()
        });

        GenericEndpoint::new(
            [Method::GET, Method::POST], // TODO: All methods
            move |req: Request| {
                // TODO: Don't clone per request, do per thread or keep reference?
                let router = self.clone();
                let openapi = openapi.clone();
                let endpoints = endpoints.clone();
                let ctx_fn = ctx_fn.clone();

                async move {
                    let path = &*req.uri().path().to_string();
                    let endpoint = endpoints.get(&path.to_string());
                    // TODO: Path related to prefix defined on the router or allow user providing a base path
                    match (path, endpoint) {
                        // TODO: Use a full blown router to allow pattern matching into variables.
                        (_, Some(endpoint)) => {
                            let OpenAPIEndpoint {
                                procedure_name,
                                procedure_type,
                                ..
                            } = endpoint;
                            #[cfg(not(feature = "workers"))]
                            let ctx = match ctx_fn.exec(
                                &mut httpz::axum::axum::extract::RequestParts::new(req.into()),
                            ) {
                                TCtxFuncResult::Value(v) => v,
                                TCtxFuncResult::Future(v) => v.await,
                            };
                            #[cfg(feature = "workers")]
                            let ctx = match ctx_fn.exec() {
                                TCtxFuncResult::Value(v) => v,
                                TCtxFuncResult::Future(v) => v.await,
                            };

                            // TODO: Support query or mutation and error out if OpenAPI is put on a subscription
                            // TODO: Input value from request body
                            let result = router
                                .exec(
                                    ctx.unwrap(),
                                    *procedure_type,
                                    procedure_name.to_string(),
                                    None,
                                )
                                .await
                                .unwrap();

                            Response::builder()
                                .status(200)
                                .header("Content-Type", "application/json")
                                .body(serde_json::to_vec(&result).unwrap())
                                .unwrap()
                        }
                        ("/api/openapi.json", _) => Response::builder()
                            .status(200)
                            .header("Content-Type", "application/json")
                            .body((*openapi).clone().into_bytes())
                            .unwrap(),
                        // TODO: Allow disabling swagger UI
                        ("/api/ui", _) => Response::builder()
                            .status(200)
                            .header("Content-Type", "text/html")
                            .body(
                                SWAGGER_UI
                                    .get_file("index.html")
                                    .unwrap()
                                    .contents()
                                    .to_vec(),
                            )
                            .unwrap(),
                        _ if req.uri().path().starts_with("/api/_swaggerui/") => {
                            let path = req.uri().path().strip_prefix("/api/_swaggerui/").unwrap();
                            match SWAGGER_UI.get_file(path) {
                                Some(file) => Response::builder()
                                    .status(200)
                                    .header(
                                        "Content-Type",
                                        mime_guess::from_path(path)
                                            .first()
                                            .unwrap_or(mime::TEXT_PLAIN)
                                            .to_string(),
                                    )
                                    .body(file.contents().to_vec())
                                    .unwrap(),
                                None => Response::builder()
                                    .status(404)
                                    .header("Content-Type", "text/html")
                                    .body("404: Not found".into())
                                    .unwrap(),
                            }
                        }
                        _ => Response::builder()
                            .status(404)
                            .header("Content-Type", "text/html")
                            .body(b"404: Not found".to_vec())
                            .unwrap(),
                    }
                }
            },
        )
    }
}
