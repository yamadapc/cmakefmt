use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::error::{ParseError, VerboseError};

use crate::parser::types::CMakeCondition;
use crate::parser::{cmake_value, IResult};

fn cmake_condition_parentheses(input: &str) -> IResult<&str, CMakeCondition> {
    let (input, _) = nom::character::complete::char('(')(input)?;
    let (input, condition) = cmake_condition(input)?;
    let (input, _) = nom::character::complete::char(')')(input)?;
    Ok((
        input,
        CMakeCondition::Parentheses {
            value: Box::new(condition),
        },
    ))
}

fn cmake_condition_unary_test(input: &str) -> IResult<&str, CMakeCondition> {
    let (input, operator) = alt((
        tag("EXISTS"),
        tag("COMMAND"),
        tag("DEFINED"),
        tag("POLICY"),
        tag("TARGET"),
    ))(input)?;
    let (input, _) = nom::character::complete::multispace1(input)?;
    let (input, value) = cmake_condition(input)?;
    Ok((
        input,
        CMakeCondition::UnaryTest {
            operator: operator.to_string(),
            value: Box::new(value),
        },
    ))
}

fn cmake_condition_binary_test(input: &str) -> IResult<&str, CMakeCondition> {
    let (input, left) = cmake_condition_value(input)?;
    let (input, _) = nom::character::complete::multispace1(input)?;
    let (input, operator) = alt((
        tag("EQUAL"),
        tag("LESS"),
        tag("LESS_EQUAL"),
        tag("GREATER"),
        tag("GREATER_EQUAL"),
        tag("STREQUAL"),
        tag("STRLESS"),
        tag("STRLESS_EQUAL"),
        tag("STRGREATER"),
        tag("STRGREATER_EQUAL"),
        tag("VERSION_EQUAL"),
        tag("VERSION_LESS"),
        tag("VERSION_LESS_EQUAL"),
        tag("VERSION_GREATER"),
        tag("VERSION_GREATER_EQUAL"),
        tag("PATH_EQUAL"),
        tag("MATCHES"),
    ))(input)?;
    let (input, _) = nom::character::complete::multispace1(input)?;
    let (input, right) = cmake_condition_value(input)?;
    Ok((
        input,
        CMakeCondition::BinaryTest {
            operator: operator.to_string(),
            left: Box::new(left),
            right: Box::new(right),
        },
    ))
}

fn cmake_condition_value(input: &str) -> IResult<&str, CMakeCondition> {
    let (input, value) = cmake_value(input)?;
    Ok((input, CMakeCondition::Value(value)))
}

fn cmake_condition_unary_logical_operator(input: &str) -> IResult<&str, CMakeCondition> {
    let (input, operator) = tag("NOT")(input)?;
    let (input, _) = nom::character::complete::multispace1(input)?;
    let (input, value) = cmake_condition(input)?;
    Ok((
        input,
        CMakeCondition::UnaryLogicalOperator {
            operator: operator.to_string(),
            value: Box::new(value),
        },
    ))
}

fn cmake_condition_binary_logical_operator(input: &str) -> IResult<&str, CMakeCondition> {
    let (input, left) =
        cmake_condition_inner(input, Some(cmake_condition_binary_logical_operator))?;
    let (input, _) = nom::character::complete::multispace1(input)?;
    let (input, operator) = alt((tag("AND"), tag("OR")))(input)?;
    let (input, _) = nom::character::complete::multispace1(input)?;
    let (input, right) = cmake_condition(input)?;
    Ok((
        input,
        CMakeCondition::BinaryLogicalOperator {
            operator: operator.to_string(),
            left: Box::new(left),
            right: Box::new(right),
        },
    ))
}

fn cmake_condition_inner(
    input: &str,
    ignore: Option<fn(&str) -> IResult<&str, CMakeCondition>>,
) -> IResult<&str, CMakeCondition> {
    let mut options = vec![
        cmake_condition_binary_logical_operator,
        cmake_condition_binary_test,
        cmake_condition_unary_logical_operator,
        cmake_condition_unary_test,
        cmake_condition_parentheses,
        cmake_condition_value,
    ];
    if let Some(to_ignore) = ignore {
        options = options
            .into_iter()
            .filter(|x| x != &to_ignore)
            .collect::<Vec<_>>();
    }
    let parser = alt_list(&options);
    parser(input)
}

pub fn cmake_condition(input: &str) -> IResult<&str, CMakeCondition> {
    cmake_condition_inner(input, None)
}

fn alt_list<'a, T>(
    parsers: &'a [impl Fn(&str) -> IResult<&str, T>],
) -> impl Fn(&str) -> IResult<&str, T> + 'a {
    return move |input| -> IResult<&str, T> {
        for parser in parsers {
            let result = parser(input);
            if result.is_ok() {
                return result;
            }
        }
        Err(nom::Err::Error(VerboseError::from_error_kind(
            input,
            nom::error::ErrorKind::Alt,
        )))
    };
}

#[cfg(test)]
mod test {
    use nom::combinator::all_consuming;

    use crate::parser::types::CMakeValue;

    use super::*;

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
