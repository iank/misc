use nom::{
    branch::alt,
    bytes::complete::{escaped_transform, is_not, tag},
    character::complete::{char, space0, space1},
    combinator::{all_consuming, map_parser, recognize, value},
    multi::separated_list0,
    sequence::tuple,
    IResult,
};

use super::Mount;

fn not_whitespace(i: &str) -> IResult<&str, &str> {
    is_not(" \t")(i)
}

fn escaped_space(i: &str) -> IResult<&str, &str> {
    value(" ", tag("040"))(i)
}

fn escaped_backslash1(i: &str) -> IResult<&str, &str> {
    recognize(char('\\'))(i)
}

fn escaped_backslash2(i: &str) -> IResult<&str, &str> {
    value("\\", tag("134"))(i)
}

// WSL can do a raw \ in the mount options, apparently. Interpret \; or \, as themselves
fn escaped_semicolon(i: &str) -> IResult<&str, &str> {
    value("\\;", tag(";"))(i)
}
fn escaped_comma(i: &str) -> IResult<&str, &str> {
    value("\\,", tag(","))(i)
}

fn transform_escaped(i: &str) -> IResult<&str, String> {
    escaped_transform(
        is_not("\\"),
        '\\',
        alt((
            escaped_backslash1,
            escaped_backslash2,
            escaped_space,
            escaped_semicolon,
            escaped_comma,
        )),
    )(i)
}

fn mount_opts(i: &str) -> IResult<&str, Vec<String>> {
    separated_list0(char(','), map_parser(is_not(", \t"), transform_escaped))(i)
}

pub fn parse_line(i: &str) -> IResult<&str, Mount> {
    let (i, device) = map_parser(not_whitespace, transform_escaped)(i)?;
    let (i, _) = space1(i)?;
    let (i, mount_point) = map_parser(not_whitespace, transform_escaped)(i)?;
    let (i, _) = space1(i)?;
    let (i, filesystem) = not_whitespace(i)?;
    let (i, _) = space1(i)?;
    let (i, options) = mount_opts(i)?;
    let (i, _) =
        all_consuming(tuple((space1, char('0'), space1, char('0'), space0)))(
            i,
        )?;
    Ok((
        i,
        Mount {
            device: device,
            mount_point: mount_point,
            filesystem: filesystem.to_string(),
            options: options,
        },
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_not_whitespace() {
        assert_eq!(not_whitespace("abcd efg"), Ok((" efg", "abcd")));
        assert!(matches!(
            not_whitespace(" abcdefg"),
            Err(nom::Err::Error(_))
        ));
    }

    #[test]
    fn test_escaped_space() {
        assert_eq!(escaped_space("040"), Ok(("", " ")));
        assert!(matches!(escaped_space(" "), Err(nom::Err::Error(_))));
    }

    #[test]
    fn test_escaped_backslash1() {
        assert_eq!(escaped_backslash1("\\"), Ok(("", "\\")));
        assert!(matches!(
            escaped_backslash1("not a backslash"),
            Err(nom::Err::Error(_))
        ));
    }

    #[test]
    fn test_transform_escaped() {
        assert_eq!(
            transform_escaped("abc\\040def\\\\g\\040h"),
            Ok(("", String::from("abc def\\g h")))
        );
        assert!(matches!(
            transform_escaped("\\bad"),
            Err(nom::Err::Error(_))
        ));
    }

    #[test]
    fn test_mount_opts() {
        assert_eq!(
            mount_opts("a,bc,d\\040e"),
            Ok((
                "",
                vec!["a".to_string(), "bc".to_string(), "d e".to_string()]
            ))
        );
    }

    #[test]
    fn test_parse_line() {
        let mount1 = Mount {
            device: "C:\\".to_string(),
            mount_point: "mount_point".to_string(),
            filesystem: "filesystem".to_string(),
            options: vec![
                "options".to_string(),
                "a".to_string(),
                "b=c".to_string(),
                "d e".to_string(),
                "f=abc;path=C:\\;uid=1000".to_string(),
            ],
        };
        let (_, mount2) = parse_line(
            "C:\\134 mount_point filesystem options,a,b=c,d\\040e,f=abc;path=C:\\;uid=1000 0 0",
        )
        .unwrap();
        assert_eq!(mount1.device, mount2.device);
        assert_eq!(mount1.mount_point, mount2.mount_point);
        assert_eq!(mount1.filesystem, mount2.filesystem);
        assert_eq!(mount1.options, mount2.options);
    }
}
