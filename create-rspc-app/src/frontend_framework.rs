use std::{fs::create_dir_all, io, path::Path};

use crate::utils::replace_in_file;
use include_dir::{include_dir, Dir};
use strum::{Display, EnumIter, EnumString};

use crate::framework::Framework;

static REACT_TEMPLATE_TAURI: Dir<'_> =
    include_dir!("$CARGO_MANIFEST_DIR/templates/react_base_tauri");
static REACT_TEMPLATE_AXUM: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/templates/react_base_axum");

static SOLID_TEMPLATE_TAURI: Dir<'_> =
    include_dir!("$CARGO_MANIFEST_DIR/templates/solid_base_tauri");
static SOLID_TEMPLATE_AXUM: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/templates/solid_base_axum");

#[derive(Debug, Clone, Display, EnumIter, EnumString)]
pub enum FrontendFramework {
    React,
    SolidJS,
    // None,
}

impl FrontendFramework {
    pub fn render(&self, path: &Path, project_name: &str, framework: Framework) -> io::Result<()> {
        let path = path.join("web");
        create_dir_all(&path).unwrap();

        match framework {
            Framework::Tauri => match self {
                FrontendFramework::React => {
                    REACT_TEMPLATE_TAURI.extract(path.clone()).unwrap();

                    replace_in_file(
                        path.join("package.json").as_path(),
                        "{{name}}",
                        project_name,
                    )?;
                }
                FrontendFramework::SolidJS => {
                    SOLID_TEMPLATE_TAURI.extract(path.clone()).unwrap();

                    replace_in_file(
                        path.join("package.json").as_path(),
                        "{{name}}",
                        project_name,
                    )?;
                } // FrontendFramework::None => {}
            },
            Framework::Axum => match self {
                FrontendFramework::React => {
                    REACT_TEMPLATE_AXUM.extract(path.clone()).unwrap();

                    replace_in_file(
                        path.join("package.json").as_path(),
                        "{{name}}",
                        project_name,
                    )?;
                }
                FrontendFramework::SolidJS => {
                    SOLID_TEMPLATE_AXUM.extract(path.clone()).unwrap();

                    replace_in_file(
                        path.join("package.json").as_path(),
                        "{{name}}",
                        project_name,
                    )?;
                } // FrontendFramework::None => {}
            },
        }

        Ok(())
    }
}
