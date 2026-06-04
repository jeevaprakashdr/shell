use crate::path;

pub(crate) enum Command {
    Exit,
    Echo,
    Type,
    Pwd,
    Cd,
    Exec(String),
    Unknown,
}

impl Command {
    pub(crate) fn from_bytes(cmd: Vec<u8>) -> Command {
        match cmd.as_slice() {
            b"exit" => Command::Exit,
            b"echo" => Command::Echo,
            b"type" => Command::Type,
            b"pwd" => Command::Pwd,
            b"cd" => Command::Cd,
            command if let Some(cmd) = path::is_executable(command) => Command::Exec(cmd),
            _ => Command::Unknown,
        }
    }
}
