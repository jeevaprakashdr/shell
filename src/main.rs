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
                println!("{}", String::from_utf8_lossy(&args));
            }
            b"type" => {}
            _ => {
                println!(
                    "{}: command not found",
                    String::from_utf8(cmd.to_vec()).unwrap()
                )
            }
        }
    }
}
