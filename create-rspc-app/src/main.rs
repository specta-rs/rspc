use std::{env::current_dir, fs::remove_dir_all, process::exit, str::FromStr};

use requestty::{prompt_one, Question};
use strum::IntoEnumIterator;

use crate::{
    database::Database, framework::Framework, frontend_framework::FrontendFramework,
    generator::code_generator, utils::check_rust_msrv,
};

mod database;
mod framework;
mod frontend_framework;
mod generator;
mod utils;

const BANNER: &str = r#"
██████╗ ███████╗██████╗  ██████╗
██╔══██╗██╔════╝██╔══██╗██╔════╝
██████╔╝███████╗██████╔╝██║     
██╔══██╗╚════██║██╔═══╝ ██║     
██║  ██║███████║██║     ╚██████╗
╚═╝  ╚═╝╚══════╝╚═╝      ╚═════╝"#;

fn main() {
    check_rust_msrv();

    println!("\n{}\n", BANNER);

    let project_name = prompt_one(
        Question::input("project_name")
            .message("What will your project be called?")
            .default("my-app")
            .build(),
    )
    .unwrap();
    let project_name = project_name.as_string().unwrap();

    if !project_name
        .chars()
        .all(|x| x.is_alphanumeric() || x == '-' || x == '_')
    {
        println!("Aborting your project name may only contain alphanumeric characters along with '-' and '_'...");
    }

    let path = current_dir().unwrap().join(project_name);
    if path.exists() {
        let force = prompt_one(
            Question::confirm("force_delete")
                .message(format!(
                    "{} directory is not empty, do you want to overwrite?",
                    project_name
                ))
                .default(false)
                .build(),
        )
        .unwrap();

        match !force.as_bool().unwrap() {
            true => {
                println!("Aborting project creation...");
                return;
            }
            false => {
                remove_dir_all(&path).unwrap();
            }
        }
    }

    // Framework
    let framework = prompt_one(
        Question::select("framework")
            .message("What backend framework would you like to use?")
            .choices(Framework::iter().map(|v| v.to_string()))
            .build(),
    )
    .unwrap();
    let framework = Framework::from_str(&framework.as_list_item().unwrap().text).unwrap();

    // Database selection - Prisma Client Rust, None
    let database = prompt_one(
        Question::select("database")
            .message("What database ORM would you like to use?")
            .choices(Database::iter().map(|v| v.to_string()))
            .build(),
    )
    .unwrap();
    let database = Database::from_str(&database.as_list_item().unwrap().text).unwrap();

    // Frontend selection - React, SolidJS, None
    let frontend_framework = prompt_one(
        Question::select("frontend_framework")
            .message("What frontend framework would you like to use?")
            .choices(FrontendFramework::iter().map(|v| v.to_string()))
            .build(),
    )
    .unwrap();
    let frontend_framework =
        FrontendFramework::from_str(&frontend_framework.as_list_item().unwrap().text).unwrap();

    if let Err(e) = code_generator(
        framework,
        database,
        frontend_framework,
        &path,
        &project_name,
    ) {
        println!("Error: {}", e);
        exit(1)
    };
}
