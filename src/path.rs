use std::{env, path::Path};

pub(crate) fn is_executable(command: &[u8]) -> Option<String> {
    let mut singe_quoted = false;
    let mut double_quoted = false;

    let cmd = if command.starts_with("\'".as_bytes()) && command.ends_with("\'".as_bytes()) {
        singe_quoted = true;
        String::from_utf8(command[1..command.len() - 1].to_vec()).unwrap()
    } else if command.starts_with("\"".as_bytes()) && command.ends_with("\"".as_bytes()) {
        double_quoted = true;
        String::from_utf8(command[1..command.len() - 1].to_vec()).unwrap()
    } else {
        String::from_utf8(command.to_vec()).unwrap()
    };

    let paths: Vec<String> = env::var("PATH")
        .unwrap()
        .split_terminator(":")
        .map(str::to_owned)
        .collect();

    for path in paths {
        let mut path_string = format!("{}/{}", path, cmd);

        if cfg!(target_os = "linux") {
            path_string = path_string.replace("\\\\", "\\");
        }

        let path = Path::new(&path_string);

        if path.exists() {
            let path_value = if singe_quoted {
                format!("'{}'", path.to_str().unwrap().to_string())
            } else if double_quoted {
                format!("\"{}\"", path.to_str().unwrap().to_string())
            } else {
                cmd
            };

            return Some(path_value);
        }
    }

    None
}
