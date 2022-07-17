use crate::utils::{UpdateUserArgs, User};
use rspc::{Config, Router};
use serde_json::json;

mod utils;

#[tokio::main]
async fn main() {
    let router = <Router>::new()
        .config(Config::new().export_ts_bindings("./ts"))
        .query("version", |_, _: ()| env!("CARGO_PKG_VERSION"))
        .mutation("createUser", |_, args| User::create(args))
        .query(
            "getUser",
            |_, id| async move { User::read(id).await.unwrap() },
        )
        .query("getUsers", |_, _: ()| User::read_all())
        .mutation("updateUser", |_, args: UpdateUserArgs| {
            User::update(args.id, args.new_user)
        })
        .mutation("deleteUser", |_, id| User::delete(id))
        .query("optional", |_, _: ()| None as Option<String>)
        .build();

    // You can also export the bindings yourself
    // router.export("./ts").unwrap();

    println!(
        "{:#?}",
        router.exec_query((), "version", json!(null)).await.unwrap()
    );
    println!(
        "{:#?}",
        router
            .exec_mutation(
                (),
                "createUser",
                json!({ "id": 1, "name": "Monty Beaumont", "email": "monty@otbeaumont.me" })
            )
            .await
            .unwrap()
    );
    println!(
        "{:#?}",
        router
            .exec_query((), "getUsers", json!(null))
            .await
            .unwrap()
    );
}
