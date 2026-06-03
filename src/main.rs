#[allow(unused_imports)]
use std::io::{self, Write};

use crate::command_line::CommadnLine;

mod command_line;

fn main() {
    let mut cli = CommadnLine::new();
    loop {
        cli.write("$ ");
        let (cmd, args) = cli.read().parse();
        match cmd.as_slice() {
            b"exit" => {
                return;
            }
            b"echo" => {
                cli.write_line(&String::from_utf8_lossy(&args));
            }
            b"type" => {
                let output = std::process::Command::new("sh")
                    .arg("-c")
                    .arg(format!("type {}", String::from_utf8(args.clone()).unwrap()))
                    .output()
                    .unwrap();

                cli.write(&String::from_utf8(output.stdout).unwrap());
            }
            _ => {
                cli.write_line(&format!(
                    "{}: command not found",
                    String::from_utf8(cmd.to_vec()).unwrap()
                ));
            }
        }
    }
}
