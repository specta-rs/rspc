use std::{io, path::Path};

use crate::utils::replace_in_file;
use include_dir::{include_dir, Dir};
use strum::{Display, EnumIter, EnumString};

static AXUM_BASE_TEMPLATE: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/templates/axum_base");
static TAURI_BASE_TEMPLATE: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/templates/tauri_base");

#[derive(Debug, Clone, Display, EnumIter, EnumString, PartialEq, Eq)]
pub enum Framework {
    Axum,
    Tauri,
}

impl Framework {
    pub fn render(&self, path: &Path, project_name: &str) -> io::Result<()> {
        match self {
            Self::Axum => {
                std::fs::create_dir_all(path)?;
                AXUM_BASE_TEMPLATE.extract(path)?;
                replace_in_file(path.join("Cargo__toml").as_path(), "__name__", project_name)?;
            }
            Self::Tauri => {
                std::fs::create_dir_all(path)?;
                TAURI_BASE_TEMPLATE.extract(path)?;
                let path = path.join("src-tauri");
                replace_in_file(path.join("Cargo__toml").as_path(), "__name__", project_name)?;
            }
        }

        Ok(())
    }
}
