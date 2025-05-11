use std::path::Path;
use std::{fs, io};

fn copy_dir(src: impl AsRef<Path>, dst: impl AsRef<Path>) -> io::Result<()> {
    if fs::exists(&dst)? {
        fs::remove_dir_all(&dst)?;
    }

    fs::create_dir_all(&dst)?;

    for entry in fs::read_dir(src)? {
        let entry = entry?;

        if entry.file_type()?.is_dir() {
            copy_dir(entry.path(), dst.as_ref().join(entry.file_name()))?;
        } else {
            fs::copy(entry.path(), dst.as_ref().join(entry.file_name()))?;
        }
    }

    Ok(())
}

fn main() {
    copy_dir(
        Path::new("examples/res"),
        Path::new(&std::env::var("OUT_DIR").unwrap()).join("res"),
    )
    .unwrap();
}
