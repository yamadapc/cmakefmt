// The MIT License (MIT)
//
// Copyright (c) 2023 Pedro Tacla Yamada
//
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in
// all copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN
// THE SOFTWARE.

use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::character::complete::{char as parse_char, none_of};
use nom::combinator::map;
use nom::multi::many0;
use nom::sequence::{delimited, preceded};

use crate::parser::IResult;

enum StringPart {
    Char(char),
    EscapedSlash,
    EscapedQuote(char),
}

pub fn parse_string(input: &str) -> IResult<&str, String> {
    let parse_quote = map(
        preceded(parse_char('\\'), parse_char('"')),
        StringPart::EscapedQuote,
    );
    let parse_escaped_slash = map(tag("\\\\"), |_| StringPart::EscapedSlash);
    let parse_string_char = map(none_of("\""), StringPart::Char);
    map(
        delimited(
            parse_char('"'),
            many0(alt((parse_escaped_slash, parse_quote, parse_string_char))),
            parse_char('"'),
        ),
        |parts| {
            parts
                .iter()
                .flat_map(|p| match p {
                    StringPart::Char(c) => format!("{}", c).chars().collect::<Vec<char>>(),
                    StringPart::EscapedQuote(_) => "\\\"".chars().collect(),
                    StringPart::EscapedSlash => "\\\\".chars().collect(),
                })
                .collect::<String>()
        },
    )(input)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_parse_string() {
        let input = "\"test 1234\"";
        let (_, result) = parse_string(input).unwrap();
        assert_eq!(result, "test 1234");
    }

    #[test]
    fn test_parse_string_with_quotes() {
        let input = "\"test \\\"quote\\\" 1234\"";
        let (_, result) = parse_string(input).unwrap();
        assert_eq!(result, "test \\\"quote\\\" 1234");
    }
}
