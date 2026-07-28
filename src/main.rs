#[allow(unused_imports)]
use std::io::{self, Write};
use std::{
    env,
    ffi::OsStr,
    io::BufWriter,
    os::unix::{ffi::OsStrExt, process::CommandExt},
    path::Path,
    process::exit,
};

use crate::{cmd::CommandType, command_line::CommadnLine};

mod cmd;
mod command_line;
mod file_utility;
mod nom_parser;
mod path;

fn main() {
    let mut cli = CommadnLine::new(std::io::stdin());
    loop {
        cli.write("$ ");
        let cmd = match cli.read().nom_parse() {
            Ok(cmd) => cmd,
            Err(e) => {
                println!("{e}");
                exit(1)
            }
        };

        match CommandType::from_bytes(cmd.name.clone()) {
            CommandType::Exit => {
                return;
            }
            CommandType::Echo => {
                let output = cmd
                    .args
                    .iter()
                    .map(|a| String::from_utf8(a.to_vec()).unwrap())
                    .collect::<Vec<_>>()
                    .join("");

                if let Some(path) = cmd.output_redirection.path {
                    let file = file_utility::create_file_with_directories(
                        String::from_utf8(path.to_vec()).unwrap(),
                        cmd.output_redirection.append_content,
                    )
                    .unwrap();
                    let mut writer = BufWriter::new(file);

                    writer.write_all(output.as_bytes()).unwrap();
                    writer.write_all("\n".as_bytes()).unwrap();
                    writer.flush().unwrap();
                } else {
                    cli.write_line(&output);
                }

                if let Some(path) = cmd.error_redirection.path {
                    let _ = file_utility::create_file_with_directories(
                        String::from_utf8(path.to_vec()).unwrap(),
                        false,
                    );
                }
            }
            CommandType::Type => {
                let cmd = cmd.args.first().unwrap();
                let output = std::process::Command::new("sh")
                    .arg("-c")
                    .arg(format!("type {}", String::from_utf8(cmd.to_vec()).unwrap()))
                    .output()
                    .unwrap();
                cli.write(&String::from_utf8(output.stdout).unwrap());
            }
            CommandType::Pwd => {
                cli.write_line(&env::current_dir().unwrap().display().to_string());
            }
            CommandType::Cd => {
                let first_arg = cmd.args.first().unwrap().as_ref();
                if first_arg == b"~" {
                    let home = env::home_dir()
                        .map(|protobuf| protobuf.display().to_string())
                        .unwrap_or("/".to_string());
                    env::set_current_dir(home).unwrap();
                    continue;
                }

                let path = Path::new(OsStr::from_bytes(first_arg));
                if path.exists() {
                    env::set_current_dir(path).unwrap();
                } else {
                    cli.write_line(&format!(
                        "cd: {}: No such file or directory",
                        String::from_utf8(first_arg.to_vec()).unwrap()
                    ));
                }
            }
            CommandType::Exec(cmd_name) => {
                let path = which::which(cmd_name.clone()).unwrap();
                let args = cmd
                    .args
                    .iter()
                    .filter(|&p| p.as_ref() != b" ")
                    .map(|p| String::from_utf8(p.to_vec()).unwrap())
                    .collect::<Vec<_>>();

                let output = std::process::Command::new(path.display().to_string())
                    .arg0(cmd_name)
                    .args(args)
                    .output()
                    .unwrap();

                if let Some(path) = cmd.output_redirection.path {
                    let file = file_utility::create_file_with_directories(
                        String::from_utf8(path.to_vec()).unwrap(),
                        cmd.output_redirection.append_content,
                    )
                    .unwrap();

                    let mut writer = BufWriter::new(file);
                    writer.write_all(&output.stdout).unwrap();
                    writer.flush().unwrap();
                } else {
                    cli.write(&String::from_utf8(output.stdout).unwrap());
                }

                if let Some(path) = cmd.error_redirection.path {
                    let file = file_utility::create_file_with_directories(
                        String::from_utf8(path.to_vec()).unwrap(),
                        cmd.error_redirection.append_content,
                    )
                    .unwrap();
                    let mut writer = BufWriter::new(file);
                    writer.write_all(&output.stderr).unwrap();
                    writer.flush().unwrap();
                } else {
                    cli.write(&String::from_utf8(output.stderr).unwrap());
                }
            }
            CommandType::Unknown => {
                cli.write_line(&format!(
                    "{}: command not found",
                    String::from_utf8(cmd.name.to_vec()).unwrap()
                ));
            }
        }
    }
}
