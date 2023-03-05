use std::{io, path::PathBuf};

use strum::{Display, EnumIter, EnumString};

use crate::database::Database;

#[derive(Debug, Display, EnumIter, EnumString)]
#[allow(clippy::upper_case_acronyms)]
pub enum PackageManager {
    NPM,
    Yarn,
    PNPM,
    None,
}

pub fn run_cargo_steps(path: PathBuf, db: Database) -> io::Result<()> {
    std::env::set_current_dir(path)?;
    if db == Database::PrismaClientRust {
        #[cfg(target_os = "windows")]
        let mut child = std::process::Command::new("cmd")
            .args(["/C", "cargo", "prisma", "generate"])
            .spawn()?;

        #[cfg(not(target_os = "windows"))]
        let mut child = std::process::Command::new("cargo")
            .args(["prisma", "generate"])
            .spawn()?;

        child.wait()?;
    }

    Ok(())
}

impl PackageManager {
    pub fn exec(&self, path: PathBuf) -> io::Result<()> {
        let pkg_path = path.join("web");
        std::env::set_current_dir(pkg_path)?;

        match self {
            PackageManager::NPM => {
                println!("$ npm install");
                #[cfg(target_os = "windows")]
                let mut child = std::process::Command::new("cmd")
                    .args(["/C", "npm", "install"])
                    .spawn()?;

                #[cfg(not(target_os = "windows"))]
                let mut child = std::process::Command::new("npm")
                    .args(["install"])
                    .spawn()?;

                child.wait()?;
                println!("Successfully installed npm packages");
            }
            PackageManager::Yarn => {
                println!("$ yarn install");
                #[cfg(target_os = "windows")]
                let mut child = std::process::Command::new("cmd")
                    .args(["/C", "yarn", "install"])
                    .spawn()?;

                #[cfg(not(target_os = "windows"))]
                let mut child = std::process::Command::new("yarn")
                    .args(["install"])
                    .spawn()?;

                child.wait()?;
                println!("Successfully installed yarn packages");
            }
            PackageManager::PNPM => {
                println!("$ pnpm install");
                #[cfg(target_os = "windows")]
                let mut child = std::process::Command::new("cmd")
                    .args(["/C", "pnpm", "install"])
                    .spawn()?;

                #[cfg(not(target_os = "windows"))]
                let mut child = std::process::Command::new("pnpm")
                    .args(["install"])
                    .spawn()?;

                child.wait()?;
                println!("Successfully installed pnpm packages");
            }
            PackageManager::None => {}
        };

        Ok(())
    }
}
