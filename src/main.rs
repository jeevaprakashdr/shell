#[allow(unused_imports)]
use std::io::{self, Write};
use std::io::{BufReader, Read};

fn main() {
    let mut buf = [0u8; 512];
    let mut reader = BufReader::new(io::stdin());

    loop {
        print!("$ ");
        io::stdout().flush().unwrap();

        let bytes_len = reader.read(&mut buf).unwrap();
        println!(
            "{}: command not found",
            String::from_utf8(buf[..bytes_len - 1].to_vec()).unwrap()
        );
    }
}
