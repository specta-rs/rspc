use std::{env::current_dir, fs::remove_dir_all, str::FromStr};

use requestty::{prompt_one, Question};
use strum::IntoEnumIterator;

use crate::framework::Framework;

mod framework;
mod utils;

const BANNER: &str = r#"██████╗ ███████╗██████╗  ██████╗
██╔══██╗██╔════╝██╔══██╗██╔════╝
██████╔╝███████╗██████╔╝██║     
██╔══██╗╚════██║██╔═══╝ ██║     
██║  ██║███████║██║     ╚██████╗
╚═╝  ╚═╝╚══════╝╚═╝      ╚═════╝"#;

fn main() {
    // TODO: Autoupdate

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

    let framework = prompt_one(
        Question::select("framework")
            .message("What framework would you like to use?")
            .choices(Framework::iter().map(|v| v.to_string()))
            .build(),
    )
    .unwrap();
    let framework = Framework::from_str(&framework.as_list_item().unwrap().text).unwrap();

    // TODO: Database selection - Prisma Client Rust, None

    // TODO: Frontend selection - React, SolidJS, None

    // TODO: Extras selection -> Multiselect - TailwindCSS, tracing

    framework.render(path.as_path(), project_name).unwrap();
}
