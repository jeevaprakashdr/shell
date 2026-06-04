use crate::path;

pub(crate) enum Command {
    Exit,
    Echo,
    Type,
    Exec(String),
    Unknown,
}

impl Command {
    pub(crate) fn from_bytes(cmd: Vec<u8>) -> Command {
        match cmd.as_slice() {
            b"exit" => Command::Exit,
            b"echo" => Command::Echo,
            b"type" => Command::Type,
            command if let Some(path) = path::is_executable(command) => Command::Exec(path),
            _ => Command::Unknown,
        }
    }
}
