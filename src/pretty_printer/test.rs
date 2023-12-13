use super::*;
use crate::parser::types::{CMakeIfBase, CMakeIfStatement};

#[test]
fn pretty_print_command_without_args() {
    let mut vec_writer = Vec::new();
    {
        let command = CMakeCommand {
            name: "foo".to_string(),
            args: vec![],
        };
        command.print().render(80, &mut vec_writer).unwrap();
    }
    let str = String::from_utf8(vec_writer).unwrap();
    assert_eq!(str, "foo()");
}

#[test]
fn pretty_print_command_with_single_arg() {
    let mut vec_writer = Vec::new();
    {
        let command = CMakeCommand {
            name: "cmake_version".to_string(),
            args: vec![CMakeValue::StringLiteral(String::from("1.2.3"))],
        };
        command.print().render(80, &mut vec_writer).unwrap();
    }
    let str = String::from_utf8(vec_writer).unwrap();
    assert_eq!(str, "cmake_version(1.2.3)");
}

#[test]
fn pretty_print_command_with_multiple_args() {
    let mut vec_writer = Vec::new();
    {
        let command = CMakeCommand {
            name: "foo".to_string(),
            args: vec![
                CMakeValue::StringLiteral(String::from("a")),
                CMakeValue::StringLiteral(String::from("b")),
                CMakeValue::StringLiteral(String::from("c")),
                CMakeValue::StringLiteral(String::from("d")),
                CMakeValue::StringLiteral(String::from("e")),
            ],
        };
        command.print().render(80, &mut vec_writer).unwrap();
    }
    let str = String::from_utf8(vec_writer).unwrap();
    assert_eq!(str, "foo(a b c d e)");
}

#[test]
fn pretty_print_command_with_long_line_args() {
    let mut vec_writer = Vec::new();
    {
        let command = CMakeCommand {
            name: "foo".to_string(),
            args: vec![
                CMakeValue::StringLiteral(String::from(
                    "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
                )),
                CMakeValue::StringLiteral(String::from(
                    "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
                )),
                CMakeValue::StringLiteral(String::from(
                    "cccccccccccccccccccccccccccccccccccccccccccccccccc",
                )),
                CMakeValue::StringLiteral(String::from(
                    "dddddddddddddddddddddddddddddddddddddddddddddddddd",
                )),
                CMakeValue::StringLiteral(String::from(
                    "eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee",
                )),
            ],
        };
        command.print().render(80, &mut vec_writer).unwrap();
    }
    let str = String::from_utf8(vec_writer).unwrap();
    assert_eq!(str, "foo(\n  aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa\n  bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb\n  cccccccccccccccccccccccccccccccccccccccccccccccccc\n  dddddddddddddddddddddddddddddddddddddddddddddddddd\n  eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee\n)");
}

#[test]
fn test_pretty_print_two_statements_has_no_blank_line() {
    let mut vec_writer = Vec::new();
    {
        let document = CMakeDocument {
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
            ],
        };
        document.print().render(80, &mut vec_writer).unwrap();
    }
    let str = String::from_utf8(vec_writer).unwrap();
    assert_eq!(str, "foo()\nbar()");
}

#[test]
fn test_args_still_stay_in_one_line_if_there_is_space() {
    let mut vec_writer = Vec::new();
    {
        let document = CMakeDocument {
            statements: vec![CMakeStatement::Command(CMakeCommand {
                name: "foo".to_string(),
                args: vec![
                    CMakeValue::ArgumentSpecifier(String::from("LANGUAGE")),
                    CMakeValue::ArgumentSpecifier(String::from("VERSION")),
                ],
            })],
        };
        document.print().render(80, &mut vec_writer).unwrap();
    }
    let str = String::from_utf8(vec_writer).unwrap();
    assert_eq!(str, "foo(LANGUAGE VERSION)");
}

#[test]
fn test_sample() {
    let mut vec_writer = Vec::new();
    let input = CMakeStatement::Command(CMakeCommand {
        name: String::from("set"),
        args: vec![
            CMakeValue::ArgumentSpecifier(String::from("CMAKE_CXX_STANDARD_REQUIRED")),
            CMakeValue::ArgumentSpecifier(String::from("ON")),
        ],
    });
    input.print().render(80, &mut vec_writer).unwrap();
    let str = String::from_utf8(vec_writer).unwrap();
    assert_eq!(str, "set(CMAKE_CXX_STANDARD_REQUIRED ON)");
}

#[test]
fn test_args_are_grouped_with_upper_case_args() {
    let mut vec_writer = Vec::new();
    {
        let document = CMakeDocument {
            statements: vec![CMakeStatement::Command(CMakeCommand {
                name: "foo".to_string(),
                args: vec![
                    CMakeValue::ArgumentSpecifier(String::from("LANGUAGE")),
                    CMakeValue::StringLiteral(String::from("cxx")),
                    CMakeValue::ArgumentSpecifier(String::from("VERSION")),
                    CMakeValue::StringLiteral(String::from("1234")),
                    CMakeValue::ArgumentSpecifier(String::from("OTHER")),
                    CMakeValue::StringLiteral(String::from("here")),
                    CMakeValue::ArgumentSpecifier(String::from("THING")),
                    CMakeValue::StringLiteral(String::from("0000000000000000000000000000000000")),
                ],
            })],
        };
        document.print().render(80, &mut vec_writer).unwrap();
    }
    let str = String::from_utf8(vec_writer).unwrap();
    assert_eq!(str, "foo(\n  LANGUAGE cxx\n  VERSION 1234\n  OTHER here\n  THING 0000000000000000000000000000000000\n)");
}

#[test]
fn test_print_if_statement() {
    let mut vec_writer = Vec::new();
    {
        let document = CMakeDocument {
            statements: vec![CMakeStatement::If(CMakeIfStatement {
                base: CMakeIfBase {
                    condition: CMakeCondition::Value(CMakeValue::ArgumentSpecifier(String::from(
                        "CMAKE_COMPILER_IS_GNUCXX",
                    ))),
                    body: vec![
                        CMakeStatement::Newline,
                        CMakeStatement::Command(CMakeCommand {
                            name: String::from("foo"),
                            args: vec![],
                        }),
                        CMakeStatement::Newline,
                    ],
                },
                else_ifs: vec![CMakeIfBase {
                    condition: CMakeCondition::Value(CMakeValue::ArgumentSpecifier(String::from(
                        "MSVC",
                    ))),
                    body: vec![
                        CMakeStatement::Newline,
                        CMakeStatement::Command(CMakeCommand {
                            name: String::from("bar"),
                            args: vec![],
                        }),
                        CMakeStatement::Newline,
                    ],
                }],
                else_body: None,
            })],
        };
        document.print().render(80, &mut vec_writer).unwrap();
    }
    let str = String::from_utf8(vec_writer).unwrap();
    assert_eq!(
        str,
        r#"
if(CMAKE_COMPILER_IS_GNUCXX)
  foo()
elseif(MSVC)
  bar()
endif()
    "#
        .trim()
    );
}

#[test]
fn test_print_nested_if_statement() {
    let mut vec_writer = Vec::new();
    {
        let document = CMakeDocument {
            statements: vec![CMakeStatement::If(CMakeIfStatement {
                base: CMakeIfBase {
                    condition: CMakeCondition::Value(CMakeValue::ArgumentSpecifier(String::from(
                        "a",
                    ))),
                    body: vec![
                        CMakeStatement::Newline,
                        CMakeStatement::If(CMakeIfStatement {
                            base: CMakeIfBase {
                                condition: CMakeCondition::Value(CMakeValue::ArgumentSpecifier(
                                    String::from("b"),
                                )),
                                body: vec![
                                    CMakeStatement::Newline,
                                    CMakeStatement::Command(CMakeCommand {
                                        name: String::from("foo"),
                                        args: vec![],
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
                else_body: Some(vec![
                    CMakeStatement::Newline,
                    CMakeStatement::Command(CMakeCommand {
                        name: String::from("bar"),
                        args: vec![],
                    }),
                    CMakeStatement::Newline,
                ]),
            })],
        };
        document.print().render(80, &mut vec_writer).unwrap();
    }
    let str = String::from_utf8(vec_writer).unwrap();
    assert_eq!(
        str,
        r#"
if(a)
  if(b)
    foo()
  endif()
else()
  bar()
endif()
    "#
        .trim()
    );
}

#[test]
fn test_preseving_newlines() {
    let document = CMakeDocument {
        statements: vec![CMakeStatement::Function(CMakeFunctionStatement {
            clause: vec![CMakeValue::StringLiteral(String::from("foo"))],
            body: vec![
                CMakeStatement::Newline,
                CMakeStatement::Command(CMakeCommand {
                    name: String::from("bar"),
                    args: vec![
                        // TODO we don't want these newlines
                        CMakeValue::StringLiteral(String::from("x")),
                        CMakeValue::StringLiteral(String::from("y")),
                        CMakeValue::StringLiteral(String::from("z")),
                    ],
                }),
                CMakeStatement::Newline,
            ],
        })],
    };
    let mut vec_writer = Vec::new();
    {
        document.print().render(80, &mut vec_writer).unwrap();
    }
    let str = String::from_utf8(vec_writer).unwrap();
    assert_eq!(
        str,
        r#"
function(foo)
  bar(x y z)
endfunction()
    "#
        .trim()
    )
}
