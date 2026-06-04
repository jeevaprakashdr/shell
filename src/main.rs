#[allow(unused_imports)]
use std::io::{self, Write};

use crate::{cmd::Command, command_line::CommadnLine};

mod cmd;
mod command_line;
mod path;

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
