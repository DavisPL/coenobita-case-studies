use same_file::is_same_file;
use std::io;

fn try_main() -> Result<(), io::Error> {
    assert!(is_same_file(std::path::Path::new("/bin/sh"), std::path::Path::new("/usr/bin/sh"))?);
    Ok(())
}

fn main() {
    try_main().unwrap();
}
