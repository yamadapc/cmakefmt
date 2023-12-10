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
use std::cmp::min;

use nom::branch::alt;
use nom::bytes::complete::{tag, take_till1};
use nom::character::complete::{char, multispace0, multispace1, space0};
use nom::error::ErrorKind;
use nom::multi::{many1, separated_list0};
use nom::sequence::{delimited, tuple};
use nom::{IResult, InputIter, InputTakeAtPosition};
use nom::{InputTake, Parser};

use crate::parser::types::{CMakeCommand, CMakeDocument, CMakeStatement, CMakeValue};

mod strings;
pub mod types;

fn cmake_comment(input: &str) -> IResult<&str, &str> {
    let (input, _) = char('#')(input)?;
    let (input, contents) = take_till1(|item| item == '\n')(input)?;
    Ok((input, contents))
}

fn cmake_command_name(input: &str) -> IResult<&str, &str> {
    input.split_at_position1_complete(
        |item| !item.is_alphanumeric() && item != '_',
        ErrorKind::AlphaNumeric,
    )
}

fn cmake_quoted_string_literal(input: &str) -> IResult<&str, CMakeValue> {
    let (input, string) = strings::parse_string(input)?;
    Ok((input, CMakeValue::QuotedString(string)))
}

fn cmake_string_literal(input: &str) -> IResult<&str, CMakeValue> {
    let (input, result) = take_till1(|item: char| item.is_whitespace() || item == ')')(input)?;
    if result
        .iter_elements()
        .all(|c| c.is_uppercase() || c == '_' || c.is_numeric())
    {
        return Ok((input, CMakeValue::ArgumentSpecifier(result.to_string())));
    }
    Ok((input, CMakeValue::StringLiteral(result.to_string())))
}

fn cmake_value(input: &str) -> IResult<&str, CMakeValue> {
    eprintln!("parse value at {:?}", input.take(min(input.len(), 10)));
    let (input, result) = alt((
        cmake_comment.map(|item| CMakeValue::Comment(item.to_string())),
        cmake_quoted_string_literal,
        cmake_string_literal,
    ))(input)?;
    eprintln!(">> {:?}", result);
    Ok((input, result))
}

fn cmake_command(input: &str) -> IResult<&str, CMakeCommand> {
    eprintln!("parse cmd at {:?}", input.take(min(input.len(), 10)));
    let (input, name) = cmake_command_name(input)?;
    eprintln!("parse args at {:?}", input.take(min(input.len(), 10)));
    let (input, args) = delimited(
        char('('),
        delimited(
            multispace0,
            separated_list0(multispace1, cmake_value),
            multispace0,
        ),
        char(')'),
    )(input)?;
    Ok((
        input,
        CMakeCommand {
            name: name.to_string(),
            args,
        },
    ))
}

fn cmake_statement(input: &str) -> IResult<&str, CMakeStatement> {
    alt((
        cmake_command.map(CMakeStatement::Command),
        cmake_comment.map(|item| CMakeStatement::Comment(item.to_string())),
        tuple((tag("\n"), space0)).map(|_| CMakeStatement::Newline),
    ))(input)
}

pub fn cmake_parser(input: &str) -> IResult<&str, CMakeDocument> {
    let (input, statements) = many1(cmake_statement)(input)?;
    Ok((input, CMakeDocument { statements }))
}

#[cfg(test)]
mod test {
    use nom::combinator::all_consuming;

    use super::*;

    #[test]
    fn test_parse_string_unquoted() {
        let (_, result) = all_consuming(cmake_value)("foo").unwrap();
        assert_eq!(result, CMakeValue::StringLiteral("foo".to_string()));
    }

    #[test]
    fn test_parse_string_quoted() {
        let (_, result) = all_consuming(cmake_value)("\"foo\"").unwrap();
        assert_eq!(result, CMakeValue::QuotedString("foo".to_string()));
    }

    #[test]
    fn test_parse_number_literal() {
        let (_, result) = all_consuming(cmake_value)("1.2").unwrap();
        assert_eq!(result, CMakeValue::StringLiteral("1.2".to_string()));
    }

    #[test]
    fn test_parse_version_literal() {
        let (_, result) = all_consuming(cmake_value)("1.2.3").unwrap();
        assert_eq!(result, CMakeValue::StringLiteral("1.2.3".to_string()));
    }

    #[test]
    fn test_parse_command() {
        let (_, result) = all_consuming(cmake_command)("foo(bar)").unwrap();
        assert_eq!(
            result,
            CMakeCommand {
                name: "foo".to_string(),
                args: vec![CMakeValue::StringLiteral("bar".to_string())],
            }
        );
    }

    #[test]
    fn test_parse_command_with_two_args() {
        let (_, result) = all_consuming(cmake_command)("foo(bar foo)").unwrap();
        assert_eq!(
            result,
            CMakeCommand {
                name: "foo".to_string(),
                args: vec![
                    CMakeValue::StringLiteral("bar".to_string()),
                    CMakeValue::StringLiteral("foo".to_string()),
                ],
            }
        );
    }

    #[test]
    fn test_whitespace_gets_parsed() {
        let (_, result) = all_consuming(cmake_parser)("foo()\n\nbar()").unwrap();
        assert_eq!(
            result,
            CMakeDocument {
                statements: vec![
                    CMakeStatement::Command(CMakeCommand {
                        name: "foo".to_string(),
                        args: vec![],
                    }),
                    CMakeStatement::Newline,
                    CMakeStatement::Newline,
                    CMakeStatement::Command(CMakeCommand {
                        name: "bar".to_string(),
                        args: vec![],
                    })
                ]
            }
        );
    }

    #[test]
    fn test_command_whitespace_is_parsed() {
        let (_, result) = all_consuming(cmake_parser)(
            "
project(
  pyramid_envelope
  VERSION 0.0.1
  LANGUAGES CXX
)"
            .trim(),
        )
        .unwrap();
        assert_eq!(
            result,
            CMakeDocument {
                statements: vec![CMakeStatement::Command(CMakeCommand {
                    name: "project".to_string(),
                    args: vec![
                        CMakeValue::StringLiteral("pyramid_envelope".to_string()),
                        CMakeValue::ArgumentSpecifier("VERSION".to_string()),
                        CMakeValue::StringLiteral("0.0.1".to_string()),
                        CMakeValue::ArgumentSpecifier("LANGUAGES".to_string()),
                        CMakeValue::ArgumentSpecifier("CXX".to_string()),
                    ],
                }),]
            }
        );
    }
}
