use nom::bytes::complete::take_till;
use nom::combinator::{map, opt};
use nom::error::{context, ParseError};
use nom::multi::many0;
use nom::{
    branch::alt,
    bytes::complete::{tag, take_till1},
    character::complete::{char, multispace0, multispace1, space0},
    error,
    error::ErrorKind,
    multi::{many1, separated_list0},
    sequence::{delimited, tuple},
    InputIter, InputTakeAtPosition, Parser,
};

use crate::parser::types::{
    CMakeCommand, CMakeDocument, CMakeStatement, CMakeValue, CmakeIfBase, CmakeIfStatement,
};

mod strings;
pub mod types;

pub type IResult<I, O> = Result<(I, O), nom::Err<error::VerboseError<I>>>;

fn cmake_comment(input: &str) -> IResult<&str, &str> {
    let (input, _) = char('#')(input)?;
    let (input, contents) = take_till(|item| item == '\n')(input)?;
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

fn cmake_variable(input: &str) -> IResult<&str, CMakeValue> {
    let (input, name) =
        delimited(tag("${"), take_till1(|item: char| item == '}'), char('}'))(input)?;
    Ok((input, CMakeValue::Variable(name.to_string())))
}

fn cmake_value(input: &str) -> IResult<&str, CMakeValue> {
    let (input, result) = context(
        "Value",
        alt((
            cmake_comment.map(|item| CMakeValue::Comment(item.to_string())),
            cmake_quoted_string_literal,
            cmake_variable,
            cmake_string_literal,
        )),
    )(input)?;
    Ok((input, result))
}

fn cmake_command(input: &str) -> IResult<&str, CMakeCommand> {
    let (input, name) = cmake_command_name(input)?;
    if name == "elseif" || name == "endif" || name == "if" {
        return Err(nom::Err::Error(error::VerboseError::from_error_kind(
            input,
            ErrorKind::AlphaNumeric,
        )));
    }
    let (input, _) = space0(input)?;
    let (input, args) = cmake_args(input)?;
    Ok((
        input,
        CMakeCommand {
            name: name.to_string(),
            args,
        },
    ))
}

fn cmake_args(input: &str) -> IResult<&str, Vec<CMakeValue>> {
    delimited(
        char('('),
        delimited(multispace0, cmake_arg_list_inner, multispace0),
        char(')'),
    )(input)
}

fn cmake_arg_parenthesis(input: &str) -> IResult<&str, Vec<CMakeValue>> {
    let (input, (start, inner, end)) = tuple((
        char('(').map(|_| CMakeValue::Parenthesis("(".to_string())),
        separated_list0(multispace1, cmake_value),
        char(')').map(|_| CMakeValue::Parenthesis(")".to_string())),
    ))(input)?;

    let mut result = vec![start];
    result.extend(inner);
    result.push(end);
    Ok((input, result))
}

fn cmake_arg_list_inner(input: &str) -> IResult<&str, Vec<CMakeValue>> {
    let value_parser = map(cmake_value, |item| vec![item]);
    map(
        alt((separated_list0(
            multispace1,
            alt((cmake_arg_parenthesis, value_parser)),
        ),)),
        |output| output.into_iter().flatten().collect(),
    )(input)
}

fn cmake_else_if_block(input: &str) -> IResult<&str, CmakeIfBase> {
    let (input, _) = tag("elseif")(input)?;
    let (input, _) = space0(input)?;
    let (input, condition) = cmake_args(input)?;
    let (input, body) = many0(delimited(space0, cmake_statement, space0))(input)?;

    Ok((input, CmakeIfBase { condition, body }))
}

fn cmake_if_block(input: &str) -> IResult<&str, CMakeStatement> {
    let (input, _) = tag("if")(input)?;
    let (input, _) = space0(input)?;
    let (input, condition) = cmake_args(input)?;

    let (input, body) = many0(delimited(space0, cmake_statement, space0))(input)?;
    let (input, else_ifs) = many0(delimited(space0, cmake_else_if_block, space0))(input)?;
    let (input, else_body) = opt(delimited(
        space0,
        tuple((
            tag("else()"),
            many0(delimited(space0, cmake_statement, space0)),
        )),
        space0,
    ))(input)?;
    let (input, _) = tag("endif()")(input)?;

    Ok((
        input,
        CMakeStatement::If(CmakeIfStatement {
            base: CmakeIfBase { condition, body },
            else_ifs,
            else_body: else_body.map(|(_, body)| body),
        }),
    ))
}

fn cmake_statement(input: &str) -> IResult<&str, CMakeStatement> {
    alt((
        cmake_if_block,
        cmake_command.map(CMakeStatement::Command),
        cmake_comment.map(|item| CMakeStatement::Comment(item.to_string())),
        tuple((tag("\n"), space0)).map(|_| CMakeStatement::Newline),
    ))(input)
}

pub fn cmake_parser(input: &str) -> IResult<&str, CMakeDocument> {
    let (input, statements) = many1(delimited(space0, cmake_statement, space0))(input)?;
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
    fn test_parse_variable() {
        let (_, result) = all_consuming(cmake_value)("${VARIABLE}").unwrap();
        assert_eq!(result, CMakeValue::Variable("VARIABLE".to_string()));
    }

    #[test]
    fn test_parse_complex_expr() {
        let input = r#"
((NOT MSVC) OR (${CMAKE_CXX_COMPILER_ID} MATCHES "Clang"))
        "#
        .trim();
        let (_, result) = all_consuming(cmake_args)(input).unwrap();
        assert_eq!(
            result,
            vec![
                CMakeValue::Parenthesis(String::from("(")),
                CMakeValue::ArgumentSpecifier(String::from("NOT")),
                CMakeValue::ArgumentSpecifier(String::from("MSVC")),
                CMakeValue::Parenthesis(String::from(")")),
                CMakeValue::ArgumentSpecifier(String::from("OR")),
                CMakeValue::Parenthesis(String::from("(")),
                CMakeValue::Variable(String::from("CMAKE_CXX_COMPILER_ID")),
                CMakeValue::ArgumentSpecifier(String::from("MATCHES")),
                CMakeValue::QuotedString(String::from("Clang")),
                CMakeValue::Parenthesis(String::from(")"))
            ]
        );
    }

    #[test]
    fn test_parse_arg_list_inner() {
        let input = r#"
(NOT MSVC) OR
        "#
        .trim();
        let (_, result) = all_consuming(cmake_arg_list_inner)(input).unwrap();
        assert_eq!(
            result,
            vec![
                CMakeValue::Parenthesis(String::from("(")),
                CMakeValue::ArgumentSpecifier(String::from("NOT")),
                CMakeValue::ArgumentSpecifier(String::from("MSVC")),
                CMakeValue::Parenthesis(String::from(")")),
                CMakeValue::ArgumentSpecifier(String::from("OR"))
            ]
        );
    }

    #[test]
    fn test_parse_cmake_arg_parenthesis() {
        let input = r#"
(NOT MSVC)
        "#
        .trim();
        let (_, result) = all_consuming(cmake_arg_parenthesis)(input).unwrap();
        assert_eq!(
            result,
            vec![
                CMakeValue::Parenthesis("(".to_string()),
                CMakeValue::ArgumentSpecifier("NOT".to_string()),
                CMakeValue::ArgumentSpecifier("MSVC".to_string()),
                CMakeValue::Parenthesis(")".to_string()),
            ]
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

    #[test]
    fn test_if_statements() {
        let input = r#"
if(CMAKE_COMPILER_IS_GNUCXX)
  foo()
elseif(MSVC)
  bar()                      
endif()
        "#
        .trim();

        let (_, result) = all_consuming(cmake_parser)(input).unwrap();
        assert_eq!(
            result,
            CMakeDocument {
                statements: vec![CMakeStatement::If(CmakeIfStatement {
                    base: CmakeIfBase {
                        condition: vec![CMakeValue::ArgumentSpecifier(String::from(
                            "CMAKE_COMPILER_IS_GNUCXX"
                        ))],
                        body: vec![
                            CMakeStatement::Newline,
                            CMakeStatement::Command(CMakeCommand {
                                name: String::from("foo"),
                                args: vec![]
                            }),
                            CMakeStatement::Newline,
                        ],
                    },
                    else_ifs: vec![CmakeIfBase {
                        condition: vec![CMakeValue::ArgumentSpecifier(String::from("MSVC"))],
                        body: vec![
                            CMakeStatement::Newline,
                            CMakeStatement::Command(CMakeCommand {
                                name: String::from("bar"),
                                args: vec![]
                            }),
                            CMakeStatement::Newline,
                        ],
                    }],
                    else_body: None,
                }),]
            }
        )
    }
}
