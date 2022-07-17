use rspc::{Error, ErrorCode, Router};
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
    let router = <Router>::new()
        .query("ok", |_, args: ()| {
            Ok("Hello World".into()) as Result<String, Error>
        })
        .query("err", |_, args: ()| {
            Err(Error::new(
                ErrorCode::BadRequest,
                "This is a custom error!".into(),
            )) as Result<String, Error>
        })
        .query("customErr", |_, args: ()| {
            Err(MyCustomError::IAmBroke) as Result<String, MyCustomError>
        })
        .build();

    router.export("./ts").unwrap();

    println!(
        "{:#?}",
        router
            .exec_query((), "ok", json!("Hello World"))
            .await
            .unwrap()
    );
    // println!(
    //     "{:#?}",
    //     router.exec_query((), "err", json!(null)).await.unwrap()
    // );
    // println!(
    //     "{:#?}",
    //     router
    //         .exec_query((), "customErr", json!(null))
    //         .await
    //         .unwrap()
    // );
}
