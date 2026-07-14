#[allow(unused_imports)]
use std::io::{self, Write};
use std::{
    env,
    ffi::OsStr,
    fs::File,
    io::BufWriter,
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
        let (cmd, args, redirection_path) = cli.read().nom_parse();
        let args: Vec<Vec<u8>> = args.iter().map(|s| s.to_vec()).collect();
        match Command::from_bytes(cmd.clone()) {
            Command::Exit => {
                return;
            }
            Command::Echo => {
                let output = args
                    .iter()
                    .map(|a| String::from_utf8(a.to_vec()).unwrap())
                    .collect::<Vec<_>>()
                    .join("");

                if redirection_path.is_empty() {
                    cli.write_line(&output);
                } else {
                    let file = File::create(String::from_utf8(redirection_path.to_vec()).unwrap())
                        .unwrap();
                    let mut writer = BufWriter::new(file);

                    writer.write_all(output.as_bytes()).unwrap();
                    writer.write_all("\n".as_bytes()).unwrap();
                    writer.flush().unwrap();
                }
            }
            Command::Type => {
                let cmd = args.first().unwrap();
                let output = std::process::Command::new("sh")
                    .arg("-c")
                    .arg(format!("type {}", String::from_utf8(cmd.to_vec()).unwrap()))
                    .output()
                    .unwrap();
                cli.write(&String::from_utf8(output.stdout).unwrap());
            }
            Command::Pwd => {
                cli.write_line(&env::current_dir().unwrap().display().to_string());
            }
            Command::Cd => {
                if args.first().unwrap() == b"~" {
                    let home = env::home_dir()
                        .map(|protobuf| protobuf.display().to_string())
                        .unwrap_or("/".to_string());
                    env::set_current_dir(home).unwrap();
                    continue;
                }

                let path = Path::new(OsStr::from_bytes(args.first().unwrap()));
                if path.exists() {
                    env::set_current_dir(path).unwrap();
                } else {
                    cli.write_line(&format!(
                        "cd: {}: No such file or directory",
                        String::from_utf8(args.first().unwrap().to_vec()).unwrap()
                    ));
                }
            }
            Command::Exec(cmd) => {
                let path = which::which(cmd.clone()).unwrap();
                let args = args
                    .iter()
                    .filter(|&p| p != b" ")
                    .map(|p| String::from_utf8(p.to_vec()).unwrap())
                    .collect::<Vec<_>>();

                let output = std::process::Command::new(path.display().to_string())
                    .arg0(cmd)
                    .args(args)
                    .output()
                    .unwrap();

                if redirection_path.is_empty() {
                    cli.write(&String::from_utf8(output.stdout).unwrap());
                } else {
                    let file = File::create(String::from_utf8(redirection_path.to_vec()).unwrap())
                        .unwrap();
                    let mut writer = BufWriter::new(file);

                    writer.write_all(&output.stdout).unwrap();
                    writer.flush().unwrap();
                }

                cli.write(&String::from_utf8(output.stderr).unwrap());
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
