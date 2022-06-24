use crate::utils::User;
use serde_json::json;
use trpc_rs::{Router, ZData, Z};

mod utils;

#[tokio::main]
async fn main() {
    let x = Z::string();
    println!("{:?}", x.parse(json!("Hello World")));
    println!("{:?}", x.parse(json!(null)));

    let x = Z::object()
        .field("name", Z::string())
        .field("email", Z::string())
        .field("extra", Z::object().field("age", Z::number()));
    println!(
        "{:?}",
        x.parse(json!({ "name": "Oscar Beaumont", "email": "oscar@otbeaumont.me" }))
    );
    println!("{:?}", x.parse(json!(null)));

    // TODO: Make this work using a macro or trait on `User` type with some prevalidation step to ensure zod types match struct before runtime.
    // let _ = Z::object_struct::<User>();

    let router = Router::<()>::new()
        .query("version", |_| env!("CARGO_PKG_VERSION"))
        .mutation("createUser", |_| async move {
            // TODO: Get User from `trpc_rs` arguments.
            User::create(User {
                id: 1,
                name: "Oscar Beaumont".into(),
                email: "oscar@otbeaumont.me".into(),
            })
            .await
        })
        .query("getUser", |_| async {
            // TODO: Get `id` from `trpc_rs` arguments.
            User::read(1).await.unwrap()
        })
        .query("getUsers", |_| async { User::read_all().await })
        .mutation("updateUser", |_| async {
            // TODO: Get `id` and new user object from `trpc_rs` arguments.
            User::update(
                1,
                User {
                    id: 1,
                    name: "Monty Beaumont".into(),
                    email: "monty@otbeaumont.me".into(),
                },
            )
            .await
        })
        .mutation("deleteUser", |_| async {
            // TODO: Get `id` from `trpc_rs` arguments.
            User::delete(1).await
        });

    println!("{:#?}", router.exec_query((), "version").await.unwrap());

    println!("{:#?}", router.exec_query((), "getUsers").await.unwrap());
}
