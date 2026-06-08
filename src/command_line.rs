use std::io::{BufReader, BufWriter, Read, SeekFrom::Current, Write, stdin, stdout};

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

    pub(crate) fn parse_string(string_bytes: Vec<u8>) -> Vec<u8> {
        let string_bytes = string_bytes.trim_ascii_start().trim_ascii_end().to_vec();
        let mut output = Vec::new();
        let mut iterator = string_bytes.into_iter();
        let mut current_byte = iterator.next();
        let mut current_state = State::Start;

        #[derive(Debug)]
        enum State {
            Start,
            SingleQuote,
            DoubleQuote,
            Backslash,
            DoubleQuoteBackslash,
            Space,
            Alphanumeric,
            End,
        }

        loop {
            match current_state {
                State::Start => match current_byte {
                    Some(b'\'') => {
                        current_state = State::SingleQuote;
                        current_byte = iterator.next()
                    }
                    Some(b'"') => {
                        current_state = State::DoubleQuote;
                        current_byte = iterator.next()
                    }
                    Some(b' ') => {
                        current_state = State::Space;
                        output.push(current_byte.unwrap());
                        current_byte = iterator.next()
                    }
                    Some(b'\\') => {
                        current_state = State::Backslash;
                        current_byte = iterator.next()
                    }
                    Some(_) => {
                        current_state = State::Alphanumeric;
                        output.push(current_byte.unwrap());
                        current_byte = iterator.next()
                    }
                    None => current_state = State::End,
                },
                State::SingleQuote => match current_byte {
                    Some(b'\'') => {
                        current_state = State::Alphanumeric;
                        current_byte = iterator.next()
                    }
                    Some(_) => {
                        current_state = State::SingleQuote;
                        output.push(current_byte.unwrap());
                        current_byte = iterator.next()
                    }
                    None => {
                        break;
                    }
                },
                State::DoubleQuote => match current_byte {
                    Some(b'"') => {
                        current_state = State::Alphanumeric;
                        current_byte = iterator.next()
                    }
                    Some(b'\\') => {
                        current_state = State::DoubleQuoteBackslash;
                        current_byte = iterator.next();
                    }
                    Some(_) => {
                        current_state = State::DoubleQuote;
                        output.push(current_byte.unwrap());
                        current_byte = iterator.next()
                    }
                    None => {
                        break;
                    }
                },
                State::DoubleQuoteBackslash => match current_byte {
                    Some(byte) => {
                        output.push(byte);
                        current_state = State::DoubleQuote;
                        current_byte = iterator.next();
                    }
                    None => break,
                },
                State::Space => match current_byte {
                    Some(b'"') => {
                        current_state = State::DoubleQuote;
                        current_byte = iterator.next()
                    }
                    Some(b'\'') => {
                        current_state = State::SingleQuote;
                        current_byte = iterator.next()
                    }
                    Some(b' ') => {
                        current_state = State::Space;
                        current_byte = iterator.next()
                    }
                    Some(b'\\') => {
                        current_state = State::Backslash;
                        current_byte = iterator.next()
                    }
                    Some(_) => {
                        current_state = State::Alphanumeric;
                        output.push(current_byte.unwrap());
                        current_byte = iterator.next()
                    }
                    None => {
                        break;
                    }
                },
                State::Backslash => match current_byte {
                    Some(byte) => {
                        current_state = State::Alphanumeric;
                        output.push(byte);
                        current_byte = iterator.next();
                    }
                    None => break,
                },
                State::Alphanumeric => match current_byte {
                    Some(b'\'') => {
                        current_state = State::SingleQuote;
                        current_byte = iterator.next()
                    }
                    Some(b'"') => {
                        current_state = State::DoubleQuote;
                        current_byte = iterator.next()
                    }
                    Some(b' ') => {
                        current_state = State::Space;
                        output.push(current_byte.unwrap());
                        current_byte = iterator.next()
                    }
                    Some(b'\\') => {
                        current_state = State::Backslash;
                        current_byte = iterator.next()
                    }
                    Some(_) => {
                        current_state = State::Alphanumeric;
                        output.push(current_byte.unwrap());
                        current_byte = iterator.next()
                    }
                    None => {
                        break;
                    }
                },
                State::End => break,
            }
        }

        output.to_vec()
    }
}

#[cfg(test)]
mod tests {
    use crate::command_line::CommadnLine;

    #[test]
    fn parse_text() {
        let value = "hallo";

        let result = CommadnLine::parse_string(value.as_bytes().to_vec());

        assert_eq!(result, "hallo".as_bytes().to_vec());
    }

    #[test]
    fn parse_text_with_space() {
        let value = [
            "hallo world",
            "hallo  world",
            "hallo   world",
            "  hallo   world",
            "hallo   world  ",
            "  hallo   world  ",
        ];
        for ele in value {
            let result = CommadnLine::parse_string(ele.as_bytes().to_vec());
            assert_eq!(result, "hallo world".as_bytes().to_vec());
        }
    }

    #[test]
    fn parse_text_with_single_quote() {
        let value = [
            ("'hallo world'", "hallo world"),
            ("'hallo   world'", "hallo   world"),
            ("'hallo''world'", "halloworld"),
            ("hallo''world", "halloworld"),
        ];

        for (input, expected) in value {
            let result = CommadnLine::parse_string(input.as_bytes().to_vec());
            assert_eq!(result, expected.as_bytes().to_vec());
        }
    }

    #[test]
    fn parse_text_with_double_quote() {
        let value = [("\"shell's test\"", "shell's test")];

        for (input, expected) in value {
            let result = CommadnLine::parse_string(input.as_bytes().to_vec());
            assert_eq!(result, expected.as_bytes().to_vec());
        }
    }

    #[test]
    fn parse_text_with_backslash() {
        let value = [
            ("three\\ \\ \\ spaces", "three   spaces"),
            ("before\\     after", "before  after"),
            ("test\\nexample", "testnexample"),
            ("hello\\\\world", "hello\\world"),
            ("\\'hello\\'", "'hello'"),
        ];

        for (input, expected) in value {
            let result = CommadnLine::parse_string(input.as_bytes().to_vec());
            assert_eq!(result, expected.as_bytes().to_vec());
        }
    }
}
