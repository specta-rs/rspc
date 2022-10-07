use std::path::Path;

use crate::{database::Database, framework::Framework, frontend_framework::FrontendFramework};

pub fn code_generator(
    framework: Framework,
    database: Database,
    frontend_framework: FrontendFramework,
    path: &Path,
    project_name: &str,
) {
    println!("Generating code...");
    println!("Framework: {:?}", framework);
    println!("Database: {:?}", database);
    println!("Frontend Framework: {:?}", frontend_framework);

    if database == Database::None {
        framework.render(path, project_name).unwrap();
    } else {
        database
            .render(path, project_name, framework.clone())
            .unwrap();
    }

    frontend_framework.render(path, project_name, framework);
}
