//! An example of exposing rspc via Binario (instead of Serde) w/ Axum.
//! This is more to prove it's possible than something you should actually copy.

use axum::{
    body::Body,
    http::{header, request::Parts, HeaderMap},
    routing::{get, post},
};
use futures::TryStreamExt;
use rspc::{DynInput, Procedure, ProcedureBuilder, Procedures, ResolverInput, ResolverOutput};
use rspc_binario::BinarioOutput;
use specta::Type;
use std::{convert::Infallible, marker::PhantomData};
use tokio_util::compat::FuturesAsyncReadCompatExt;
use tower_http::cors::{Any, CorsLayer};

#[derive(Type)]
pub enum Error {
    Binario(#[specta(skip)] rspc_binario::DeserializeError),
}
impl From<rspc_binario::DeserializeError> for Error {
    fn from(err: rspc_binario::DeserializeError) -> Self {
        Error::Binario(err)
    }
}
impl rspc::Error for Error {
    fn into_procedure_error(self) -> rspc::ProcedureError {
        todo!(); // TODO: Work this out
    }
}

type Ctx = ();
pub struct BaseProcedure<TErr = Error>(PhantomData<TErr>);
impl<TErr> BaseProcedure<TErr> {
    pub fn builder<TInput, TResult>(
    ) -> ProcedureBuilder<TErr, Ctx, Ctx, TInput, TInput, TResult, TResult>
    where
        TErr: rspc::Error,
        TInput: ResolverInput,
        TResult: ResolverOutput<TErr>,
    {
        Procedure::builder() // You add default middleware here
    }
}

#[derive(Debug, Clone, binario::Encode, binario::Decode, Type)]
pub struct Input {
    name: String,
}

pub fn mount() -> rspc::Router<()> {
    rspc::Router::new()
        .procedure("binario", {
            <BaseProcedure>::builder()
                .with(rspc_binario::binario())
                .query(|_, input: Input| async move { Ok(input) })
        })
        .procedure("streaming", {
            <BaseProcedure>::builder()
                .with(rspc_binario::binario())
                .query(|_, input: Input| async move {
                    Ok(rspc::Stream(futures::stream::iter([
                        input.clone(),
                        input.clone(),
                        input,
                    ])))
                })
        })
}

#[tokio::main]
async fn main() {
    let router = mount();
    let (procedures, _types) = router.build().unwrap();

    // We disable CORS because this is just an example. DON'T DO THIS IN PRODUCTION!
    let cors = CorsLayer::new()
        .allow_methods(Any)
        .allow_headers(Any)
        .allow_origin(Any);

    let app = axum::Router::new()
        .route("/", get(|| async { "Hello 'rspc'!" }))
        .nest("/rspc", rspc_binario_handler(procedures))
        .layer(cors);

    let addr = "[::]:4000".parse::<std::net::SocketAddr>().unwrap(); // This listens on IPv6 and IPv4
    println!("listening on http://{}/rspc/binario", addr);
    axum::serve(tokio::net::TcpListener::bind(addr).await.unwrap(), app)
        .await
        .unwrap();
}

pub fn rspc_binario_handler(procedures: Procedures<()>) -> axum::Router {
    // This endpoint lacks batching, SFMs and more but that's not really the point of this example.
    axum::Router::new().route(
        "/binario",
        post({
            let procedures = procedures.clone();

            move |parts: Parts, body: Body| async move {
                let ctx = ();

                // if parts.headers.get("Content-Type") != Some(&"text/x-binario".parse().unwrap()) {
                //     // TODO: Error handling
                // }

                let mut params = form_urlencoded::parse(parts.uri.query().unwrap_or("").as_bytes());
                let procedure_name = params
                    .find(|(key, _)| key == "procedure")
                    .map(|(_, value)| value)
                    .unwrap(); // TODO: Error handling

                let procedure = procedures.get(&procedure_name).unwrap(); // TODO: Error handling

                let mut input = Some(rspc_binario::BinarioInput::from_stream(
                    body.into_data_stream()
                        .map_err(|_err| todo!()) // TODO: Error handling
                        .into_async_read()
                        .compat(),
                ));
                let stream = procedure.exec(ctx.clone(), DynInput::new_value(&mut input));
                let mut headers = HeaderMap::new();
                headers.insert(header::CONTENT_TYPE, "text/x-binario".parse().unwrap());

                let mut first = true;
                (
                    headers,
                    Body::from_stream(stream.map(move |v| {
                        let buf = match v {
                            Ok(v) => Ok(Ok::<_, Infallible>(
                                v.as_value::<BinarioOutput>().unwrap().0,
                            )),
                            Err(err) => todo!("{err:?}"),
                        };

                        if first {
                            first = false;
                            buf
                        } else {
                            buf.map(|v| {
                                v.map(|mut v| {
                                    let mut buf = vec!['\n' as u8, '\n' as u8];
                                    buf.append(&mut v);
                                    buf
                                })
                            })
                        }
                    })),
                )
            }
        }),
    )
}
