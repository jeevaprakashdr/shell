use std::io::{BufReader, BufWriter, Read, Write, stdout};

pub(crate) struct CommadnLine<R> {
    reader: BufReader<R>,
    writer: BufWriter<std::io::Stdout>,
    inner: Vec<u8>,
}

impl<R> CommadnLine<R> {
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
                    Some(byte) => {
                        current_state = State::Alphanumeric;
                        output.push(byte);
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
                    None => current_state = State::End,
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
                    Some(byte) => {
                        current_state = State::DoubleQuote;
                        output.push(byte);
                        current_byte = iterator.next()
                    }
                    None => current_state = State::End,
                },
                State::DoubleQuoteBackslash => match current_byte {
                    Some(byte) => {
                        output.push(byte);
                        current_state = State::DoubleQuote;
                        current_byte = iterator.next();
                    }
                    None => current_state = State::End,
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
                    Some(byte) => {
                        current_state = State::Alphanumeric;
                        output.push(byte);
                        current_byte = iterator.next()
                    }
                    None => current_state = State::End,
                },
                State::Backslash => match current_byte {
                    Some(byte) => {
                        current_state = State::Alphanumeric;
                        output.push(byte);
                        current_byte = iterator.next();
                    }
                    None => current_state = State::End,
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
                    Some(byte) => {
                        current_state = State::Alphanumeric;
                        output.push(byte);
                        current_byte = iterator.next()
                    }
                    None => current_state = State::End,
                },
                State::End => break,
            }
        }

        output.to_vec()
    }
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
        let last_index = |&delemeter| {
            self.inner[1..]
                .into_iter()
                .position(|&p| p == delemeter)
                .map(|pos| pos + 2)
                .unwrap()
        };

        let parsed = |index| {
            return (
                self.inner[0..index].to_vec(),
                self.inner[index..].trim_ascii().to_vec(),
            );
        };

        let single_quote = "\'".as_bytes();
        let double_quote = "\"".as_bytes();
        if self.inner.starts_with(single_quote) {
            return parsed(last_index(single_quote.first().unwrap()));
        } else if self.inner.starts_with(double_quote) {
            return parsed(last_index(double_quote.first().unwrap()));
        }

        let mut x = self.inner.splitn(2, |&b| b == b' ');
        let (cmd, args) = (x.next().unwrap_or_default(), x.next().unwrap_or_default());
        (cmd.trim_ascii().to_vec(), args.trim_ascii().to_vec())
    }
}

#[cfg(test)]
mod tests {
    use crate::command_line::CommadnLine;

    #[test]
    fn parse() {
        let reader = "'exe  with  space' /tmp/dog/f1";
        let mut cli = CommadnLine::new(reader.as_bytes());
        cli.read();

        let result = cli.parse();
        println!("{}", String::from_utf8(result.clone().0).unwrap());
        println!("{}", String::from_utf8(result.clone().1).unwrap());
        assert_eq!(result.0, "'exe  with  space'".as_bytes().to_vec());
    }

    #[test]
    fn parse_string() {
        let value = "hallo";

        let result = CommadnLine::<()>::parse_string(value.as_bytes().to_vec());

        assert_eq!(result, "hallo".as_bytes().to_vec());
    }

    #[test]
    fn parse_string_with_space() {
        let value = [
            "hallo world",
            "hallo  world",
            "hallo   world",
            "  hallo   world",
            "hallo   world  ",
            "  hallo   world  ",
        ];
        for ele in value {
            let result = CommadnLine::<()>::parse_string(ele.as_bytes().to_vec());
            assert_eq!(result, "hallo world".as_bytes().to_vec());
        }
    }

    #[test]
    fn parse_string_with_single_quote() {
        let value = [
            ("'hallo world'", "hallo world"),
            ("'hallo   world'", "hallo   world"),
            ("'hallo''world'", "halloworld"),
            ("hallo''world", "halloworld"),
        ];

        for (input, expected) in value {
            let result = CommadnLine::<()>::parse_string(input.as_bytes().to_vec());
            assert_eq!(result, expected.as_bytes().to_vec());
        }
    }

    #[test]
    fn parse_string_with_double_quote() {
        let value = [("\"shell's test\"", "shell's test")];

        for (input, expected) in value {
            let result = CommadnLine::<()>::parse_string(input.as_bytes().to_vec());
            assert_eq!(result, expected.as_bytes().to_vec());
        }
    }

    #[test]
    fn parse_string_with_backslash() {
        let value = [
            ("three\\ \\ \\ spaces", "three   spaces"),
            ("before\\     after", "before  after"),
            ("test\\nexample", "testnexample"),
            ("hello\\\\world", "hello\\world"),
            ("\\'hello\\'", "'hello'"),
        ];

        for (input, expected) in value {
            let result = CommadnLine::<()>::parse_string(input.as_bytes().to_vec());
            assert_eq!(result, expected.as_bytes().to_vec());
        }
    }
}
