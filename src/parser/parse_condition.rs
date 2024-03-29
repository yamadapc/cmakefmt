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
use nom::bytes::complete::tag_no_case;
use nom::character::complete::multispace0;
use nom::combinator::{map, opt};
use nom::error::ParseError;
use nom::error::{context, ErrorKind};
use nom::sequence::tuple;
use std::fmt::Debug;

use crate::parser::types::CMakeCondition;
use crate::parser::{cmake_comment, cmake_value, ErrorType, IResult};

fn cmake_condition_parentheses(input: &str) -> IResult<&str, CMakeCondition> {
    let base = tuple((
        nom::character::complete::char('('),
        multispace0,
        cmake_condition,
        multispace0,
        nom::character::complete::char(')'),
    ));
    let inner = map(base, |(_, _, condition, _, _)| {
        CMakeCondition::Parentheses {
            value: Box::new(condition),
        }
    });

    context("condition_parentheses", inner)(input)
}

fn cmake_condition_unary_test(input: &str) -> IResult<&str, CMakeCondition> {
    let operator = alt((
        tag_no_case("EXISTS"),
        tag_no_case("IS_ABSOLUTE"),
        tag_no_case("IS_DIRECTORY"),
        tag_no_case("IS_SYMLINK"),
        tag_no_case("COMMAND"),
        tag_no_case("DEFINED"),
        tag_no_case("POLICY"),
        tag_no_case("TEST"),
        tag_no_case("TARGET"),
    ));
    let base = tuple((
        operator,
        nom::character::complete::multispace1,
        cmake_condition,
    ));
    let inner = map(base, |(operator, _, value)| CMakeCondition::UnaryTest {
        operator: operator.to_string(),
        value: Box::new(value),
    });

    context("condition_unary_test", inner)(input)
}

fn cmake_condition_binary_test(input: &str) -> IResult<&str, CMakeCondition> {
    let operator = alt((
        tag_no_case("EQUAL"),
        tag_no_case("LESS_EQUAL"),
        tag_no_case("LESS"),
        tag_no_case("GREATER_EQUAL"),
        tag_no_case("GREATER"),
        tag_no_case("STREQUAL"),
        tag_no_case("STRLESS_EQUAL"),
        tag_no_case("STRLESS"),
        tag_no_case("STRGREATER_EQUAL"),
        tag_no_case("STRGREATER"),
        tag_no_case("VERSION_EQUAL"),
        tag_no_case("VERSION_LESS_EQUAL"),
        tag_no_case("VERSION_LESS"),
        tag_no_case("VERSION_GREATER_EQUAL"),
        tag_no_case("VERSION_GREATER"),
        tag_no_case("PATH_EQUAL"),
        tag_no_case("MATCHES"),
        tag_no_case("INLIST"),
        tag_no_case("IN_LIST"),
        tag_no_case("NOTINLIST"),
        tag_no_case("NOT_IN_LIST"),
    ));
    let base = tuple((
        cmake_condition_value,
        nom::character::complete::multispace1,
        operator,
        nom::character::complete::multispace1,
        cmake_condition_value,
    ));
    let inner = map(base, |(left, _, operator, _, right)| {
        CMakeCondition::BinaryTest {
            operator: operator.to_string(),
            left: Box::new(left),
            right: Box::new(right),
        }
    });

    context("condition_binary_test", inner)(input)
}

fn cmake_condition_value(input: &str) -> IResult<&str, CMakeCondition> {
    let inner = map(cmake_value, CMakeCondition::Value);
    context("condition_value", inner)(input)
}

fn cmake_condition_unary_logical_operator(input: &str) -> IResult<&str, CMakeCondition> {
    let base = tuple((
        tag_no_case("NOT"),
        nom::character::complete::multispace1,
        cmake_condition,
    ));
    let inner = map(base, |(operator, _, value)| {
        CMakeCondition::UnaryLogicalOperator {
            operator: operator.to_string(),
            value: Box::new(value),
        }
    });

    context("condition_unary_logical_operator", inner)(input)
}

fn cmake_condition_binary_logical_operator(input: &str) -> IResult<&str, CMakeCondition> {
    let parse_operator = alt((tag_no_case("AND"), tag_no_case("OR")));
    let parse_base = tuple((
        cmake_condition_inner(Some(cmake_condition_binary_logical_operator)),
        nom::character::complete::multispace1,
        parse_operator,
        nom::character::complete::multispace1,
        cmake_condition,
    ));
    let inner = map(parse_base, |(left, _, operator, _, right)| {
        CMakeCondition::BinaryLogicalOperator {
            operator: operator.to_string(),
            left: Box::new(left),
            right: Box::new(right),
        }
    });

    context("binary_logical_operator", inner)(input)
}

fn cmake_condition_comment(input: &str) -> IResult<&str, CMakeCondition> {
    let inner = map(
        tuple((cmake_comment, multispace0, opt(cmake_condition))),
        |(comment, _, tail)| CMakeCondition::Comment {
            content: comment.to_string(),
            tail: tail.map(Box::new),
        },
    );
    context("condition_comment", inner)(input)
}

fn cmake_condition_inner(
    ignore: Option<fn(&str) -> IResult<&str, CMakeCondition>>,
) -> impl Fn(&str) -> IResult<&str, CMakeCondition> {
    let mut options = vec![
        cmake_condition_binary_logical_operator,
        cmake_condition_binary_test,
        cmake_condition_unary_logical_operator,
        cmake_condition_unary_test,
        cmake_condition_parentheses,
        cmake_condition_comment,
        cmake_condition_value,
    ];
    if let Some(to_ignore) = ignore {
        options = options
            .into_iter()
            .filter(|x| x != &to_ignore)
            .collect::<Vec<_>>();
    }
    move |input| {
        let parser = alt_list(&options);
        parser(input)
    }
}

pub fn cmake_condition(input: &str) -> IResult<&str, CMakeCondition> {
    let mut parser = context("cmake_condition", cmake_condition_inner(None));
    parser(input)
}

fn alt_list<'a, T: Debug>(
    parsers: &'a [impl Fn(&str) -> IResult<&str, T>],
) -> impl Fn(&str) -> IResult<&str, T> + 'a {
    return move |input| -> IResult<&str, T> {
        let parser = &parsers[0];
        let result = parser(input);
        if let Ok(result) = result {
            return Ok(result);
        }
        let mut error = result.unwrap_err();
        for parser in parsers.iter().skip(1) {
            let result = parser(input);
            match result {
                Ok(_) => return result,
                Err(nom::Err::Error(err)) => error = error.map(|e| e.or(err)),
                Err(nom::Err::Incomplete(_)) => {
                    error = error.map(|e| e.or(ErrorType::from_error_kind(input, ErrorKind::Eof)));
                }
                Err(nom::Err::Failure(err)) => {
                    error = error.map(|e| e.or(err));
                }
            }
        }
        Err(error)
    };
}

#[cfg(test)]
mod test {
    use nom::combinator::all_consuming;

    use crate::parser::types::CMakeValue;

    use super::*;

    #[test]
    fn test_parse_condition_value() {
        let input = "value";
        let result = cmake_condition(input).unwrap().1;
        assert_eq!(
            result,
            CMakeCondition::Value(CMakeValue::StringLiteral(String::from("value")))
        );
    }

    #[test]
    fn test_parse_condition_unary() {
        let input = "EXISTS /usr/include";
        let result = cmake_condition(input).unwrap().1;
        assert_eq!(
            result,
            CMakeCondition::UnaryTest {
                operator: "EXISTS".to_string(),
                value: Box::new(CMakeCondition::Value(CMakeValue::StringLiteral(
                    "/usr/include".to_string()
                )))
            }
        );
    }

    #[test]
    fn test_parse_condition_binary() {
        let input = "/usr/include STRLESS other";
        let result = cmake_condition(input).unwrap().1;
        assert_eq!(
            result,
            CMakeCondition::BinaryTest {
                operator: "STRLESS".to_string(),
                left: Box::new(CMakeCondition::Value(CMakeValue::StringLiteral(
                    "/usr/include".to_string()
                ))),
                right: Box::new(CMakeCondition::Value(CMakeValue::StringLiteral(
                    "other".to_string()
                )))
            }
        );
    }

    #[test]
    fn test_parse_version_binary_condition() {
        let input = "CUDA_VERSION VERSION_GREATER_EQUAL 10.1";
        let result = cmake_condition(input).unwrap().1;
        assert_eq!(
            result,
            CMakeCondition::BinaryTest {
                operator: "VERSION_GREATER_EQUAL".to_string(),
                left: Box::new(CMakeCondition::Value(CMakeValue::ArgumentSpecifier(
                    "CUDA_VERSION".to_string()
                ))),
                right: Box::new(CMakeCondition::Value(CMakeValue::StringLiteral(
                    "10.1".to_string()
                )))
            }
        );
    }

    #[test]
    fn test_parse_condition_unary_logical_operator() {
        let input = "NOT (EXISTS /usr/include)";
        let result = cmake_condition(input).unwrap().1;
        assert_eq!(
            result,
            CMakeCondition::UnaryLogicalOperator {
                operator: "NOT".to_string(),
                value: Box::new(CMakeCondition::Parentheses {
                    value: Box::new(CMakeCondition::UnaryTest {
                        operator: "EXISTS".to_string(),
                        value: Box::new(CMakeCondition::Value(CMakeValue::StringLiteral(
                            "/usr/include".to_string()
                        )))
                    })
                }),
            }
        );
    }

    #[test]
    fn test_parse_condition_binary_logical_operator() {
        let input = "true AND false";
        let result = cmake_condition(input).unwrap().1;
        assert_eq!(
            result,
            CMakeCondition::BinaryLogicalOperator {
                operator: "AND".to_string(),
                left: Box::new(CMakeCondition::Value(CMakeValue::StringLiteral(
                    "true".to_string()
                ))),
                right: Box::new(CMakeCondition::Value(CMakeValue::StringLiteral(
                    "false".to_string()
                )))
            }
        );
    }

    #[ignore]
    #[test]
    fn test_parse_condition_with_command_before_operator() {
        let input = r#"true   # comment
            AND
            false"#;
        let result = cmake_condition(input).unwrap();
        assert_eq!(
            result,
            (
                "",
                CMakeCondition::BinaryLogicalOperator {
                    operator: "AND".to_string(),
                    left: Box::new(CMakeCondition::Value(CMakeValue::StringLiteral(
                        "true".to_string()
                    ))),
                    right: Box::new(CMakeCondition::Comment {
                        content: " comment".to_string(),
                        tail: Some(Box::new(CMakeCondition::Value(CMakeValue::StringLiteral(
                            "false".to_string()
                        ))))
                    })
                }
            )
        );
    }

    #[test]
    fn test_parse_condition_with_comment() {
        let input = r#"true
            AND  # comment
            false"#;
        let result = cmake_condition(input).unwrap();
        assert_eq!(
            result,
            (
                "",
                CMakeCondition::BinaryLogicalOperator {
                    operator: "AND".to_string(),
                    left: Box::new(CMakeCondition::Value(CMakeValue::StringLiteral(
                        "true".to_string()
                    ))),
                    right: Box::new(CMakeCondition::Comment {
                        content: " comment".to_string(),
                        tail: Some(Box::new(CMakeCondition::Value(CMakeValue::StringLiteral(
                            "false".to_string()
                        ))))
                    })
                }
            )
        );
    }

    #[test]
    fn test_parse_complex_expression() {
        let input = "(true AND false OR NOT true) OR true OR (true AND NOT (false OR true))";
        let result = cmake_condition(input).unwrap().1;
        use CMakeCondition::*;
        use CMakeValue::*;
        assert_eq!(
            result,
            BinaryLogicalOperator {
                operator: String::from("OR"),
                left: Box::from(Parentheses {
                    value: Box::from(BinaryLogicalOperator {
                        operator: String::from("AND"),
                        left: Box::from(Value(StringLiteral(String::from("true")))),
                        right: Box::from(BinaryLogicalOperator {
                            operator: String::from("OR"),
                            left: Box::from(Value(StringLiteral(String::from("false")))),
                            right: Box::from(UnaryLogicalOperator {
                                operator: String::from("NOT"),
                                value: Box::from(Value(StringLiteral(String::from("true"))))
                            })
                        })
                    })
                }),
                right: Box::from(BinaryLogicalOperator {
                    operator: String::from("OR"),
                    left: Box::from(Value(StringLiteral(String::from("true")))),
                    right: Box::from(Parentheses {
                        value: Box::from(BinaryLogicalOperator {
                            operator: String::from("AND"),
                            left: Box::from(Value(StringLiteral(String::from("true")))),
                            right: Box::from(UnaryLogicalOperator {
                                operator: String::from("NOT"),
                                value: Box::from(Parentheses {
                                    value: Box::from(BinaryLogicalOperator {
                                        operator: String::from("OR"),
                                        left: Box::from(Value(StringLiteral(String::from(
                                            "false"
                                        )))),
                                        right: Box::from(Value(StringLiteral(String::from(
                                            "true"
                                        ))))
                                    })
                                })
                            })
                        })
                    })
                })
            }
        );
    }

    #[test]
    fn test_multiline_condition() {
        let input = r#"CMAKE_C_COMPILER_ID STREQUAL "Clang" OR
CMAKE_C_COMPILER_ID STREQUAL "AppleClang""#
            .trim();
        let result = all_consuming(cmake_condition)(input).unwrap().1;
        use CMakeCondition::*;
        use CMakeValue::*;
        assert_eq!(
            result,
            BinaryLogicalOperator {
                operator: String::from("OR"),
                left: Box::from(BinaryTest {
                    operator: String::from("STREQUAL"),
                    left: Box::from(Value(ArgumentSpecifier(String::from(
                        "CMAKE_C_COMPILER_ID"
                    )))),
                    right: Box::from(Value(QuotedString(String::from("Clang"))))
                }),
                right: Box::from(BinaryTest {
                    operator: String::from("STREQUAL"),
                    left: Box::from(Value(ArgumentSpecifier(String::from(
                        "CMAKE_C_COMPILER_ID"
                    )))),
                    right: Box::from(Value(QuotedString(String::from("AppleClang"))))
                })
            }
        );
    }
}
