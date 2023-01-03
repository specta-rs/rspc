use serde_json::Value;
use std::{
    fs::File,
    io::{self, Read, Write},
    path::Path,
    process::exit,
};

pub fn replace_in_file(path: &Path, from: &str, to: &str) -> io::Result<()> {
    let data = {
        let mut src = File::open(path)?;
        let mut data = String::new();
        src.read_to_string(&mut data)?;
        data
    };

    let data = data.replace(from, to);

    {
        let mut dst = File::create(path)?;
        dst.write_all(data.as_bytes())?;
    }

    Ok(())
}

#[allow(unused)]
pub(crate) fn check_rust_msrv() -> Result<(), rustc_version::Error> {
    let version = rustc_version::version()?;

    if version.minor < 64 {
        println!("You are using an unsupported version of Rust, please update to 1.64 or higher.");
        println!("To update, run `rustup update`.");
        exit(1);
    };

    Ok(())
}

#[allow(unused)]
pub(crate) fn check_version() -> Result<(), Box<dyn std::error::Error>> {
    let resp: Value = ureq::get(&format!(
        "https://crates.io/api/v1/crates/{}",
        env!("CARGO_PKG_NAME")
    ))
    .call()?
    .into_json()?;

    let latest = resp.get("crate")
        .ok_or("Unable to find crate key in response from crates.io, please try again later.")?
        .get("max_version")
        .ok_or("Unable to find crate>max_version key in response from crates.io, please try again later.")?
    .as_str()
    .unwrap();

    if env!("CARGO_PKG_VERSION") != latest {
        println!(
            "A new version of create-rspc-app is available, please update to {}.",
            latest
        );
        println!("To update, run `cargo install create-rspc-app --force`.\n");
    }

    Ok(())
}
