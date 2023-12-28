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
    CMakeBlockStatement, CMakeCommand, CMakeCommandGroup, CMakeCondition, CMakeDocument,
    CMakeForEachStatement, CMakeFunctionStatement, CMakeIfStatement, CMakeMacroStatement,
    CMakeStatement, CMakeValue,
};

impl CMakeValue {
    fn to_doc(&self) -> RcDoc<'static, ()> {
        match self {
            CMakeValue::QuotedString(str) => RcDoc::text(format!("\"{}\"", str)),
            CMakeValue::StringLiteral(str) => RcDoc::text(str.to_string()),
            CMakeValue::Comment(str) => RcDoc::text(format!("#{}", str))
                .flat_alt(RcDoc::text(format!("#{}", str)).append(RcDoc::hardline())),
            CMakeValue::ArgumentSpecifier(arg) => RcDoc::text(arg.to_string()),
            CMakeValue::Parenthesis(char) => RcDoc::text(char.to_string()),
        }
    }
}

impl CMakeCommand {
    fn print(&self) -> RcDoc<'static, ()> {
        let args = print_args(&self.args, false);

        RcDoc::text(self.name.to_string())
            .append("(")
            .append(args)
            .append(")")
            .group()
    }
}

fn print_args(args: &[CMakeValue], grouping_disabled: bool) -> RcDoc<'static> {
    let args = print_args_to_vec(args, grouping_disabled);
    RcDoc::line_()
        .append(RcDoc::intersperse(args, RcDoc::line()))
        .append(RcDoc::line_())
        .nest(2)
        .group()
}

fn print_args_to_vec(args: &[CMakeValue], grouping_disabled: bool) -> Vec<RcDoc<'static>> {
    if args.is_empty() {
        vec![]
    } else {
        let mut groups = vec![vec![&args[0]]];
        for arg in args.iter().skip(1) {
            if grouping_disabled {
                groups.last_mut().unwrap().push(arg);
            } else {
                if let CMakeValue::ArgumentSpecifier(_) = arg {
                    groups.push(vec![arg]);
                } else if let CMakeValue::Comment(_) = arg {
                    groups.push(vec![arg]);
                } else {
                    groups.last_mut().unwrap().push(arg);
                }
            }
        }

        groups
            .iter()
            .map(|values| {
                let result = RcDoc::intersperse(
                    values.iter().map(|value| value.to_doc()),
                    if grouping_disabled {
                        RcDoc::softline()
                    } else {
                        RcDoc::line()
                    },
                );
                if grouping_disabled {
                    result
                } else {
                    result.group()
                }
            })
            .collect::<Vec<RcDoc>>()
    }
}

impl CMakeCondition {
    fn print(&self) -> RcDoc<'static> {
        match self {
            CMakeCondition::Parentheses { value } => RcDoc::text("(")
                .append(value.print())
                .append(RcDoc::text(")")),
            CMakeCondition::UnaryTest { value, operator } => RcDoc::text(operator.to_string())
                .append(RcDoc::space())
                .append(value.print()),
            CMakeCondition::BinaryTest {
                operator,
                left,
                right,
            } => left
                .print()
                .append(RcDoc::space())
                .append(RcDoc::text(operator.to_string()))
                .append(RcDoc::space())
                .append(right.print()),
            CMakeCondition::UnaryLogicalOperator { value, operator } => {
                RcDoc::text(operator.to_string())
                    .append(RcDoc::space())
                    .append(value.print())
            }
            CMakeCondition::BinaryLogicalOperator {
                operator,
                left,
                right,
            } => left
                .print()
                .append(RcDoc::space())
                .append(RcDoc::text(operator.to_string()))
                .append(RcDoc::space())
                .append(right.print()),
            CMakeCondition::Value(value) => value.to_doc(),
        }
    }
}

impl CMakeIfStatement {
    fn print(&self) -> RcDoc<'static> {
        let make_body = |body: &Vec<CMakeStatement>| {
            RcDoc::intersperse(body.iter().map(|statement| statement.print()), RcDoc::nil())
                .nest(2)
                .group()
        };
        let mut output = RcDoc::text("if(")
            .append(self.base.condition.print())
            .append(RcDoc::text(")"))
            .append(make_body(&self.base.body));

        for else_if in &self.else_ifs {
            output = output
                .append("elseif(")
                .append(else_if.condition.print())
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

impl CMakeCommandGroup {
    fn print(&self, name: &str) -> RcDoc<'static> {
        print_clause_body(name, &self.clause, &self.body)
    }
}

fn print_clause_body(
    keyword: &str,
    clause: &[CMakeValue],
    body: &[CMakeStatement],
) -> RcDoc<'static> {
    RcDoc::intersperse(
        [
            RcDoc::text(format!("{}(", keyword))
                .append(print_args(clause, false))
                .append(RcDoc::text(")"))
                .group(),
            RcDoc::intersperse(body.iter().map(|statement| statement.print()), RcDoc::nil())
                .nest(2)
                .group(),
            RcDoc::text(format!("end{}()", keyword)).group(),
        ],
        RcDoc::nil(),
    )
}

impl CMakeForEachStatement {
    fn print(&self) -> RcDoc<'static> {
        self.group.print("foreach")
    }
}

impl CMakeFunctionStatement {
    fn print(&self) -> RcDoc<'static> {
        self.group.print("function")
    }
}

impl CMakeMacroStatement {
    fn print(&self) -> RcDoc<'static> {
        self.group.print("macro")
    }
}

impl CMakeBlockStatement {
    fn print(&self) -> RcDoc<'static> {
        self.group.print("block")
    }
}

impl CMakeStatement {
    fn print(&self) -> RcDoc<'static, ()> {
        match self {
            CMakeStatement::Command(command) => command.print(),
            CMakeStatement::Comment(comment) => RcDoc::text(format!("#{}", comment)),
            CMakeStatement::Newline => RcDoc::hardline(),
            CMakeStatement::If(if_statement) => if_statement.print(),
            CMakeStatement::For(for_statement) => for_statement.print(),
            CMakeStatement::Function(fn_statement) => fn_statement.print(),
            CMakeStatement::Macro(m_statement) => m_statement.print(),
            CMakeStatement::Block(s) => s.print(),
        }
        .group()
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
mod test;
