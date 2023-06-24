/// Not working yet. Still some issues to Iron out.
use nom::{
    branch::alt,
    bytes::complete::{escaped, is_not, tag},
    character::complete::{char, digit1, multispace0},
    combinator::{map, map_res, recognize},
    multi::separated_list0,
    sequence::{delimited, preceded, tuple},
    IResult, Parser,
};

#[derive(Debug, PartialEq)]
pub enum JsonValue {
    Object(Vec<(String, JsonValue)>),
    Array(Vec<JsonValue>),
    String(String),
    Number(f64),
    Boolean(bool),
    Null,
}

pub fn parse_string(input: &str) -> IResult<&str, String> {
    let (input, string) = delimited(
        char('"'),
        escaped(is_not("\\\""), '\\', char('"')),
        char('"'),
    )(input)?;
    Ok((input, string.to_owned()))
}

pub fn parse_number(input: &str) -> IResult<&str, f64> {
    let integer_parser = map_res(digit1, |s: &str| s.parse::<f64>());
    let integer_parser_2 = map_res(digit1, |s: &str| s.parse::<f64>());

    let fractional_parser = map_res(digit1, |s: &str| s.parse::<f64>())
        .map(|fractional| fractional / 10f64.powi(fractional.to_string().len() as i32));

    let mut number_parser = alt((
        recognize(tuple((integer_parser, char('.'), fractional_parser))),
        recognize(integer_parser_2),
    ));

    number_parser(input).map(|(remaining, number)| (remaining, number.parse().unwrap()))
}

pub fn parse_boolean(input: &str) -> IResult<&str, bool> {
    alt((map(tag("true"), |_| true), map(tag("false"), |_| false)))(input)
}

pub fn parse_null(input: &str) -> IResult<&str, ()> {
    map(tag("null"), |_| ())(input)
}

pub fn parse_value(input: &str) -> IResult<&str, JsonValue> {
    preceded(
        multispace0,
        alt((
            parse_object,
            parse_array,
            map(parse_string, JsonValue::String),
            map(parse_number, JsonValue::Number),
            map(parse_boolean, JsonValue::Boolean),
            map(parse_null, |_| JsonValue::Null),
        )),
    )(input)
}

pub fn parse_object(input: &str) -> IResult<&str, JsonValue> {
    let parse_opening_brace = preceded(multispace0, char('{'));
    let parse_closing_brace = preceded(multispace0, char('}'));
    let parse_comma = preceded(multispace0, char(','));
    // let parse_quoted_string = preceded(multispace0, parse_string);

    let parser = map(separated_list0(parse_comma, parse_key_value), |pairs| {
        JsonValue::Object(pairs)
    });

    delimited(parse_opening_brace, parser, parse_closing_brace)(input)
}

pub fn parse_key_value(input: &str) -> IResult<&str, (String, JsonValue)> {
    let parse_key = parse_string;
    let parse_separator = preceded(multispace0, char(':'));
    let parse_value = parse_value;

    let mut parser = tuple((parse_key, parse_separator, parse_value));

    parser(input).map(|(rest, (key, _, value))| (rest, (key, value)))
}

pub fn parse_array(input: &str) -> IResult<&str, JsonValue> {
    let parse_array = delimited(
        preceded(multispace0, char('[')),
        separated_list0(preceded(multispace0, char(',')), parse_value),
        preceded(multispace0, char(']')),
    );
    map(parse_array, JsonValue::Array)(input)
}

pub fn parse_json(input: &str) -> IResult<&str, JsonValue> {
    preceded(multispace0, parse_value)(input)
}

#[cfg(test)]
mod tests {
    #[test]
    fn parse_string_test() {
        assert_eq!(
            super::parse_string(r#""Hello, World!""#),
            Ok(("", "Hello, World!".to_owned()))
        );
    }

    #[test]
    fn parse_decimal_number_test() {
        assert_eq!(super::parse_number("123.456"), Ok(("", 123.456)));
    }

    #[test]
    fn parse_integer_number_test() {
        assert_eq!(super::parse_number("123"), Ok(("", 123.0)));
    }

    #[test]
    fn parse_boolean_test() {
        assert_eq!(super::parse_boolean("true"), Ok(("", true)));
        assert_eq!(super::parse_boolean("false"), Ok(("", false)));
    }

    #[test]
    fn parse_null_test() {
        assert_eq!(super::parse_null("null"), Ok(("", ())));
    }

    #[test]
    fn parse_array_test() {
        assert_eq!(
            super::parse_array("[1, 2, 3]"),
            Ok((
                "",
                super::JsonValue::Array(vec![
                    super::JsonValue::Number(1.0),
                    super::JsonValue::Number(2.0),
                    super::JsonValue::Number(3.0)
                ])
            ))
        );
    }

    #[test]
    fn parse_key_value_test() {
        assert_eq!(
            super::parse_key_value(r#""a": 1"#),
            Ok(("", ("a".to_owned(), super::JsonValue::Number(1.0))))
        );
    }

    #[test]
    fn parse_object_test() {
        let result = super::parse_object(r#"{"a": "1", "b": "2"}"#);
        // let result = super::parse_object(r#"[1,2,3]"#);

        println!("{:?}", result);
        assert_eq!(
            result,
            Ok((
                "",
                super::JsonValue::Object(vec![
                    ("a".to_owned(), super::JsonValue::Number(1.0)),
                    ("b".to_owned(), super::JsonValue::Number(2.0))
                ])
            ))
        );
    }

    #[test]
    fn parse_value_test() {
        assert_eq!(
            super::parse_value(" 123.456 "),
            Ok((" ", super::JsonValue::Number(123.456)))
        );
        assert_eq!(
            super::parse_value("123"),
            Ok(("", super::JsonValue::Number(123.0)))
        );
        assert_eq!(
            super::parse_value(" true "),
            Ok((" ", super::JsonValue::Boolean(true)))
        );
        assert_eq!(
            super::parse_value(" false "),
            Ok((" ", super::JsonValue::Boolean(false)))
        );
        assert_eq!(
            super::parse_value(" null "),
            Ok((" ", super::JsonValue::Null))
        );
        assert_eq!(
            super::parse_value(" \"Hello, World!\" "),
            Ok((" ", super::JsonValue::String("Hello, World!".to_owned())))
        );
        assert_eq!(
            super::parse_value(" [1, 2, 3] "),
            Ok((
                " ",
                super::JsonValue::Array(vec![
                    super::JsonValue::Number(1.0),
                    super::JsonValue::Number(2.0),
                    super::JsonValue::Number(3.0)
                ])
            ))
        );
        assert_eq!(
            super::parse_value(" {\"foo\": \"bar\"} "),
            Ok((
                " ",
                super::JsonValue::Object(vec![(
                    "foo".to_owned(),
                    super::JsonValue::String("bar".to_owned())
                )])
            ))
        );
    }
}

// #[test]
// fn parse_json_test() {
//     let input = r#"{
//         "name": "John Doe",
//         "age": 42,
//         "isStudent": true,
//         "grades": [90, 85, 95],
//         "address": {
//             "street": "123 Main St",
//             "city": "Anytown",
//             "state": "CA",
//             "zip": "12345"
//         },
//         "phoneNumbers": [
//             {"type": "home", "number": "555-1234"},
//             {"type": "work", "number": "555-5678"}
//         ]
//     }"#;

//     assert_eq!(
//         parse_json(input),
//         Ok((
//             "",
//             JsonValue::Object(vec![
//                 ("name".to_owned(), JsonValue::String("John Doe".to_owned())),
//                 ("age".to_owned(), JsonValue::Number(42.0)),
//                 ("isStudent".to_owned(), JsonValue::Boolean(true)),
//                 (
//                     "grades".to_owned(),
//                     JsonValue::Array(vec![
//                         JsonValue::Number(90.0),
//                         JsonValue::Number(85.0),
//                         JsonValue::Number(95.0),
//                     ])
//                 ),
//                 (
//                     "address".to_owned(),
//                     JsonValue::Object(vec![
//                         (
//                             "street".to_owned(),
//                             JsonValue::String("123 Main St".to_owned())
//                         ),
//                         ("city".to_owned(), JsonValue::String("Anytown".to_owned())),
//                         ("state".to_owned(), JsonValue::String("CA".to_owned())),
//                         ("zip".to_owned(), JsonValue::String("12345".to_owned())),
//                     ])
//                 ),
//                 (
//                     "phoneNumbers".to_owned(),
//                     JsonValue::Array(vec![
//                         JsonValue::Object(vec![
//                             ("type".to_owned(), JsonValue::String("home".to_owned())),
//                             (
//                                 "number".to_owned(),
//                                 JsonValue::String("555-1234".to_owned())
//                             ),
//                         ]),
//                         JsonValue::Object(vec![
//                             ("type".to_owned(), JsonValue::String("work".to_owned())),
//                             (
//                                 "number".to_owned(),
//                                 JsonValue::String("555-5678".to_owned())
//                             ),
//                         ]),
//                     ])
//                 ),
//             ])
//         ))
//     );
// }
