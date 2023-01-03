use std::{env::current_dir, fs::remove_dir_all, str::FromStr};

use requestty::{prompt_one, ErrorKind, Question};
use strum::IntoEnumIterator;

use crate::{
    database::Database,
    framework::Framework,
    frontend_framework::FrontendFramework,
    generator::code_generator,
    post_gen::{run_cargo_steps, PackageManager},
    utils::{check_rust_msrv, check_version},
};

pub(crate) mod database;
mod errors;
pub(crate) mod framework;
pub(crate) mod frontend_framework;
pub(crate) mod generator;
pub mod internal;
pub(crate) mod post_gen;
mod utils;

const BANNER: &str = r#"
██████╗ ███████╗██████╗  ██████╗
██╔══██╗██╔════╝██╔══██╗██╔════╝
██████╔╝███████╗██████╔╝██║     
██╔══██╗╚════██║██╔═══╝ ██║     
██║  ██║███████║██║     ╚██████╗
╚═╝  ╚═╝╚══════╝╚═╝      ╚═════╝"#;

fn try_main() -> Result<(), errors::Error> {
    println!("\n{}\n", BANNER);

    check_rust_msrv()?;
    check_version()?;

    ctrlc::set_handler(|| {
        println!("Operation cancelled by user");
    })
    .expect("Unable to setup ctrl+c handler");

    let project_name = prompt_one(
        Question::input("project_name")
            .message("What will your project be called?")
            .default("my-app")
            .build(),
    )?;
    let project_name = project_name.as_string().unwrap();

    if !project_name
        .chars()
        .all(|x| x.is_alphanumeric() || x == '-' || x == '_')
    {
        println!("Aborting your project name may only contain alphanumeric characters along with '-' and '_'...");
    }

    // First argument is the path where the project has to be created
    let dir_path = match std::env::args().nth(1) {
        Some(value) => value,
        None => project_name.to_string(),
    };

    let path = current_dir()?.join(dir_path);
    if path.exists() {
        let force = prompt_one(
            Question::confirm("force_delete")
                .message(format!(
                    "{} directory is not empty, do you want to overwrite?",
                    project_name
                ))
                .default(false)
                .build(),
        )?;

        match !force.as_bool().unwrap() {
            true => {
                println!("Aborting project creation...");
                return Err(errors::Error::Other(
                    "Aborting project creation...".to_string(),
                ));
            }
            false => {
                remove_dir_all(&path)?;
            }
        }
    }

    // Framework
    let framework = prompt_one(
        Question::select("framework")
            .message("What backend framework would you like to use?")
            .choices(Framework::iter().map(|v| v.to_string()))
            .build(),
    )?;
    let framework = Framework::from_str(&framework.as_list_item().unwrap().text).unwrap();

    // Database selection - Prisma Client Rust, None
    let database = prompt_one(
        Question::select("database")
            .message("What database ORM would you like to use?")
            .choices(Database::iter().map(|v| v.to_string()))
            .build(),
    )?;
    let database = Database::from_str(&database.as_list_item().unwrap().text).unwrap();

    // Frontend selection - React, SolidJS, None
    let frontend_framework = prompt_one(
        Question::select("frontend_framework")
            .message("What frontend framework would you like to use?")
            .choices(FrontendFramework::iter().map(|v| v.to_string()))
            .build(),
    )?;
    let frontend_framework =
        FrontendFramework::from_str(&frontend_framework.as_list_item().unwrap().text).unwrap();

    let package_manager = prompt_one(
        Question::select("package_manager")
            .message("What package manager would you like to use?")
            .choices(PackageManager::iter().map(|v| v.to_string()))
            .build(),
    )?;
    let package_manager =
        PackageManager::from_str(&package_manager.as_list_item().unwrap().text).unwrap();

    code_generator(
        framework,
        database.clone(),
        frontend_framework,
        // extras,
        &path,
        project_name,
    )?;

    package_manager.exec(path.clone())?;

    run_cargo_steps(path, database)?;

    Ok(())
}

fn main() {
    if let Err(e) = try_main() {
        if matches!(e, errors::Error::RequestTty(ErrorKind::Interrupted)) {
            return;
        }

        println!("\n{}", e);
        println!("\nNOTE: If this error persists please consider opening an issue at");
        println!("  https://github.com/oscartbeaumont/rspc/issues\n")
    }
}
