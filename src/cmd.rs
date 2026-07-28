use crate::path;

#[derive(Debug)]
pub(crate) struct Redirection {
    pub path: Option<Box<[u8]>>,
    pub append_content: bool,
}

impl Default for Redirection {
    fn default() -> Self {
        Self {
            path: Default::default(),
            append_content: Default::default(),
        }
    }
}

#[derive(Debug)]
pub(crate) struct Command {
    pub name: Vec<u8>,
    pub args: Vec<Box<[u8]>>,
    pub output_redirection: Redirection,
    pub error_redirection: Redirection,
}

impl Command {
    pub(crate) fn new(
        name: Vec<u8>,
        args: Vec<Box<[u8]>>,
        output_redirection: Redirection,
        error_redirection: Redirection,
    ) -> Self {
        Self {
            name,
            args,
            output_redirection,
            error_redirection,
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
