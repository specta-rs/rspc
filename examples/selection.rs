use rspc::{selection, selection_vec, Router};

#[derive(Clone)]
pub struct User {
    pub name: String,
    pub email: String,
    pub age: u8,
    pub password: String,
}

#[tokio::main]
async fn main() {
    let router = <Router>::new()
        .query("me", |_, _: ()| {
            // We have some data which contains information but we only want to return some of it the user.
            // Eg. We don't want to expose the password field.
            let user = User {
                name: "Monty Beaumont".into(),
                email: "monty@otbeaumont.me".into(),
                age: 7,
                password: "password123".into(),
            };

            // TODO: Fix the Rust compile warning here
            selection!(user, { name, age }) // Here we are selecting the fields we want to expose from the user. This is completely type safe!
        })
        .query("users", |_, _: ()| {
            // We have some data which contains information but we only want to return some of it the user.
            // Eg. We don't want to expose the password field.
            let user = User {
                name: "Monty Beaumont".into(),
                email: "monty@otbeaumont.me".into(),
                age: 7,
                password: "password123".into(),
            };
            let users = vec![user.clone(), user.clone(), user];

            // TODO: Fix the Rust compile warning here
            selection_vec!(users, { name, age }) // Here we are selecting the fields we want to expose on each item in the list. This is completely type safe!
        })
        .build();

    router.export("./ts").unwrap();
}
