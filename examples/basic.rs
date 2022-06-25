use std::path::Path;

use crate::utils::{UpdateUserArgs, User};
use serde_json::json;
use trpc_rs::Router;

mod utils;

#[tokio::main]
async fn main() {
    let router = Router::<()>::new()
        .query("version", |_, _: ()| env!("CARGO_PKG_VERSION"))
        .mutation(
            "createUser",
            |_, args| async move { User::create(args).await },
        )
        .query(
            "getUser",
            |_, id| async move { User::read(id).await.unwrap() },
        )
        .query("getUsers", |_, _: ()| async { User::read_all().await })
        .mutation("updateUser", |_, args: UpdateUserArgs| async move {
            User::update(args.id, args.new_user).await
        })
        .mutation("deleteUser", |_, id| async move { User::delete(id).await });

    router.export(Path::new("./ts")).unwrap();

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
