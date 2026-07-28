use std::{fs::File, io, path::Path};

pub(crate) fn create_file_with_directories<P: AsRef<Path>>(
    path: P,
    append: bool,
) -> io::Result<File> {
    let path = path.as_ref();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    File::options()
        .write(true)
        .append(append)
        .create(true)
        .truncate(!append)
        .open(path)
}
