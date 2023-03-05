use std::{
    fs::File,
    io::{self, Read, Write},
    path::Path,
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
        let _ = dst.write(data.as_bytes())?;
    }

    Ok(())
}
