use std::io::{BufReader, BufWriter, Read, Write, stdout};

use crate::{cmd::Command, nom_parser};

pub(crate) struct CommadnLine<R> {
    reader: BufReader<R>,
    writer: BufWriter<std::io::Stdout>,
    inner: Vec<u8>,
}

impl<R> CommadnLine<R>
where
    R: Read,
{
    pub(crate) fn new(reader: R) -> Self {
        Self {
            reader: BufReader::new(reader),
            writer: BufWriter::new(stdout()),
            inner: Vec::new(),
        }
    }

    pub(crate) fn write(&mut self, arg: &str) -> &Self {
        self.writer.write_all(arg.as_bytes()).unwrap();
        self.writer.flush().unwrap();
        self
    }

    pub(crate) fn write_line(&mut self, arg: &str) -> &Self {
        self.writer.write_all(arg.as_bytes()).unwrap();
        self.writer.write_all(b"\n").unwrap();
        self.writer.flush().unwrap();
        self
    }

    pub(crate) fn read(&mut self) -> &Self {
        let mut buf = [0u8; 512];
        let bytes_len = self.reader.read(&mut buf).unwrap();
        self.inner = buf[..bytes_len].to_vec();
        self
    }

    pub(crate) fn nom_parse(&self) -> Result<Command, &str> {
        match nom_parser::parse(&self.inner) {
            Ok((_, cmd)) => Ok(cmd),
            Err(_) => Err("Failed to process command"),
        }
    }
}
