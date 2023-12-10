use nom::combinator::all_consuming;

use super::*;

#[test]
fn test_parse_empty_string_literal() {
    let result = all_consuming(cmake_string_literal)("").unwrap_err();
    assert!(matches!(result, nom::Err::Error(_)));
}

#[test]
fn test_parse_string_literal_with_special_characters() {
    let (_, result) = all_consuming(cmake_string_literal)("foo_bar123").unwrap();
    assert_eq!(result, CMakeValue::StringLiteral("foo_bar123".to_string()));
}

#[test]
fn test_parse_quoted_string_with_escaped_quotes() {
    let (_, result) = all_consuming(cmake_quoted_string_literal)(r#""foo""#).unwrap();
    assert_eq!(result, CMakeValue::QuotedString(r#"foo"#.to_string()));
}

#[test]
fn test_parse_quoted_string_with_inner_quotes() {
    let (_, result) = all_consuming(cmake_quoted_string_literal)(r#""foo\"bar""#).unwrap();
    assert_eq!(result, CMakeValue::QuotedString(r#"foo"bar"#.to_string()));
}

#[test]
fn test_parse_quoted_string_empty() {
    let (_, result) = all_consuming(cmake_quoted_string_literal)(r#""""#).unwrap();
    assert_eq!(result, CMakeValue::QuotedString("".to_string()));
}

#[test]
fn test_parse_quoted_string_with_newline() {
    let (_, result) = all_consuming(cmake_quoted_string_literal)("\"foo\nbar\"").unwrap();
    assert_eq!(result, CMakeValue::QuotedString("foo\nbar".to_string()));
}

#[test]
fn test_parse_comment_empty() {
    let (_, result) = all_consuming(cmake_comment)("#").unwrap();
    assert_eq!(result, "");
}

#[test]
fn test_parse_comment_with_spaces() {
    let (_, result) = all_consuming(cmake_comment)("#    ").unwrap();
    assert_eq!(result, "    ");
}

#[test]
fn test_parse_comment_with_content() {
    let (_, result) = all_consuming(cmake_comment)("# This is a test").unwrap();
    assert_eq!(result, " This is a test");
}

#[test]
fn test_parse_comment_with_special_characters() {
    let (_, result) = all_consuming(cmake_comment)("# Special_chars: @#$%^&*()").unwrap();
    assert_eq!(result, " Special_chars: @#$%^&*()");
}

#[test]
fn test_parse_command_name_simple() {
    let (_, result) = all_consuming(cmake_command_name)("foo").unwrap();
    assert_eq!(result, "foo");
}

#[test]
fn test_parse_command_name_with_underscores() {
    let (_, result) = all_consuming(cmake_command_name)("foo_bar").unwrap();
    assert_eq!(result, "foo_bar");
}

#[test]
fn test_parse_command_name_with_numbers() {
    let (_, result) = all_consuming(cmake_command_name)("foo123").unwrap();
    assert_eq!(result, "foo123");
}

#[test]
fn test_parse_command_name_fail_with_special_char() {
    let result = all_consuming(cmake_command_name)("foo!bar").unwrap_err();
    assert!(matches!(result, nom::Err::Error(_)));
}

#[test]
fn test_parse_command_name_fail_with_space() {
    let result = all_consuming(cmake_command_name)("foo bar").unwrap_err();
    assert!(matches!(result, nom::Err::Error(_)));
}

#[test]
fn test_parse_string_unquoted() {
    let (_, result) = all_consuming(cmake_value)("foo").unwrap();
    assert_eq!(result, CMakeValue::StringLiteral("foo".to_string()));
}

#[test]
fn test_parse_comment() {
    let (_, result) = all_consuming(cmake_comment)("# This is a comment").unwrap();
    assert_eq!(result, " This is a comment");
}

#[test]
fn test_parse_command_with_comment() {
    let (_, result) = all_consuming(cmake_parser)("foo(bar) # comment").unwrap();
    assert_eq!(
        result,
        CMakeDocument {
            statements: vec![
                CMakeStatement::Command(CMakeCommand {
                    name: "foo".to_string(),
                    args: vec![CMakeValue::StringLiteral("bar".to_string())],
                }),
                CMakeStatement::Comment(" comment".to_string())
            ]
        }
    );
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
fn test_parse_empty_command() {
    let (_, result) = all_consuming(cmake_command)("foo()").unwrap();
    assert_eq!(
        result,
        CMakeCommand {
            name: "foo".to_string(),
            args: vec![],
        }
    );
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
fn test_parse_command_multiple_args() {
    let (_, result) = all_consuming(cmake_command)("foo(bar baz)").unwrap();
    assert_eq!(
        result,
        CMakeCommand {
            name: "foo".to_string(),
            args: vec![
                CMakeValue::StringLiteral("bar".to_string()),
                CMakeValue::StringLiteral("baz".to_string()),
            ],
        }
    );
}

#[test]
fn test_parse_command_with_variable() {
    let (_, result) =
        all_consuming(cmake_command)("foo(\n  ${CMAKE_CURRENT_LIST_DIR}/vendor\n)").unwrap();
    assert_eq!(
        result,
        CMakeCommand {
            name: "foo".to_string(),
            args: vec![CMakeValue::StringLiteral(
                "${CMAKE_CURRENT_LIST_DIR}/vendor".to_string()
            ),],
        }
    );
}

#[test]
fn test_parse_command_with_line_break() {
    let (_, result) =
        all_consuming(cmake_command)("foo(\n  name\n  VERSION bar\n  LANGUAGE ZIG\n)").unwrap();
    assert_eq!(
        result,
        CMakeCommand {
            name: "foo".to_string(),
            args: vec![
                CMakeValue::StringLiteral("name".to_string()),
                CMakeValue::ArgumentSpecifier("VERSION".to_string()),
                CMakeValue::StringLiteral("bar".to_string()),
                CMakeValue::ArgumentSpecifier("LANGUAGE".to_string()),
                CMakeValue::ArgumentSpecifier("ZIG".to_string()),
            ],
        }
    );
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
            CMakeValue::StringLiteral(String::from("${CMAKE_CXX_COMPILER_ID}")),
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

#[test]
fn test_parse_if_statement_with_single_condition() {
    let input = "if(ON)\nfoo()\nendif()";
    let (_, result) = all_consuming(cmake_if_block)(input).unwrap();
    assert_eq!(
        result,
        CMakeStatement::If(CmakeIfStatement {
            base: CmakeIfBase {
                condition: vec![CMakeValue::ArgumentSpecifier(String::from("ON"))],
                body: vec![
                    CMakeStatement::Newline,
                    CMakeStatement::Command(CMakeCommand {
                        name: String::from("foo"),
                        args: vec![]
                    }),
                    CMakeStatement::Newline,
                ],
            },
            else_ifs: vec![],
            else_body: None,
        })
    );
}

#[test]
fn test_parse_if_statement_with_else() {
    let input = "if(OFF)\nfoo()\nelse()\nbar()\nendif()";
    let (_, result) = all_consuming(cmake_if_block)(input).unwrap();
    assert_eq!(
        result,
        CMakeStatement::If(CmakeIfStatement {
            base: CmakeIfBase {
                condition: vec![CMakeValue::ArgumentSpecifier(String::from("OFF"))],
                body: vec![
                    CMakeStatement::Newline,
                    CMakeStatement::Command(CMakeCommand {
                        name: String::from("foo"),
                        args: vec![]
                    }),
                    CMakeStatement::Newline,
                ],
            },
            else_ifs: vec![],
            else_body: Some(vec![
                CMakeStatement::Newline,
                CMakeStatement::Command(CMakeCommand {
                    name: String::from("bar"),
                    args: vec![]
                }),
                CMakeStatement::Newline,
            ]),
        })
    );
}

#[test]
fn test_parse_nested_if_statements() {
    let input = "if(ON)\nif(OFF)\nfoo()\nendif()\nendif()";
    let (_, result) = all_consuming(cmake_if_block)(input).unwrap();
    assert_eq!(
        result,
        CMakeStatement::If(CmakeIfStatement {
            base: CmakeIfBase {
                condition: vec![CMakeValue::ArgumentSpecifier(String::from("ON"))],
                body: vec![
                    CMakeStatement::Newline,
                    CMakeStatement::If(CmakeIfStatement {
                        base: CmakeIfBase {
                            condition: vec![CMakeValue::ArgumentSpecifier(String::from("OFF"))],
                            body: vec![
                                CMakeStatement::Newline,
                                CMakeStatement::Command(CMakeCommand {
                                    name: String::from("foo"),
                                    args: vec![]
                                }),
                                CMakeStatement::Newline,
                            ],
                        },
                        else_ifs: vec![],
                        else_body: None,
                    }),
                    CMakeStatement::Newline,
                ],
            },
            else_ifs: vec![],
            else_body: None,
        })
    );
}

#[test]
fn test_parse_multiple_commands() {
    let input = "foo()\nbar()";
    let (_, result) = all_consuming(cmake_parser)(input).unwrap();
    assert_eq!(
        result,
        CMakeDocument {
            statements: vec![
                CMakeStatement::Command(CMakeCommand {
                    name: "foo".to_string(),
                    args: vec![],
                }),
                CMakeStatement::Newline,
                CMakeStatement::Command(CMakeCommand {
                    name: "bar".to_string(),
                    args: vec![],
                }),
            ]
        }
    );
}

#[test]
fn test_parse_command_with_nested_parenthesis() {
    let input = "foo((bar baz))";
    let (_, result) = all_consuming(cmake_command)(input).unwrap();
    assert_eq!(
        result,
        CMakeCommand {
            name: "foo".to_string(),
            args: vec![
                CMakeValue::Parenthesis("(".to_string()),
                CMakeValue::StringLiteral("bar".to_string()),
                CMakeValue::StringLiteral("baz".to_string()),
                CMakeValue::Parenthesis(")".to_string()),
            ],
        }
    );
}
