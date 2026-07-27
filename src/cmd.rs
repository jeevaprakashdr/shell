use crate::path;

#[derive(Debug)]
pub(crate) struct Command {
    pub name: Vec<u8>,
    pub args: Vec<Box<[u8]>>,
    pub redirection_path: Option<Box<[u8]>>,
    pub error_redirection_path: Option<Box<[u8]>>,
}

impl Command {
    pub(crate) fn new(
        name: Vec<u8>,
        args: Vec<Box<[u8]>>,
        redirection_path: Option<Box<[u8]>>,
        error_redirection_path: Option<Box<[u8]>>,
    ) -> Self {
        Self {
            name,
            args,
            redirection_path,
            error_redirection_path,
        }
    }
}

#[derive(Debug)]
pub(crate) enum CommandType {
    Exit,
    Echo,
    Type,
    Pwd,
    Cd,
    Exec(String),
    Unknown,
}

impl CommandType {
    pub(crate) fn from_bytes(cmd: Vec<u8>) -> CommandType {
        if cmd.is_empty() {
            return CommandType::Unknown;
        }

        match cmd.as_slice() {
            b"exit" => CommandType::Exit,
            b"echo" => CommandType::Echo,
            b"type" => CommandType::Type,
            b"pwd" => CommandType::Pwd,
            b"cd" => CommandType::Cd,
            command if let Some(cmd) = path::is_executable(command) => CommandType::Exec(cmd),
            _ => CommandType::Unknown,
        }
    }
}
