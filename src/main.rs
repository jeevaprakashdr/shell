#[allow(unused_imports)]
use std::io::{self, Write};
use std::{env, path::Path};

use crate::command_line::CommadnLine;

mod command_line;

fn main() {
    let mut cli = CommadnLine::new();
    loop {
        cli.write("$ ");
        let (cmd, args) = cli.read().parse();
        match Command::from_bytes(cmd.clone()) {
            Command::Exit => {
                return;
            }
            Command::Echo => {
                cli.write_line(&String::from_utf8_lossy(&args));
            }
            Command::Type => {
                let output = std::process::Command::new("sh")
                    .arg("-c")
                    .arg(format!("type {}", String::from_utf8(args.clone()).unwrap()))
                    .output()
                    .unwrap();
                cli.write(&String::from_utf8(output.stdout).unwrap());
            }
            Command::Exec(cmd) => {
                let output = std::process::Command::new("sh")
                    .arg("-c")
                    .arg(format!(
                        "{cmd} {}",
                        String::from_utf8(args.clone()).unwrap()
                    ))
                    .output()
                    .unwrap();
                cli.write(&String::from_utf8(output.stdout).unwrap());
            }
            Command::Unknown => {
                cli.write_line(&format!(
                    "{}: command not found",
                    String::from_utf8(cmd.to_vec()).unwrap()
                ));
            }
        }
    }
}

pub(crate) enum Command {
    Exit,
    Echo,
    Type,
    Exec(String),
    Unknown,
}

impl Command {
    fn from_bytes(cmd: Vec<u8>) -> Command {
        match cmd.as_slice() {
            b"exit" => Command::Exit,
            b"echo" => Command::Echo,
            b"type" => Command::Type,
            command if let Some(path) = is_executable(command) => Command::Exec(path),
            _ => Command::Unknown,
        }
    }
}

fn is_executable(command: &[u8]) -> Option<String> {
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
