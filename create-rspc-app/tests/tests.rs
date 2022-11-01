use std::{
    env,
    fs::{create_dir_all, remove_dir_all},
    path::Path,
    process::Stdio,
};

use cargo::{
    core::{Shell, Verbosity, Workspace},
    ops::{CompileOptions, Packages},
    util::{command_prelude::CompileMode, homedir},
    Config,
};
use create_rspc_app::internal::{
    database::Database, framework::Framework, frontend_framework::FrontendFramework,
    generator::code_generator,
};
use futures::future::join_all;
use strum::IntoEnumIterator;
use tempdir::TempDir;
use tokio::process::Command;

#[tokio::test]
async fn test_templates() {
    env::set_var(
        "CARGO_TARGET_DIR",
        env::current_dir().unwrap().join("test-target"),
    );
    env::set_var("CARGO_TERM_VERBOSE", "false");
    env::set_var("CARGO_QUITE", "true");
    let dir = TempDir::new("create_rspc_app_test").unwrap();
    let result = _test(dir.path()).await;
    env::remove_var("CARGO_TARGET_DIR");
    remove_dir_all(&dir).unwrap();
    assert_eq!(result, Ok(()));
}

async fn _test(base_dir: &Path) -> Result<(), String> {
    let x = Framework::iter().flat_map(|framework| {
        Database::iter().flat_map(move |database| {
            let framework = framework.clone();
            FrontendFramework::iter().map(move |frontend| {
                let (framework, database) = (framework.clone(), database.clone());
                async move {
                    let dir =
                        base_dir.join(format!("{:?}-{:?}-{:?}", framework, database, frontend));

                    if let Err(err) = code_generator(
                        framework.clone(),
                        database.clone(),
                        frontend.clone(),
                        // extra.clone(),
                        &dir,
                        "rspc-test",
                    ) {
                        return Err(format!(
                            "Error({:?}-{:?}-{:?}): Failed to generate: {}",
                            framework, database, frontend, err
                        ));
                    }

                    dir.exists().then_some(()).ok_or_else(|| {
                        format!(
                            "Error({:?}-{:?}-{:?}): No directory was generated!",
                            framework, database, frontend
                        )
                    })?;

                    // TODO: This should probs be configurable but fine for now given it's only for the test cases
                    Command::new("pnpm")
                        .args(["install"])
                        .current_dir(&dir.join("web"))
                        .stdout(Stdio::null())
                        .stderr(Stdio::null())
                        .status()
                        .await
                        .map_err(|err| {
                            format!(
                                "Error({:?}-{:?}-{:?}): Failed to run 'pnpm install' in web: {}",
                                framework, database, frontend, err
                            )
                        })?;

                    Command::new("pnpm")
                        .args(["build"])
                        .current_dir(&dir.join("web"))
                        .stdout(Stdio::null())
                        .stderr(Stdio::null())
                        .status()
                        .await
                        .map_err(|err| {
                            format!(
                                "Error({:?}-{:?}-{:?}): Failed to run 'pnpm build' in web: {}",
                                framework, database, frontend, err
                            )
                        })?;

                    // TODO: Do a Typescript typecheck

                    if framework == Framework::Tauri {
                        create_dir_all(dir.join("./web/dist")).map_err(|err| {
                            format!(
                                "Error({:?}-{:?}-{:?}): Failed to create Prisma dist folder: {}",
                                framework, database, frontend, err
                            )
                        })?;
                    }

                    if database == Database::PrismaClientRust {
                        Command::new("cargo")
                            .args(["--quiet", "prisma", "generate"])
                            .current_dir(&dir)
                            .stdout(Stdio::null())
                            .stderr(Stdio::null())
                            .status()
                            .await
                            .map_err(|err| {
                                format!(
                                    "Error({:?}-{:?}-{:?}): Failed to run cargo prisma generate: {}",
                                    framework, database, frontend, err
                                )
                            })?;
                    }

                    let mut shell = Shell::new();
                    shell.set_verbosity(Verbosity::Quiet);
                    let cwd = env::current_dir().unwrap();
                    let cfg = Config::new(shell, cwd.clone(),  homedir(&cwd).unwrap());

                    let ws =
                        &Workspace::new(dir.join("Cargo.toml").as_path(), &cfg).map_err(|err| {
                            format!(
                                "Error({:?}-{:?}-{:?}): Failed to load workspace: {}",
                                framework, database, frontend, err
                            )
                        })?;

                    let mut compile_cfg = CompileOptions::new(&cfg, CompileMode::Check { test: false })
                        .map_err(|err| {
                            format!(
                                "Error({:?}-{:?}-{:?}): Failed to load workspace: {}",
                                framework, database, frontend, err
                            )
                        })?;
                    compile_cfg.spec = Packages::Packages(vec!["rspc-test".into()]);
                    if let Err(err) = cargo::ops::compile(&ws, &compile_cfg) {
                        return Err(format!(
                            "Error({:?}-{:?}-{:?}): Failed to compile: {}",
                            framework, database, frontend, err
                        ));
                    }

                    Ok(())
                }
            })
        })
    });

    join_all(x).await.into_iter().collect::<Result<(), _>>()
}
