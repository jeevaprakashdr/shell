use nom::error::Error;
use nom::{
    IResult, Parser,
    branch::alt,
    bytes::complete::{escaped_transform, tag, take, take_while1},
    character::complete::{alpha1, multispace1},
    combinator::{map, not, peek, value},
    error::ErrorKind::{self},
    multi::{fold_many0, many0, many1},
    sequence::preceded,
};

use crate::cmd::{Command, Redirection};

fn alphanumeric_with_special_chars<'a>(
    allowed_specials: &'a [u8],
) -> impl FnMut(&'a [u8]) -> IResult<&'a [u8], Box<[u8]>> {
    move |input: &[u8]| {
        let (i, o) =
            take_while1(|b: u8| b.is_ascii_alphanumeric() || allowed_specials.contains(&b))(input)?;
        Ok((i, o.to_vec().into_boxed_slice()))
    }
}

fn command(input: &[u8]) -> IResult<&[u8], Box<[u8]>> {
    let (i, o) = alt((
        single_quoted_with(" \"".as_bytes()),
        double_quoted_with(" '".as_bytes()),
        unquoted_with("_".as_bytes()),
    ))
    .parse(input.trim_ascii())?;

    Ok((i, o))
}

fn arguments(input: &[u8]) -> IResult<&[u8], Box<[u8]>> {
    let (input, _) = not(peek(redirection)).parse(input)?;
    let (input, _) = not(peek(error_redirection)).parse(input)?;

    alt((
        map(multispace1, |_| Box::from(" ".as_bytes())),
        list_directory_or_file,
        unquoted_with("_:".as_bytes()),
        unquoted,
        absolute_path("-_.".as_bytes()),
        relative_path,
        unquoted_backslash(" _".as_bytes()),
        home,
        empty_single_quoted,
        single_quoted_with(" ".as_bytes()),
        single_quoted_absolute_path,
        empty_double_quoted,
        double_quoted_with(" '".as_bytes()),
        double_quoted_absolute_path,
    ))
    .parse(input)
}

fn error_redirection_argument(input: &[u8]) -> IResult<&[u8], Option<Box<[u8]>>> {
    match preceded(
        tag("2>"),
        alphanumeric_with_special_chars(" /_.".as_bytes()),
    )
    .parse(input)
    {
        Ok((i, o)) => Ok((i, Some(Box::from(o.trim_ascii())))),
        Err(_) => Ok((input, None)),
    }
}

fn redirection_argument(input: &[u8]) -> IResult<&[u8], Redirection> {
    let append_content = match preceded(
        alt((tag("1>>"), tag(">>"))),
        alphanumeric_with_special_chars(" /_.".as_bytes()),
    )
    .parse(input)
    {
        Ok(_) => true,
        Err(_) => false,
    };

    let (i, o) = preceded(
        alt((tag("1>>"), tag(">>"), tag("1>"), tag(">"))),
        alphanumeric_with_special_chars(" /_.".as_bytes()),
    )
    .parse(input)?;

    let redirection = Redirection {
        path: Some(Box::from(o.trim_ascii())),
        append_content,
    };

    Ok((i, redirection))
}

fn path_component<'a>(
    allowed_chars: &'a [u8],
) -> impl FnMut(&'a [u8]) -> IResult<&'a [u8], Box<[u8]>> {
    move |input: &[u8]| {
        let (i, o) = (
            tag("/"),
            alt((
                map(tag(".."), Box::from),
                unquoted_backslash(allowed_chars),
                single_quoted_path_component(),
                double_quoted_path_component(),
            )),
        )
            .parse(input)?;

        Ok((i, [o.0, &o.1[..]].concat().into_boxed_slice()))
    }
}

fn absolute_path<'a>(
    allowed_specials: &'a [u8],
) -> impl FnMut(&'a [u8]) -> IResult<&'a [u8], Box<[u8]>> {
    move |input: &[u8]| {
        let (i, o) = many1(path_component(allowed_specials)).parse(input)?;
        Ok((i, o.concat().into_boxed_slice()))
    }
}

fn relative_path(input: &[u8]) -> IResult<&[u8], Box<[u8]>> {
    let (i, o) = (
        alt((tag(".."), tag("."))),
        many1(path_component(" _".as_bytes())),
    )
        .parse(input)?;
    Ok((i, [o.0, &o.1.concat()[..]].concat().into_boxed_slice()))
}

fn list_directory_or_file(input: &[u8]) -> IResult<&[u8], Box<[u8]>> {
    let (i, o) = tag("-1").parse(input)?;

    Ok((i, o.to_vec().into_boxed_slice()))
}

fn redirection(input: &[u8]) -> IResult<&[u8], Box<[u8]>> {
    let (_, _) = alt((tag("1>"), tag(">"))).parse(input)?;

    Ok(("".as_bytes(), Box::from(input)))
}

fn error_redirection(input: &[u8]) -> IResult<&[u8], Box<[u8]>> {
    let (_, _) = tag("2>").parse(input)?;

    Ok(("".as_bytes(), Box::from(input)))
}

fn home(input: &[u8]) -> IResult<&[u8], Box<[u8]>> {
    let (i, o) = tag("~").parse(input)?;
    Ok((i, o.to_vec().into_boxed_slice()))
}

fn unquoted(input: &[u8]) -> IResult<&[u8], Box<[u8]>> {
    let (i, o) = alpha1.parse(input)?;
    Ok((i, o.to_vec().into_boxed_slice()))
}

fn unquoted_backslash<'a>(
    allowed_chars: &'a [u8],
) -> impl FnMut(&'a [u8]) -> IResult<&'a [u8], Box<[u8]>> {
    move |input: &[u8]| {
        if input.is_empty() {
            return Err(nom::Err::Error(Error::new(input, ErrorKind::Eof)));
        }

        escaped_transform(
            take_while1(|c: u8| {
                c != b'\\' && c.is_ascii_alphanumeric() || allowed_chars.contains(&c)
            }),
            '\\',
            alt((
                value("\'".as_bytes(), tag("'".as_bytes())),
                value("\"".as_bytes(), tag("\"".as_bytes())),
                value("n".as_bytes(), tag("n")),
                value("_".as_bytes(), tag("_")),
                value("\\".as_bytes(), tag("\\")),
                take(1usize),
            )),
        )
        .map(|o| o.into_boxed_slice())
        .parse(input)
    }
}

fn unquoted_with<'a>(
    allowed_chars: &'a [u8],
) -> impl FnMut(&'a [u8]) -> IResult<&'a [u8], Box<[u8]>> {
    move |input: &[u8]| {
        let (i, o) = alphanumeric_with_special_chars(allowed_chars).parse(input)?;
        Ok((i, o.to_vec().into_boxed_slice()))
    }
}

fn quoted_path_component<'a>(
    allowed_chars: &'a [u8],
) -> impl FnMut(&'a [u8]) -> IResult<&'a [u8], Box<[u8]>> {
    move |input: &[u8]| {
        fold_many0(
            alt((
                value(b"\"".to_vec(), tag("\\\"")),
                value(b"\\".to_vec(), tag("\\\\")),
                value(b"\\".to_vec(), tag("\\")),
                map(
                    take_while1(|c: u8| c.is_ascii_alphanumeric() || allowed_chars.contains(&c)),
                    |slice: &[u8]| slice.to_vec(),
                ),
            )),
            Vec::new,
            |mut acc, item| {
                acc.extend(item);
                acc
            },
        )
        .map(|o| o.into_boxed_slice())
        .parse(input)
    }
}

fn single_quoted_backslash<'a>(
    allowed_chars: &'a [u8],
) -> impl FnMut(&'a [u8]) -> IResult<&'a [u8], Box<[u8]>> {
    move |input: &[u8]| {
        if input.is_empty() {
            return Err(nom::Err::Error(Error::new(input, ErrorKind::Eof)));
        }

        escaped_transform(
            take_while1(|c: u8| c.is_ascii_alphanumeric() || allowed_chars.contains(&c)),
            '\\',
            alt((
                value("\\\"".as_bytes(), tag("\"".as_bytes())),
                value("\\\\".as_bytes(), tag("\\".as_bytes())),
                value("\\n".as_bytes(), tag("n")),
            )),
        )
        .map(|o| o.into_boxed_slice())
        .parse(input)
    }
}

fn double_quoted_backslash<'a>(
    allowed_chars: &'a [u8],
) -> impl FnMut(&'a [u8]) -> IResult<&'a [u8], Box<[u8]>> {
    move |input: &[u8]| {
        if input.is_empty() {
            return Err(nom::Err::Error(Error::new(input, ErrorKind::Eof)));
        }

        escaped_transform(
            take_while1(|c: u8| c.is_ascii_alphanumeric() || allowed_chars.contains(&c)),
            '\\',
            alt((
                value("\"".as_bytes(), tag("\"".as_bytes())),
                value("\\".as_bytes(), tag("\\".as_bytes())),
                value("\\n".as_bytes(), tag("n")),
            )),
        )
        .map(|o| o.into_boxed_slice())
        .parse(input)
    }
}

fn empty_single_quoted(input: &[u8]) -> IResult<&[u8], Box<[u8]>> {
    map(tag("''"), |_| Box::from("".as_bytes())).parse(input)
}

fn single_quoted_with<'a>(
    allowed_chars: &'a [u8],
) -> impl FnMut(&'a [u8]) -> IResult<&'a [u8], Box<[u8]>> {
    move |input: &[u8]| {
        let (i, (_, o, _)) = (
            tag("'".as_bytes()),
            alt((single_quoted_backslash(allowed_chars), unquoted)),
            tag("'".as_bytes()),
        )
            .parse(input)?;
        Ok((i, o.to_vec().into_boxed_slice()))
    }
}

fn single_quoted_path_component<'a>() -> impl FnMut(&'a [u8]) -> IResult<&'a [u8], Box<[u8]>> {
    move |input: &[u8]| {
        let (i, (_, o, _)) = (
            tag("'".as_bytes()),
            alt((quoted_path_component(" ".as_bytes()), unquoted)),
            tag("'".as_bytes()),
        )
            .parse(input)?;
        Ok((i, o.to_vec().into_boxed_slice()))
    }
}

fn double_quoted_path_component<'a>() -> impl FnMut(&'a [u8]) -> IResult<&'a [u8], Box<[u8]>> {
    move |input: &[u8]| {
        let (i, (_, o, _)) = (
            tag("\"".as_bytes()),
            alt((quoted_path_component(" ".as_bytes()), unquoted)),
            tag("\"".as_bytes()),
        )
            .parse(input)?;
        Ok((i, o.to_vec().into_boxed_slice()))
    }
}

fn single_quoted_absolute_path(input: &[u8]) -> IResult<&[u8], Box<[u8]>> {
    let (i, (_, o, _)) = (
        tag("'".as_bytes()),
        absolute_path(" _".as_bytes()),
        tag("'".as_bytes()),
    )
        .parse(input)?;

    Ok((i, o.to_vec().into_boxed_slice()))
}

fn empty_double_quoted(input: &[u8]) -> IResult<&[u8], Box<[u8]>> {
    map(tag("\"\""), |_| Box::from("".as_bytes())).parse(input)
}

fn double_quoted_with<'a>(
    allowed_chars: &'a [u8],
) -> impl FnMut(&'a [u8]) -> IResult<&'a [u8], Box<[u8]>> {
    move |input: &[u8]| {
        let (i, (_, o, _)) = (
            tag("\"".as_bytes()),
            alt((double_quoted_backslash(allowed_chars), unquoted)),
            tag("\"".as_bytes()),
        )
            .parse(input)?;
        Ok((i, o.to_vec().into_boxed_slice()))
    }
}

fn double_quoted_absolute_path(input: &[u8]) -> IResult<&[u8], Box<[u8]>> {
    let (i, (_, o, _)) = (
        tag("\"".as_bytes()),
        absolute_path(" '".as_bytes()),
        tag("\"".as_bytes()),
    )
        .parse(input)?;

    Ok((i, o))
}

pub(crate) fn parse(input: &[u8]) -> IResult<&[u8], Command> {
    let (i, c) = command.parse(input.trim_ascii())?;
    if i.is_empty() {
        return Ok((
            i,
            Command::new(c.to_vec(), Vec::new(), Redirection::default(), None),
        ));
    }

    let (i, args) = many0(arguments).parse(i.trim_ascii())?;
    if i.is_empty() || i == "/".as_bytes() {
        return Ok((
            i,
            Command::new(c.to_vec(), args, Redirection::default(), None),
        ));
    }

    let (i, error_redirection_path) = error_redirection_argument.parse(i.trim_ascii())?;
    if error_redirection_path.is_some() {
        return Ok((
            i,
            Command::new(
                c.to_vec(),
                args,
                Redirection::default(),
                error_redirection_path,
            ),
        ));
    }

    let (i, output_redirection) = redirection_argument.parse(i.trim_ascii())?;
    Ok((
        i,
        Command::new(c.to_vec(), args, output_redirection, error_redirection_path),
    ))
}

#[cfg(test)]
mod tests {
    use nom::Parser;

    use crate::nom_parser;

    #[test]
    fn list_directory_or_file() {
        let fixture = vec![("-1 abc".as_bytes(), "-1")];
        for (input, expected) in fixture {
            let result = nom_parser::list_directory_or_file(input);

            assert!(result.is_ok());
            let actual = result.unwrap();
            assert_eq!(actual.1.as_ref(), expected.as_bytes());
            assert_eq!(actual.0.as_ref(), " abc".as_bytes());
        }
    }

    #[test]
    fn redirection() {
        let fixture = vec![("1> abc".as_bytes()), ("> xyz".as_bytes())];

        for input in fixture {
            let result = nom_parser::redirection(input);

            assert!(result.is_ok());
            let actual = result.unwrap();
            assert_eq!(actual.1.as_ref(), input);
        }
    }

    #[test]
    fn error_redirection() {
        let fixture = vec![("2> abc".as_bytes())];

        for input in fixture {
            let result = nom_parser::error_redirection(input);

            assert!(result.is_ok());
            let actual = result.unwrap();
            assert_eq!(actual.1.as_ref(), input);
        }
    }

    #[test]
    fn unquoted() {
        let fixture = vec![("test".as_bytes(), "test".as_bytes())];
        for (input, expected) in fixture {
            let result = nom_parser::unquoted(input);

            assert!(result.is_ok());
            let actual = result.unwrap();
            assert_eq!(actual.1.as_ref(), expected);
        }
    }

    #[test]
    fn unquoted_backslash() {
        let fixture = vec![
            (
                "pear apple strawberry".as_bytes(),
                "pear apple strawberry".as_bytes(),
            ),
            (
                "hello\\ \\ \\ \\ \\ \\ world".as_bytes(),
                "hello      world".as_bytes(),
            ),
            ("\\'\\\"e\\\"\\'".as_bytes(), "\'\"e\"\'".as_bytes()),
            ("s\\ne".as_bytes(), "sne".as_bytes()),
            ("\\_ignored_2".as_bytes(), "_ignored_2".as_bytes()),
            ("_ignored_\\\\_2".as_bytes(), "_ignored_\\_2".as_bytes()),
        ];
        for (input, expected) in fixture {
            let result = nom_parser::unquoted_backslash(b" _").parse(input);

            let actual = result.unwrap();
            println!(
                "actual {}",
                String::from_utf8(actual.clone().1.to_vec()).unwrap()
            );
            println!("expected {}", String::from_utf8(expected.to_vec()).unwrap());
            assert_eq!(actual.1.as_ref(), expected);
        }
    }

    #[test]
    fn unquoted_with() {
        let fixture = vec![
            ("test_1".as_bytes(), "test_1".as_bytes()),
            ("test".as_bytes(), "test".as_bytes()),
        ];
        for (input, expected) in fixture {
            let result = nom_parser::unquoted_with("_".as_bytes()).parse(input);

            assert!(result.is_ok());
            let actual = result.unwrap();
            assert_eq!(actual.1.as_ref(), expected);
        }
    }

    #[test]
    fn empty_single_quoted() {
        let fixture = vec![("''".as_bytes(), "".as_bytes())];
        for (input, expected) in fixture {
            let result = nom_parser::empty_single_quoted(input);

            assert!(result.is_ok());
            let actual = result.unwrap();
            assert_eq!(actual.1.as_ref(), expected);
        }
    }

    #[test]
    fn single_quoted_with() {
        let fixture = vec![
            (
                "'a b'".as_bytes(),
                vec!["a".as_bytes(), " ".as_bytes(), "b".as_bytes()],
            ),
            (
                "'shell\\nhello'".as_bytes(),
                vec!["shell\\nhello".as_bytes()],
            ),
            (
                "'example\\\"scripttest\\\"world'".as_bytes(),
                vec!["example\\\"scripttest\\\"world".as_bytes()],
            ),
        ];
        for (input, expected) in fixture {
            let result = nom_parser::single_quoted_with(" ".as_bytes()).parse(input);

            assert!(result.is_ok());
            let actual = result.unwrap();
            println!(
                "actual {}",
                String::from_utf8(actual.clone().1.to_vec()).unwrap()
            );
            println!(
                "expected {}",
                String::from_utf8(expected.clone().concat()).unwrap()
            );
            assert_eq!(actual.1.as_ref(), &expected.concat());
        }
    }

    #[test]
    fn single_quoted_absolute_path() {
        let fixture = vec![("'/tmp/cow/f   54'".as_bytes(), "/tmp/cow/f   54")];
        for (input, expected) in fixture {
            let result = nom_parser::single_quoted_absolute_path.parse(input);

            assert!(result.is_ok());
            let actual = result.unwrap();
            assert_eq!(actual.1.as_ref(), expected.as_bytes());
        }
    }

    #[test]
    fn empty_double_quoted() {
        let fixture = vec![("\"\"".as_bytes(), "".as_bytes())];
        for (input, expected) in fixture {
            let result = nom_parser::empty_double_quoted(input);

            assert!(result.is_ok());
            let actual = result.unwrap();
            assert_eq!(actual.1.as_ref(), expected);
        }
    }

    #[test]
    fn double_quoted_with() {
        let fixture = vec![
            (
                "\"a b\"".as_bytes(),
                vec!["a".as_bytes(), " ".as_bytes(), "b".as_bytes()],
            ),
            ("\"world's\"".as_bytes(), vec!["world's".as_bytes()]),
        ];
        for (input, expected) in fixture {
            let result = nom_parser::double_quoted_with(" '".as_bytes()).parse(input);

            assert!(result.is_ok());
            let actual = result.unwrap();
            assert_eq!(actual.1.as_ref(), &expected.concat());
        }
    }

    #[test]
    fn double_quoted_absolute_path() {
        let fixture = vec![
            ("\"/tmp/rat/f 84\"".as_bytes(), "/tmp/rat/f 84"),
            ("\"/tmp/rat/f   62\"".as_bytes(), "/tmp/rat/f   62"),
            ("\"/tmp/rat/f's50\"".as_bytes(), "/tmp/rat/f's50"),
        ];
        for (input, expected) in fixture {
            let result = nom_parser::double_quoted_absolute_path.parse(input);

            assert!(result.is_ok());
            let actual = result.unwrap();
            assert_eq!(actual.1.as_ref(), expected.as_bytes());
        }
    }

    #[test]
    fn alphanumeric_with_special_chars() {
        let fixture = vec![("'a'", "'", "'a'")];
        for (input, special_chars, expected) in fixture {
            let result = nom_parser::alphanumeric_with_special_chars(special_chars.as_bytes())
                .parse(input.as_bytes());

            assert_eq!(result, Ok(("".as_bytes(), Box::from(expected.as_bytes()))))
        }
    }

    #[test]
    fn absolute_path() {
        let fixture = vec![
            ("/tmp/fox", "/tmp/fox"),
            ("/tmp/ant/\\_ignored_6", "/tmp/ant/_ignored_6"),
            ("/tmp/ant/just_one_\\\\_51", "/tmp/ant/just_one_\\_51"),
            ("/tmp/ant/ignore_\\34", "/tmp/ant/ignore_34"),
            ("/tmp/apple/ping.md", "/tmp/apple/ping.md"),
        ];

        for (input, expected) in fixture {
            let r = nom_parser::absolute_path(" _.".as_bytes()).parse(input.as_bytes());
            assert!(r.is_ok());
            let actual = r.unwrap();
            println!(
                "actual {}",
                String::from_utf8(actual.clone().1.to_vec()).unwrap()
            );
            println!("expected {}", expected);
            assert_eq!(actual.1.as_ref(), expected.as_bytes());
        }
    }

    #[test]
    fn relative_path() {
        let fixture = vec![
            ("./blueberry/apple", "./blueberry/apple"),
            ("../../../", "../../.."),
        ];

        for (input, expected) in fixture {
            let r = nom_parser::relative_path(input.as_bytes());
            assert!(r.is_ok());
            let actual = r.unwrap();
            assert_eq!(actual.1.as_ref(), expected.as_bytes())
        }
    }

    #[test]
    fn home() {
        let fixture = vec![("~", "~")];

        for (input, expected) in fixture {
            let r = nom_parser::home(input.as_bytes());
            assert!(r.is_ok());
            let actual = r.unwrap();
            assert_eq!(actual.1.as_ref(), expected.as_bytes());
        }
    }

    #[test]
    fn path_component() {
        let fixture = vec![
            ("/_ignored_\\\\_2".as_bytes(), "/_ignored_\\_2".as_bytes()),
            ("/'no slash 49'".as_bytes(), "/no slash 49".as_bytes()),
            ("/'one slash \\3'".as_bytes(), "/one slash \\3".as_bytes()),
            ("/'one slash \\3'".as_bytes(), "/one slash \\3".as_bytes()),
            ("/ignored.md".as_bytes(), "/ignored.md".as_bytes()),
        ];
        for (input, expected) in fixture {
            let result = nom_parser::path_component(" _.".as_bytes()).parse(input);

            let actual = result.unwrap();
            println!(
                "actual {}",
                String::from_utf8(actual.clone().1.to_vec()).unwrap()
            );
            println!("expected {}", String::from_utf8(expected.to_vec()).unwrap());
            assert_eq!(actual.1.as_ref(), expected);
        }
    }

    #[test]
    fn arguments() {
        let fixture = vec![
            ("a", vec!["a".as_bytes()]),
            ("a_1", vec!["a_1".as_bytes()]),
            ("a b", vec!["a".as_bytes(), " ".as_bytes(), "b".as_bytes()]),
            (
                "a b c",
                vec![
                    "a".as_bytes(),
                    " ".as_bytes(),
                    "b".as_bytes(),
                    " ".as_bytes(),
                    "c".as_bytes(),
                ],
            ),
            ("/tmp/some_path", vec!["/tmp/some_path".as_bytes()]),
            ("/tmp/some-path", vec!["/tmp/some-path".as_bytes()]),
            ("./tmp", vec!["./tmp".as_bytes()]),
            ("~", vec!["~".as_bytes()]),
            ("'a b'", vec!["a b".as_bytes()]),
            (
                "'a b' 'c'",
                vec!["a b".as_bytes(), " ".as_bytes(), "c".as_bytes()],
            ),
            (
                "'a b' 'c''' dkfc",
                vec![
                    "a b".as_bytes(),
                    " ".as_bytes(),
                    "c".as_bytes(),
                    "".as_bytes(),
                    " ".as_bytes(),
                    "dkfc".as_bytes(),
                ],
            ),
            (
                "/tmp1 /tmp2",
                vec!["/tmp1".as_bytes(), " ".as_bytes(), "/tmp2".as_bytes()],
            ),
            (
                "/tmp/dog/\\_ignored_80 /tmp/dog/ignore_\\73 /tmp/dog/just_one_\\\\_93",
                vec![
                    "/tmp/dog/_ignored_80".as_bytes(),
                    " ".as_bytes(),
                    "/tmp/dog/ignore_73".as_bytes(),
                    " ".as_bytes(),
                    "/tmp/dog/just_one_\\_93".as_bytes(),
                ],
            ),
            (
                "/tmp/some-path/file.md",
                vec!["/tmp/some-path/file.md".as_bytes()],
            ),
        ];

        for (input, expected) in fixture {
            let r = nom_parser::arguments(input.as_bytes());
            let actual = r.unwrap();
            assert_eq!(&actual.1.as_ref(), expected.first().unwrap());
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
            assert_eq!(r.1.as_ref(), expected.as_bytes());
        }
    }

    #[test]
    fn redirection_argument() {
        let fixture = vec![
            ("> /redirection_path", "/redirection_path"),
            ("1> /redirection_path", "/redirection_path"),
        ];

        for (input, expected) in fixture {
            let r = nom_parser::redirection_argument.parse(input.as_bytes());

            assert!(r.is_ok());

            let r = r.unwrap();
            assert!(r.1.path.is_some());
            assert_eq!(r.1.path.unwrap().as_ref(), expected.as_bytes());
        }
    }

    #[test]
    fn parse() {
        let fixture: Vec<(&_, (&_, Vec<_>, Option<&[u8]>, Option<&[u8]>))> = vec![
            ("cmd", ("cmd", vec![], None, None)),
            (" cmd", ("cmd", vec![], None, None)),
            (" cmd ", ("cmd", vec![], None, None)),
            ("cmd a", ("cmd", vec!["a".as_bytes()], None, None)),
            ("cmd a_1", ("cmd", vec!["a_1".as_bytes()], None, None)),
            (
                "cmd a b",
                (
                    "cmd",
                    vec!["a".as_bytes(), " ".as_bytes(), "b".as_bytes()],
                    None,
                    None,
                ),
            ),
            (
                "cmd -1 abc",
                (
                    "cmd",
                    vec!["-1".as_bytes(), " ".as_bytes(), "abc".as_bytes()],
                    None,
                    None,
                ),
            ),
            (
                "echo H > abc",
                (
                    "echo",
                    vec!["H".as_bytes(), " ".as_bytes()],
                    Some("abc".as_bytes()),
                    None,
                ),
            ),
            (
                "echo H 1> abc",
                (
                    "echo",
                    vec!["H".as_bytes(), " ".as_bytes()],
                    Some("abc".as_bytes()),
                    None,
                ),
            ),
            (
                "echo H 2> abc",
                (
                    "echo",
                    vec!["H".as_bytes(), " ".as_bytes()],
                    None,
                    Some("abc".as_bytes()),
                ),
            ),
        ];
        for (
            input,
            (
                expected_cmd,
                expected_args,
                expected_redirection_path,
                expected_error_redirection_path,
            ),
        ) in fixture
        {
            let (_input, command) = nom_parser::parse(input.as_bytes()).unwrap();
            assert_eq!(command.name, expected_cmd.as_bytes());
            assert_eq!(
                command.args.iter().map(|f| f.to_vec()).collect::<Vec<_>>(),
                expected_args.as_slice()
            );
            if expected_redirection_path.is_some() {
                assert!(command.output_redirection.path.is_some());
                assert_eq!(
                    command.output_redirection.path.unwrap(),
                    expected_redirection_path.unwrap().into()
                );
            }
            if expected_error_redirection_path.is_some() {
                assert!(command.error_redirection_path.is_some());
                assert_eq!(
                    command.error_redirection_path.unwrap(),
                    expected_error_redirection_path.unwrap().into()
                );
            }
        }
    }
}
