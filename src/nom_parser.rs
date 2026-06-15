use nom::{
    IResult, Parser,
    branch::alt,
    bytes::complete::{tag, take_until, take_while},
    character::complete::multispace1,
    multi::separated_list0,
};

fn command(input: &[u8]) -> IResult<&[u8], &[u8]> {
    alt((
        take_until(" "),
        alphanumeric_with_special_chars("_".as_bytes()),
    ))
    .parse(input.trim_ascii())
}

fn alphanumeric_with_special_chars<'a>(
    allowed_specials: &'a [u8],
) -> impl FnMut(&'a [u8]) -> IResult<&'a [u8], &'a [u8]> {
    move |input: &[u8]| {
        take_while(|b: u8| b.is_ascii_alphanumeric() || allowed_specials.contains(&b))(input)
    }
}

fn arguments(input: &[u8]) -> IResult<&[u8], Vec<&[u8]>> {
    let (_input, text) = parse_quoted(input)?;

    separated_list0(
        multispace1,
        take_while(|b: u8| {
            b.is_ascii_alphanumeric() || matches!(b, b'_' | b'/' | b'-' | b'.' | b'~')
        }),
    )
    .parse(text)
}

fn parse_quoted(input: &[u8]) -> IResult<&[u8], &[u8]> {
    alt((
        parse_single_quoted,
        parse_double_quoted,
        alphanumeric_with_special_chars(" _/-.~".as_bytes()),
    ))
    .parse(input)
}

fn parse_single_quoted(input: &[u8]) -> IResult<&[u8], &[u8]> {
    let (input, (_, text, _)) = (
        tag("'".as_bytes()),
        alphanumeric_with_special_chars(" _/-.~".as_bytes()),
        tag("'".as_bytes()),
    )
        .parse(input)?;

    Ok((input, text))
}

fn parse_double_quoted(input: &[u8]) -> IResult<&[u8], &[u8]> {
    let (input, (_, text, _)) = (
        tag("\"".as_bytes()),
        alphanumeric_with_special_chars(" _/-.~".as_bytes()),
        tag("\"".as_bytes()),
    )
        .parse(input)?;

    Ok((input, text))
}

pub(crate) fn parse(input: &[u8]) -> IResult<&[u8], (&[u8], Vec<&[u8]>)> {
    let (input, command) = command(input.trim_ascii())?;
    let (input, args) = arguments(input.trim_ascii())?;
    Ok((input, (command, args)))
}

#[cfg(test)]
mod tests {
    use nom::branch::{alt, permutation};
    use nom::bytes::complete::tag;
    use nom::character::complete::{alpha1, digit1};
    use nom::{IResult, Parser};

    use crate::nom_parser;

    #[test]
    fn test() {
        // 'shell     world' 'script''example' test''hello shell world
    }

    #[test]
    fn alphanumeric_with_special_chars() {
        let fixture = vec![("'a'", "'", "'a'")];
        for (input, special_chars, expected) in fixture {
            let result = nom_parser::alphanumeric_with_special_chars(special_chars.as_bytes())
                .parse(input.as_bytes());

            assert_eq!(result, Ok(("".as_bytes(), expected.as_bytes())))
        }
    }

    #[test]
    fn parse_quoted() {
        let fixture = vec![
            ("'a'", "a"),
            ("'a b'", "a b"),
            ("\"a\"", "a"),
            ("\"a b\"", "a b"),
        ];
        for (input, expected) in fixture {
            let r = nom_parser::parse_quoted(input.as_bytes()).unwrap();
            assert_eq!(r.1, expected.as_bytes());
        }
    }

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
            ("/tmp/some_path", vec!["/tmp/some_path".as_bytes()]),
            ("/tmp/some-path", vec!["/tmp/some-path".as_bytes()]),
            ("./tmp", vec!["./tmp".as_bytes()]),
            ("~", vec!["~".as_bytes()]),
            ("'a b'", vec!["a".as_bytes(), "b".as_bytes()]),
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
