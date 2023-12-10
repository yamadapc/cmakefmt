use pretty::RcDoc;

use crate::parser::types::{CMakeCommand, CMakeDocument, CMakeStatement, CMakeValue};

impl CMakeValue {
    fn to_doc(&self) -> RcDoc<'static, ()> {
        match self {
            CMakeValue::QuotedString(str) => RcDoc::text(format!("\"{}\"", str)),
            CMakeValue::StringLiteral(str) => RcDoc::text(format!("{}", str)),
            CMakeValue::Comment(str) => RcDoc::text(format!("#{}", str)),
            CMakeValue::ArgumentSpecifier(arg) => RcDoc::text(format!("{}", arg)),
        }
    }
}

impl CMakeCommand {
    fn print(&self) -> RcDoc<'static, ()> {
        RcDoc::text(self.name.to_string())
            .append("(")
            .append(
                RcDoc::line_()
                    .append(RcDoc::intersperse(
                        {
                            if self.args.is_empty() {
                                vec![]
                            } else {
                                let mut groups = vec![vec![&self.args[0]]];
                                for arg in self.args.iter().skip(1) {
                                    if let CMakeValue::ArgumentSpecifier(_) = arg {
                                        groups.push(vec![arg]);
                                    } else {
                                        groups.last_mut().unwrap().push(arg);
                                    }
                                }
                                groups
                                    .iter()
                                    .map(|values| {
                                        RcDoc::intersperse(
                                            values.iter().map(|value| value.to_doc()),
                                            RcDoc::line(),
                                        )
                                        .group()
                                    })
                                    .collect::<Vec<RcDoc>>()
                            }
                        },
                        RcDoc::line(),
                    ))
                    .append(RcDoc::line_())
                    .nest(2)
                    .group(),
            )
            .append(")")
            .group()
    }
}

impl CMakeStatement {
    fn print(&self) -> RcDoc<'static, ()> {
        match self {
            CMakeStatement::CMakeCommandStatement(command) => command.print(),
            CMakeStatement::CMakeCommentStatement(comment) => RcDoc::text(format!("#{}", comment)),
            CMakeStatement::CMakeNewline => RcDoc::hardline(),
        }
    }
}

impl CMakeDocument {
    pub(crate) fn print(&self) -> RcDoc<'static, ()> {
        RcDoc::intersperse(
            {
                let mut result = vec![];
                let mut newline_count = 0;
                for statement in self.statements.iter() {
                    if let CMakeStatement::CMakeNewline = statement {
                        if newline_count > 2 {
                            continue;
                        }
                        newline_count += 1;
                    } else {
                        newline_count = 0;
                    }

                    result.push(statement.print().group())
                }
                result
            },
            RcDoc::nil(),
        )
    }
}

#[cfg(test)]
mod test {
    use super::*;

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
                    CMakeStatement::CMakeCommandStatement(CMakeCommand {
                        name: "foo".to_string(),
                        args: vec![],
                    }),
                    CMakeStatement::CMakeNewline,
                    CMakeStatement::CMakeCommandStatement(CMakeCommand {
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
                statements: vec![CMakeStatement::CMakeCommandStatement(CMakeCommand {
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
        let input = CMakeStatement::CMakeCommandStatement(CMakeCommand {
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
                statements: vec![CMakeStatement::CMakeCommandStatement(CMakeCommand {
                    name: "foo".to_string(),
                    args: vec![
                        CMakeValue::ArgumentSpecifier(String::from("LANGUAGE")),
                        CMakeValue::StringLiteral(String::from("cxx")),
                        CMakeValue::ArgumentSpecifier(String::from("VERSION")),
                        CMakeValue::StringLiteral(String::from("1234")),
                        CMakeValue::ArgumentSpecifier(String::from("OTHER")),
                        CMakeValue::StringLiteral(String::from("here")),
                        CMakeValue::ArgumentSpecifier(String::from("THING")),
                        CMakeValue::StringLiteral(String::from(
                            "0000000000000000000000000000000000",
                        )),
                    ],
                })],
            };
            document.print().render(80, &mut vec_writer).unwrap();
        }
        let str = String::from_utf8(vec_writer).unwrap();
        assert_eq!(str, "foo(\n  LANGUAGE cxx\n  VERSION 1234\n  OTHER here\n  THING 0000000000000000000000000000000000\n)");
    }
}