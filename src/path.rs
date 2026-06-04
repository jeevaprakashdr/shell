use std::{env, path::Path};

pub(crate) fn is_executable(command: &[u8]) -> Option<String> {
    let path = env::var("PATH").unwrap();
    let cmd = String::from_utf8(command.to_vec()).unwrap();

    let paths: Vec<String> = path.split_terminator(":").map(str::to_owned).collect();
    for path in paths {
        let p = Path::new(&path).join(cmd.clone());
        if p.exists() {
            return Some(cmd);
        }
    }

    None
}
