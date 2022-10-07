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
        dst.write(data.as_bytes())?;
    }

    Ok(())
}

pub fn check_rust_msrv() {
    let version = rustc_version::version().unwrap();

    if version.minor < 62 {
        println!("You are using an unsupported version of Rust, please update to 1.62 or higher.");
        println!("To update, run `rustup update`.");
        exit(1);
    }
}
