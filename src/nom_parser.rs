use nom::{
    AsChar, IResult, Parser,
    branch::alt,
    bytes::complete::{tag, take_until, take_while},
    multi::separated_list0,
};

fn command(input: &[u8]) -> IResult<&[u8], &[u8]> {
    alt((
        take_until(" "),
        take_while(|b: u8| b.is_ascii_alphanumeric() || matches!(b, b'_')),
    ))
    .parse(input.trim_ascii())
}

fn arguments(input: &[u8]) -> IResult<&[u8], Vec<&[u8]>> {
    separated_list0(
        tag(" "),
        take_while(|b: u8| b.is_ascii_alphanumeric() || matches!(b, b'_')),
    )
    .parse(input)
}

pub(crate) fn parse(input: &[u8]) -> IResult<&[u8], (&[u8], Vec<&[u8]>)> {
    let (input, command) = command(input.trim_ascii())?;
    let (input, args) = arguments(input.trim_ascii())?;
    Ok((input, (command, args)))
}

#[cfg(test)]
mod tests {
    use crate::nom_parser;

    #[test]
    fn arguments() {
        let fixture = vec![
            ("a", vec!["a".as_bytes()]),
            ("a_1", vec!["a_1".as_bytes()]),
            ("a b", vec!["a".as_bytes(), "b".as_bytes()]),
            (
                "a b c",
                vec!["a".as_bytes(), "b".as_bytes(), "c".as_bytes()],
            ),
        ];

        for (input, expected) in fixture {
            let r = nom_parser::arguments(input.as_bytes()).unwrap();
            assert_eq!(r.1, expected);
        }
    }

    #[test]
    fn command() {
        let fixture = vec![
            ("abc", "abc"),
            (" abc", "abc"),
            (" abc ", "abc"),
            ("abc_123", "abc_123"),
            ("abc xyz", "abc"),
            ("abc123 xyz", "abc123"),
            ("abc_123 xyz", "abc_123"),
            ("invalid_orange_command", "invalid_orange_command"),
        ];

        for (input, expected) in fixture {
            let r = nom_parser::command(input.as_bytes()).unwrap();
            assert_eq!(r.1, expected.as_bytes());
        }
    }

    #[test]
    fn parse() {
        let fixture = vec![
            ("cmd", ("cmd", vec![vec![]])),
            (" cmd", ("cmd", vec![vec![]])),
            (" cmd ", ("cmd", vec![vec![]])),
            ("cmd a", ("cmd", vec!["a".as_bytes().to_vec()])),
            ("cmd a_1", ("cmd", vec!["a_1".as_bytes().to_vec()])),
            (
                "cmd a b",
                (
                    "cmd",
                    vec!["a".as_bytes().to_vec(), "b".as_bytes().to_vec()],
                ),
            ),
            (
                "cmd a b c",
                (
                    "cmd",
                    vec![
                        "a".as_bytes().to_vec(),
                        "b".as_bytes().to_vec(),
                        "c".as_bytes().to_vec(),
                    ],
                ),
            ),
        ];

        for (input, (expected_cmd, expected_args)) in fixture {
            let (_input, (cmd, args)) = nom_parser::parse(input.as_bytes()).unwrap();
            assert_eq!(cmd, expected_cmd.as_bytes());
            assert_eq!(args, expected_args);
        }
    }
}
