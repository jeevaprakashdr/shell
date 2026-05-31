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
        let cmd = &user_input[..bytes_len - 1];
        match cmd {
            b"exit" => {
                return;
            }
            _ => {
                println!(
                    "{}: command not found",
                    String::from_utf8(user_input[..bytes_len - 1].to_vec()).unwrap()
                )
            }
        }
    }
}
