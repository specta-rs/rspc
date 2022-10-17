use std::{io, path::Path};

use crate::{database::Database, framework::Framework, frontend_framework::FrontendFramework};

pub fn code_generator(
    framework: Framework,
    database: Database,
    frontend_framework: FrontendFramework,
    path: &Path,
    project_name: &str,
) -> io::Result<()> {
    if database == Database::None {
        framework.render(path, project_name)?;
    } else {
        database.render(path, project_name, framework.clone())?;
    }

    frontend_framework.render(path, project_name, framework)?;

    println!("Generated project at '{}'", path.display());

    Ok(())
}
