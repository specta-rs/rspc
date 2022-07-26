use std::path::PathBuf;

use rspc::{selection, Config, Router};
use specta::Type;

#[derive(Type)]
pub struct User {
    pub id: i32,
    pub name: String,
    pub age: i32,
    pub password: String,
}

#[tokio::main]
async fn main() {
    let _r =
        <Router>::new()
            .config(Config::new().export_ts_bindings(
                PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("./bindings.ts"),
            ))
            .query("customSelection", |_, _: ()| {
                // The user come from your database.
                let user = User {
                    id: 1,
                    name: "Monty Beaumont".to_string(),
                    age: 7,
                    password: "password".to_string(),
                };

                selection!(user, { id, name, age })
            })
            .query("customSelectionOnList", |_, _: ()| {
                // The users come from your database.
                let users = vec![User {
                    id: 1,
                    name: "Monty Beaumont".to_string(),
                    age: 7,
                    password: "password".to_string(),
                }];
                selection!(users, [{ id, name, age }])
            })
            .build();
}
