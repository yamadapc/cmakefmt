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

use nom::bytes::complete::{tag_no_case, take_till};
use nom::combinator::{map, opt};
use nom::error::{context, ParseError};
use nom::multi::many0;
use nom::{
    branch::alt,
    bytes::complete::{tag, take_till1},
    character::complete::{char, multispace0, multispace1, space0},
    error::ErrorKind,
    multi::{many1, separated_list0},
    sequence::{delimited, tuple},
    InputTakeAtPosition, Parser,
};
use nom_supreme::context::ContextError;
use nom_supreme::multi::parse_separated_terminated;
use nom_supreme::ParserExt;

use crate::parser::parse_condition::cmake_condition;
use crate::parser::types::{
    CMakeBlockStatement, CMakeCommand, CMakeCommandGroup, CMakeDocument, CMakeForEachStatement,
    CMakeFunctionStatement, CMakeIfBase, CMakeIfStatement, CMakeMacroStatement, CMakeStatement,
    CMakeValue,
};

pub mod types;

const RESERVED_WORDS: [&str; 12] = [
    "if",
    "elseif",
    "else",
    "endif",
    "foreach",
    "endforeach",
    "function",
    "endfunction",
    "macro",
    "endmacro",
    "block",
    "endblock",
];

pub type ErrorType<I> = nom_supreme::error::ErrorTree<I>;
pub type IResult<I, O> = Result<(I, O), nom::Err<ErrorType<I>>>;

fn cmake_comment(input: &str) -> IResult<&str, &str> {
    let comment_start = char('#');
    let comment_contents = take_till(|item| item == '\n');
    map(tuple((comment_start, comment_contents)), |(_, comment)| {
        comment
    })(input)
}

fn cmake_command_name(input: &str) -> IResult<&str, &str> {
    input.split_at_position1_complete(
        |item| !item.is_alphanumeric() && item != '_',
        ErrorKind::AlphaNumeric,
    )
}

fn cmake_quoted_string_literal(input: &str) -> IResult<&str, CMakeValue> {
    map(strings::parse_string, CMakeValue::QuotedString)(input)
}

fn cmake_string_part(input: &str) -> IResult<&str, String> {
    let is_invalid_char = |item: char| {
        item.is_whitespace() || item == ')' || item == '(' || item == '{' || item == '}'
    };
    context(
        "string_part",
        map(
            many1(alt((
                map(take_till1(is_invalid_char), |item: &str| item.to_string()),
                // $(variable)
                map(delimited(char('('), cmake_string_part, char(')')), |s| {
                    format!("({})", s)
                }),
                // ${variable}
                map(delimited(char('{'), cmake_string_part, char('}')), |s| {
                    format!("{{{}}}", s)
                }),
            ))),
            |result| result.join(""),
        ),
    )(input)
}

#[inline]
fn cmake_string_literal(input: &str) -> IResult<&str, CMakeValue> {
    let (input, result) = cmake_string_part(input)?;
    if result
        .chars()
        .all(|c| c.is_uppercase() || c == '_' || c.is_numeric())
    {
        return Ok((input, CMakeValue::ArgumentSpecifier(result.to_string())));
    }
    Ok((input, CMakeValue::StringLiteral(result)))
}

fn cmake_value(input: &str) -> IResult<&str, CMakeValue> {
    context(
        "Value",
        alt((
            context(
                "comment",
                cmake_comment.map(|item| CMakeValue::Comment(item.to_string())),
            ),
            context("quoted_string_literal", cmake_quoted_string_literal),
            context("string_literal", cmake_string_literal),
        )),
    )(input)
}

fn cmake_command(input: &str) -> IResult<&str, CMakeCommand> {
    let (input, name) = cmake_command_name(input)?;
    if RESERVED_WORDS.contains(&&*name.to_lowercase()) {
        return Err(nom::Err::Error(ErrorType::add_context(
            input,
            "reserved word can't be used as command",
            ErrorType::from_error_kind(input, ErrorKind::AlphaNumeric),
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
        separated_list0(multispace1, cmake_arg_list_inner),
        char(')').map(|_| CMakeValue::Parenthesis(")".to_string())),
    ))(input)?;

    let mut result = vec![start];
    result.extend(inner.into_iter().flatten());
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

fn cmake_else_if_block(input: &str) -> IResult<&str, CMakeIfBase> {
    let base = tuple((
        tag_no_case("elseif"),
        multispace0,
        tag("("),
        multispace0,
        cmake_condition,
        multispace0,
        tag(")"),
        parse_statement_list(),
    ));
    let mut inner = map(base, |(_, _, _, _, condition, _, _, body)| CMakeIfBase {
        condition,
        body,
    });

    inner(input)
}

fn cmake_if_group(input: &str) -> IResult<&str, CMakeStatement> {
    let if_start = tuple((tag_no_case("if"), multispace0, tag("("), multispace0));
    let condition = cmake_condition;
    let if_end = tuple((multispace0, tag(")")));
    let parse_condition = context(
        "parse_if_condition",
        map(tuple((if_start, condition, if_end)), |(_, condition, _)| {
            condition
        }),
    );
    let parse_if_statements = context("parse_if_statements", parse_statement_list());
    let parse_else_if_blocks = context(
        "parse_else_if_blocks",
        many0(delimited(space0, cmake_else_if_block, space0)),
    );
    let parse_else_block = context(
        "parse_else_block",
        opt(delimited(
            space0,
            tuple((skip_empty_command("else"), parse_statement_list())),
            space0,
        )),
    );
    let parse_endif = context("parse_endif", skip_empty_command("endif"));

    let parse_if_statement_tuple = tuple((
        parse_condition,
        parse_if_statements,
        parse_else_if_blocks,
        parse_else_block,
        parse_endif,
    ));

    let mut parse_if_statement = map(
        parse_if_statement_tuple,
        |(condition, body, else_ifs, else_body, _)| {
            CMakeStatement::If(CMakeIfStatement {
                base: CMakeIfBase { condition, body },
                else_ifs,
                else_body: else_body.map(|(_, body)| body),
            })
        },
    );

    parse_if_statement(input)
}

fn parse_statement_list() -> impl FnMut(&str) -> IResult<&str, Vec<CMakeStatement>> {
    |input| many0(delimited(space0, cmake_statement, space0))(input)
}

fn cmake_clause_body_block<'a>(
    keyword: &'a str,
) -> impl FnMut(&str) -> IResult<&str, CMakeCommandGroup> + 'a {
    let keyword_end = format!("end{}", keyword);
    move |input| {
        let prefix = tuple((tag_no_case(keyword), space0));
        let body = parse_statement_list();
        let base = tuple((prefix, cmake_args, body, skip_empty_command(&keyword_end)));

        let mut parser = map(base, |(_, clause, body, end_clause)| CMakeCommandGroup {
            clause,
            body,
            end_clause,
        });
        parser(input)
    }
}

fn cmake_foreach_group(input: &str) -> IResult<&str, CMakeStatement> {
    let block = cmake_clause_body_block("foreach");
    map(block, |group| {
        CMakeStatement::For(CMakeForEachStatement { group })
    })(input)
}

fn cmake_function_group(input: &str) -> IResult<&str, CMakeStatement> {
    let function_block = cmake_clause_body_block("function");
    map(function_block, |group| {
        CMakeStatement::Function(CMakeFunctionStatement { group })
    })(input)
}

fn cmake_macro_group(input: &str) -> IResult<&str, CMakeStatement> {
    map(cmake_clause_body_block("macro"), |group| {
        CMakeStatement::Macro(CMakeMacroStatement { group })
    })(input)
}

fn cmake_block_group(input: &str) -> IResult<&str, CMakeStatement> {
    map(cmake_clause_body_block("block"), |group| {
        CMakeStatement::Block(CMakeBlockStatement { group })
    })(input)
}

fn skip_empty_command<'a>(name: &'a str) -> impl Fn(&str) -> IResult<&str, Vec<CMakeValue>> + 'a {
    move |input| {
        let command = tag_no_case(name);
        let parser = tuple((command, space0, cmake_args));
        map(parser, |(_, _, clause)| clause)(input)
    }
}

fn cmake_statement(input: &str) -> IResult<&str, CMakeStatement> {
    alt((
        context("command", cmake_command.map(CMakeStatement::Command)),
        context(
            "comment",
            cmake_comment.map(|item| CMakeStatement::Comment(item.to_string())),
        ),
        context(
            "newline",
            tuple((tag("\n"), space0)).map(|_| CMakeStatement::Newline),
        ),
        context("if", cmake_if_group),
        context("foreach", cmake_foreach_group),
        context("function", cmake_function_group),
        context("macro", cmake_macro_group),
        context("block", cmake_block_group),
    ))(input)
}

pub fn cmake_parser(input: &str) -> IResult<&str, CMakeDocument> {
    let mut parser = parse_separated_terminated(
        cmake_statement,
        space0,
        multispace0.all_consuming(),
        || vec![],
        |mut memo, current| {
            memo.push(current);
            memo
        },
    );
    let (input, statements) = parser.parse(input)?;
    Ok((input, CMakeDocument { statements }))
}

mod parse_condition;
mod strings;
#[cfg(test)]
mod test;
