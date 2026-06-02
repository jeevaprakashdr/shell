use std::io::{BufReader, BufWriter, Read, Write, stdin, stdout};

pub(crate) struct CommadnLine {
    reader: BufReader<std::io::Stdin>,
    writer: BufWriter<std::io::Stdout>,
    inner: Vec<u8>,
}

impl CommadnLine {
    pub(crate) fn new() -> Self {
        Self {
            reader: BufReader::new(stdin()),
            writer: BufWriter::new(stdout()),
            inner: Vec::new(),
        }
    }

    pub(crate) fn write(&mut self, arg: &str) -> &Self {
        self.writer.write(&arg.as_bytes()).unwrap();
        self.writer.flush().unwrap();
        self
    }

    pub(crate) fn write_line(&mut self, arg: &str) -> &Self {
        self.writer.write(&arg.as_bytes()).unwrap();
        self.writer.write(b"\n").unwrap();
        self.writer.flush().unwrap();
        self
    }

    pub(crate) fn read(&mut self) -> &Self {
        let mut buf = [0u8; 512];
        let bytes_len = self.reader.read(&mut buf).unwrap();
        self.inner = buf[..bytes_len].to_vec();
        self
    }

    pub(crate) fn parse(&self) -> (Vec<u8>, Vec<u8>) {
        let mut x = self.inner.splitn(2, |&b| b == b' ');
        let (cmd, args) = (x.next().unwrap_or_default(), x.next().unwrap_or_default());
        (cmd.trim_ascii().to_vec(), args.trim_ascii().to_vec())
    }
}
