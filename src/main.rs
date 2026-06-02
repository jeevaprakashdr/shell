#[allow(unused_imports)]
use std::io::{self, Write};

use crate::command_line::CommadnLine;

mod command_line;

fn main() {
    let mut cli = CommadnLine::new();
    loop {
        cli.write("$ ");
        let (cmd, args) = cli.read().parse();
        match &cmd[..] {
            b"exit" => {
                return;
            }
            b"echo" => {
                // println!("{}", String::from_utf8_lossy(&args));
                cli.write_line(&String::from_utf8_lossy(&args));
            }
            b"type" => match args {
                val if val == "exit".as_bytes()
                    || val == "echo".as_bytes()
                    || val == "type".as_bytes() =>
                {
                    cli.write_line(&format!(
                        "{} is a shell builtin",
                        String::from_utf8(val).unwrap()
                    ));
                    // println!("{} is a shell builtin", String::from_utf8(val).unwrap());
                }
                _ => {
                    // println!("{}: not found", String::from_utf8(args).unwrap());
                    cli.write_line(&format!("{}: not found", String::from_utf8(args).unwrap()));
                }
            },
            _ => {
                cli.write_line(&format!(
                    "{}: command not found",
                    String::from_utf8(cmd.to_vec()).unwrap()
                ));
            }
        }
    }
}
