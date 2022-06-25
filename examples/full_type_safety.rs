use std::path::Path;

use crate::utils::{UpdateUserArgs, User};
use trpc_rs::{Mutation, Query, Router};

mod utils;

#[derive(Query)]
#[query(key = "QueryKey")]
pub enum Query {
    Version,
    GetUsers,
    GetUser(i32),
}

#[derive(Mutation)]
#[query(key = "MutationKey")]
pub enum Mutation {
    CreateUser(User),
    UpdateUser(UpdateUserArgs),
    DeleteUser(i32),
}

#[tokio::main]
async fn main() {
    let router = Router::<(), Query, Mutation>::new()
        .query(QueryKey::Version, |_, _| env!("CARGO_PKG_VERSION"))
        .query(QueryKey::GetUsers, |_, _| async { User::read_all().await })
        .query(QueryKey::GetUser, |_, id| async move {
            User::read(id).await.unwrap()
        })
        .mutation(MutationKey::CreateUser, |_, args| async move {
            User::create(args).await
        })
        .mutation(MutationKey::UpdateUser, |_, args| async move {
            User::update(args.id, args.new_user).await
        })
        .mutation(MutationKey::DeleteUser, |_, id| async move {
            User::delete(id).await
        });

    router.export(Path::new("./ts")).unwrap();

    println!(
        "{:#?}",
        router.exec_query((), QueryKey::Version, ()).await.unwrap()
    );
    println!(
        "{:#?}",
        router
            .exec_mutation(
                (),
                MutationKey::CreateUser,
                User {
                    id: 1,
                    name: "Monty Beaumont".into(),
                    email: "monty@otbeaumont.me".into(),
                }
            )
            .await
            .unwrap()
    );
    println!(
        "{:#?}",
        router.exec_query((), QueryKey::GetUsers, ()).await.unwrap()
    );
}
