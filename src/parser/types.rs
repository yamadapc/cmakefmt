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

#[derive(Debug, PartialEq, PartialOrd)]
pub enum CMakeValue {
    ArgumentSpecifier(String),
    QuotedString(String),
    StringLiteral(String),
    Comment(String),
    BracketComment(CMakeBracketComment),
    Parenthesis(String),
}

#[derive(Debug, PartialEq, PartialOrd)]
pub enum CMakeCondition {
    Parentheses {
        // Parentheses ( and ).
        value: Box<CMakeCondition>,
    },
    UnaryTest {
        // Unary tests such as EXISTS, COMMAND, and DEFINED.
        operator: String,
        value: Box<CMakeCondition>,
    },
    BinaryTest {
        // Binary tests such as EQUAL, LESS, LESS_EQUAL, GREATER, GREATER_EQUAL, STREQUAL, STRLESS, STRLESS_EQUAL, STRGREATER, STRGREATER_EQUAL, VERSION_EQUAL, VERSION_LESS, VERSION_LESS_EQUAL, VERSION_GREATER, VERSION_GREATER_EQUAL, PATH_EQUAL, and MATCHES.
        operator: String,
        left: Box<CMakeCondition>,
        right: Box<CMakeCondition>,
    },
    UnaryLogicalOperator {
        // Unary logical operator NOT.
        operator: String,
        value: Box<CMakeCondition>,
    },
    BinaryLogicalOperator {
        // Binary logical operators AND and OR, from left to right, without any short-circuit.
        operator: String,
        left: Box<CMakeCondition>,
        right: Box<CMakeCondition>,
    },
    Value(CMakeValue),
}

#[derive(Debug, PartialEq, PartialOrd)]
pub struct CMakeCommand {
    pub name: String,
    pub args: Vec<CMakeValue>,
}

#[derive(Debug, PartialEq, PartialOrd)]
pub struct CMakeIfBase {
    pub condition: CMakeCondition,
    pub body: Vec<CMakeStatement>,
}

#[derive(Debug, PartialEq, PartialOrd)]
pub struct CMakeIfStatement {
    pub base: CMakeIfBase,
    pub else_ifs: Vec<CMakeIfBase>,
    pub else_body: Option<Vec<CMakeStatement>>,
}

#[derive(Debug, PartialEq, PartialOrd)]
pub struct CMakeCommandGroup {
    pub clause: Vec<CMakeValue>,
    pub body: Vec<CMakeStatement>,
    pub end_clause: Vec<CMakeValue>,
}

#[derive(Debug, PartialEq, PartialOrd)]
pub struct CMakeForEachStatement {
    pub group: CMakeCommandGroup,
}

#[derive(Debug, PartialEq, PartialOrd)]
pub struct CMakeFunctionStatement {
    pub group: CMakeCommandGroup,
}

#[derive(Debug, PartialEq, PartialOrd)]
pub struct CMakeMacroStatement {
    pub group: CMakeCommandGroup,
}

#[derive(Debug, PartialEq, PartialOrd)]
pub struct CMakeBlockStatement {
    pub group: CMakeCommandGroup,
}

#[derive(Debug, PartialEq, PartialOrd)]
pub struct CMakeBracketComment {
    pub delimiter: String,
    pub contents: String,
}

#[derive(Debug, PartialEq, PartialOrd)]
pub enum CMakeStatement {
    If(CMakeIfStatement),
    For(CMakeForEachStatement),
    Function(CMakeFunctionStatement),
    Block(CMakeBlockStatement),
    Macro(CMakeMacroStatement),
    Command(CMakeCommand),
    BracketComment(CMakeBracketComment),
    Comment(String),
    Newline,
}

#[derive(Debug, PartialEq, PartialOrd)]
pub struct CMakeDocument {
    pub statements: Vec<CMakeStatement>,
}
