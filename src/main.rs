#[allow(unused_imports)]
use std::io::{self, Write};
use std::io::{BufReader, Read};

fn main() {
    let mut user_input = [0u8; 512];
    let mut reader = BufReader::new(io::stdin());

    loop {
        print!("$ ");
        io::stdout().flush().unwrap();

        let bytes_len = reader.read(&mut user_input).unwrap();
        let user_input = &user_input[..bytes_len - 1]
            .split(|p| p == " ".as_bytes().first().unwrap())
            .collect::<Vec<_>>();
        let cmd = user_input.first().unwrap();
        match &cmd[..] {
            b"exit" => {
                return;
            }
            b"echo" => {
                let space = b" ".to_vec();
                let args = user_input[1..].to_vec().join(space.first().unwrap());
                println!("{}", String::from_utf8(args).unwrap());
            }
            _ => {
                println!(
                    "{}: command not found",
                    String::from_utf8(cmd.to_vec()).unwrap()
                )
            }
        }
    }
}
