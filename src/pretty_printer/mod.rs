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
use pretty::RcDoc;

use crate::parser::types::{
    CMakeCommand, CMakeDocument, CMakeStatement, CMakeValue, CmakeIfStatement,
};

impl CMakeValue {
    fn to_doc(&self) -> RcDoc<'static, ()> {
        match self {
            CMakeValue::QuotedString(str) => RcDoc::text(format!("\"{}\"", str)),
            CMakeValue::StringLiteral(str) => RcDoc::text(str.to_string()),
            CMakeValue::Comment(str) => RcDoc::text(format!("#{}", str))
                .flat_alt(RcDoc::text(format!("#{}", str)).append(RcDoc::hardline())),
            CMakeValue::ArgumentSpecifier(arg) => RcDoc::text(arg.to_string()),
        }
    }
}

impl CMakeCommand {
    fn print(&self) -> RcDoc<'static, ()> {
        let args = print_args(&self.args);

        RcDoc::text(self.name.to_string())
            .append("(")
            .append(args)
            .append(")")
            .group()
    }
}

fn print_args(args: &Vec<CMakeValue>) -> RcDoc<'static> {
    let args = print_args_to_vec(args);
    RcDoc::line_()
        .append(RcDoc::intersperse(args, RcDoc::line()))
        .append(RcDoc::line_())
        .nest(2)
        .group()
}

fn print_args_to_vec(args: &Vec<CMakeValue>) -> Vec<RcDoc<'static>> {
    if args.is_empty() {
        vec![]
    } else {
        let mut groups = vec![vec![&args[0]]];
        for arg in args.iter().skip(1) {
            if let CMakeValue::ArgumentSpecifier(_) = arg {
                groups.push(vec![arg]);
            } else if let CMakeValue::Comment(_) = arg {
                groups.push(vec![arg]);
            } else {
                groups.last_mut().unwrap().push(arg);
            }
        }

        groups
            .iter()
            .map(|values| {
                RcDoc::intersperse(values.iter().map(|value| value.to_doc()), RcDoc::line()).group()
            })
            .collect::<Vec<RcDoc>>()
    }
}

impl CmakeIfStatement {
    fn print(&self) -> RcDoc<'static> {
        let make_body = |body: &Vec<CMakeStatement>| {
            RcDoc::intersperse(body.iter().map(|statement| statement.print()), RcDoc::nil())
                .nest(2)
                .group()
        };
        let mut output = RcDoc::text("if(")
            .append(print_args(&self.base.condition))
            .append(RcDoc::text(")"))
            .append(make_body(&self.base.body));

        for else_if in &self.else_ifs {
            output = output
                .append("elseif(")
                .append(print_args(&else_if.condition))
                .append(RcDoc::text(")"))
                .append(make_body(&else_if.body));
        }

        if let Some(else_body) = &self.else_body {
            output = output
                .append("else(")
                .append(RcDoc::text(")"))
                .append(make_body(&else_body));
        }

        output = output.append(RcDoc::text("endif()"));
        output
    }
}

impl CMakeStatement {
    fn print(&self) -> RcDoc<'static, ()> {
        match self {
            CMakeStatement::Command(command) => command.print(),
            CMakeStatement::Comment(comment) => RcDoc::text(format!("#{}", comment)),
            CMakeStatement::Newline => RcDoc::hardline(),
            CMakeStatement::If(if_statement) => if_statement.print(),
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
                    if let CMakeStatement::Newline = statement {
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
    use crate::parser::types::{CmakeIfBase, CmakeIfStatement};

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

    #[test]
    fn test_print_if_statement() {
        let mut vec_writer = Vec::new();
        {
            let document = CMakeDocument {
                statements: vec![CMakeStatement::If(CmakeIfStatement {
                    base: CmakeIfBase {
                        condition: vec![CMakeValue::ArgumentSpecifier(String::from(
                            "CMAKE_COMPILER_IS_GNUCXX",
                        ))],
                        body: vec![
                            CMakeStatement::Newline,
                            CMakeStatement::Command(CMakeCommand {
                                name: String::from("foo"),
                                args: vec![],
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
                statements: vec![CMakeStatement::If(CmakeIfStatement {
                    base: CmakeIfBase {
                        condition: vec![CMakeValue::ArgumentSpecifier(String::from("a"))],
                        body: vec![
                            CMakeStatement::Newline,
                            CMakeStatement::If(CmakeIfStatement {
                                base: CmakeIfBase {
                                    condition: vec![CMakeValue::ArgumentSpecifier(String::from(
                                        "b",
                                    ))],
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
}
