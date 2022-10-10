use std::{io, path::PathBuf};

use strum::{Display, EnumIter, EnumString};

#[derive(Debug, Display, EnumIter, EnumString)]
pub enum PackageManager {
    NPM,
    Yarn,
    PNPM,
    None,
}

pub fn run_cargo_steps(path: PathBuf, cmd: &str) -> io::Result<()> {
    std::env::set_current_dir(path.clone())?;

    if path.join("prisma").exists() {
        let mut cargo_process = std::process::Command::new(cmd)
            .args(&["/C", "cargo", "prisma", "generate"])
            .spawn()?;

        cargo_process.wait()?;
    }

    Ok(())
}

impl PackageManager {
    pub fn exec(&self, path: PathBuf) -> io::Result<()> {
        let pkg_path = path.join("web");
        std::env::set_current_dir(pkg_path.clone())?;
        let mut cmd = "cmd";

        if cfg!(unix) {
            cmd = "sh";
        }

        match self {
            PackageManager::NPM => {
                println!("$ npm install");
                let mut npm = std::process::Command::new(cmd)
                    .args(&["/C", "npm", "install"])
                    .spawn()?;

                npm.wait()?;

                println!("Successfully installed npm packages");
            }
            PackageManager::Yarn => {
                println!("$ yarn install");

                // run yarn install and print the output
                let mut yarn = std::process::Command::new(cmd)
                    .args(&["/C", "yarn", "install"])
                    .spawn()?;

                yarn.wait()?;
                println!("Successfully installed yarn packages");
            }
            PackageManager::PNPM => {
                println!("$ pnpm install");

                let mut child = std::process::Command::new(cmd)
                    .args(&["/C", "pnpm", "install"])
                    .spawn()?;

                child.wait()?;

                println!("Successfully installed pnpm packages");
            }
            PackageManager::None => {}
        };

        run_cargo_steps(path, cmd)?;

        Ok(())
    }
}
