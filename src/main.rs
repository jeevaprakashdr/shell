use std::io::Read;
#[allow(unused_imports)]
use std::io::{self, Write};

fn main() {
    print!("$ ");
    io::stdout().flush().unwrap();

    let mut reader = io::BufReader::new(io::stdin());
    let mut buf = [0; 512];
    let bytes_len = reader.read(&mut buf).unwrap();
    println!(
        "{}: command not found",
        String::from_utf8(buf[..bytes_len - 1].to_vec()).unwrap()
    );
}
