use rspc::{
    Config, Error, ErrorCode, ExecError, OperationKey, OperationKind, Router, StreamOrValue,
};
use serde_json::json;

pub enum MyCustomError {
    IAmBroke,
}

impl Into<Error> for MyCustomError {
    fn into(self) -> Error {
        match self {
            MyCustomError::IAmBroke => {
                Error::new(ErrorCode::InternalServerError, "I am broke".into())
            }
        }
    }
}

#[tokio::main]
async fn main() {
    let r = <Router>::new()
        .config(Config::new().export_ts_bindings("./ts"))
        .query("ok", |_, _args: ()| {
            Ok("Hello World".into()) as Result<String, Error>
        })
        .query("err", |_, _args: ()| {
            Err(Error::new(
                ErrorCode::BadRequest,
                "This is a custom error!".into(),
            )) as Result<String, Error>
        })
        .query("customErr", |_, _args: ()| {
            Err(MyCustomError::IAmBroke) as Result<String, MyCustomError>
        })
        .query("asyncCustomError", |_, _args: ()| async move {
            Err(MyCustomError::IAmBroke) as Result<String, MyCustomError>
        })
        .build();

    // You usually don't use this method directly. An integration will handle this for you. Check out the Axum and Tauri integrations to see how to use them!
    match r
        .exec((), OperationKind::Query, OperationKey("ok".into(), None))
        .await
        .unwrap()
    {
        StreamOrValue::Stream(_) => unreachable!(),
        StreamOrValue::Value(v) => {
            println!("{:?}", v);
            assert_eq!(v, json!("Hello World"));
        }
    }

    let v = r
        .exec((), OperationKind::Query, OperationKey("err".into(), None))
        .await;
    println!("{:?}", v);
    assert!(matches!(v, Err(ExecError::ErrResolverError(_))));
}
