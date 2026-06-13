#[allow(unused_imports)]
use std::io::{self, Write};
use std::{
    env,
    ffi::OsStr,
    os::unix::{ffi::OsStrExt, process::CommandExt},
    path::Path,
};

use crate::{cmd::Command, command_line::CommadnLine};

mod cmd;
mod command_line;
mod nom_parser;
mod path;

fn main() {
    let mut cli = CommadnLine::new(std::io::stdin());
    loop {
        cli.write("$ ");
        let (cmd, args) = cli.read().nom_parse();
        match Command::from_bytes(cmd.clone()) {
            Command::Exit => {
                return;
            }
            Command::Echo => {
                let output = args
                    .iter()
                    .map(|a| String::from_utf8(a.to_vec()).unwrap())
                    .collect::<Vec<_>>()
                    .join(" ");
                cli.write_line(&output);
            }
            Command::Type => {
                let arg = args
                    .iter()
                    .map(|a| String::from_utf8(a.to_vec()).unwrap())
                    .collect::<Vec<_>>()
                    .first()
                    .unwrap()
                    .clone();
                let output = std::process::Command::new("sh")
                    .arg("-c")
                    .arg(format!("type {}", arg))
                    .output()
                    .unwrap();
                cli.write(&String::from_utf8(output.stdout).unwrap());

                // let binary_name = String::from_utf8(args.first().unwrap().to_vec()).unwrap();
                // match which::which(binary_name.clone()) {
                //     Ok(_) => cli.write_line(&format!("{} is a shell builtin", binary_name)),
                //     Err(e) => cli.write_line(&e.to_string()),
                // };
            }
            Command::Pwd => {
                cli.write_line(&env::current_dir().unwrap().display().to_string());
            }
            Command::Cd => {
                // if args == b"~" {
                //     let home = env::home_dir()
                //         .map(|protobuf| protobuf.display().to_string())
                //         .unwrap_or("/".to_string());
                //     env::set_current_dir(home).unwrap();
                //     continue;
                // }

                // let path = Path::new(OsStr::from_bytes(args.as_slice()));
                // if path.exists() {
                //     env::set_current_dir(path).unwrap();
                // } else {
                //     cli.write_line(&format!(
                //         "cd: {}: No such file or directory",
                //         String::from_utf8(args).unwrap()
                //     ));
                // }
            }
            Command::Exec(cmd) => {
                let path = which::which(cmd.clone()).unwrap();
                let args = args
                    .iter()
                    .map(|a| String::from_utf8(a.to_vec()).unwrap())
                    .collect::<Vec<_>>();
                let output = std::process::Command::new(path.display().to_string())
                    .arg0(cmd)
                    .args(args)
                    .output()
                    .unwrap();
                // let output = std::process::Command::new("sh")
                //     .arg("-c")
                //     .arg(format!(
                //         "{cmd} {}",
                //         String::from_utf8(args.clone()).unwrap()
                //     ))
                //     .output()
                //     .unwrap();
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
