use std::{fs::File, io, path::Path};

pub(crate) fn create_file_with_directories<P: AsRef<Path>>(path: P) -> io::Result<File> {
    let path = path.as_ref();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    File::options().create(true).append(true).open(path)
}
