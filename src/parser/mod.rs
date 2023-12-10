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

fn cmake_value(input: &str) -> IResult<&str, CMakeValue> {
    let (input, result) = context(
        "Value",
        alt((
            cmake_comment.map(|item| CMakeValue::Comment(item.to_string())),
            cmake_quoted_string_literal,
            cmake_string_literal,
        )),
    )(input)?;
    Ok((input, result))
}

fn cmake_command(input: &str) -> IResult<&str, CMakeCommand> {
    let (input, name) = cmake_command_name(input)?;
    if name == "elseif" || name == "endif" || name == "if" || name == "else" {
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
mod test;
