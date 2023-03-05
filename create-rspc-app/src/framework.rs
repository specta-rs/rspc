use std::{fs::create_dir, io, path::Path};

use crate::utils::replace_in_file;
use create_tauri_app::internal::{package_manager::PackageManager, template::Template};
use include_dir::{include_dir, Dir};
use strum::{Display, EnumIter, EnumString};

static AXUM_BASE_TEMPLATE: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/templates/axum_base");
static TAURI_BASE_TEMPLATE: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/templates/tauri_base");

#[derive(Debug, Display, EnumIter, EnumString)]
pub enum Framework {
    Axum,
    Tauri,
}

impl Framework {
    pub fn render(&self, path: &Path, project_name: &str) -> io::Result<()> {
        match self {
            Self::Axum => {
                create_dir(&path).unwrap();
                AXUM_BASE_TEMPLATE.extract(path)?;
                replace_in_file(path.join("Cargo.toml").as_path(), "{{name}}", project_name)?;

                println!(
                    "\nNow run `cd {}/ && cargo run` to get started with your new project!",
                    project_name
                );
            }
            Self::Tauri => {
                // TODO: Don't hardcode Template and PackageManager
                Template::ReactTs
                    .render(path, PackageManager::Pnpm, project_name)
                    .unwrap();

                TAURI_BASE_TEMPLATE.extract(path)?;

                println!("\nNow run `cd {}/ && pnpm i && cargo tauri dev` to get started with your new project!", project_name);
            }
        }

        Ok(())
    }
}
