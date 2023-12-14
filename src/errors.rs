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

use std::error::Error;

use colored::{ColoredString, Colorize};
use nom_supreme::error::{BaseErrorKind, ErrorTree, StackContext};
use nom_supreme::final_parser::Location;

fn print_alternative(file_path: &str, input_file: &str, errors: &[ErrorTree<Location>]) {
    eprintln!("tried alternatives:\n");
    for (i, error) in errors.iter().enumerate() {
        eprintln!("======================================================================");
        eprintln!("alternative {}:", i + 1);
        print_error(file_path, input_file, &error);
        eprintln!();
    }
}

fn print_stack(
    file_path: &str,
    input_file: &str,
    base: &ErrorTree<Location>,
    contexts: &[(Location, StackContext<&str>)],
) {
    if contexts.len() > 1 {
        eprintln!(
            "base error for STACK len={}: ===========================",
            contexts.len()
        );
    }
    print_error(file_path, input_file, base);
    for context in contexts.iter() {
        let (location, context) = context;
        match context {
            StackContext::Kind(err) => {
                let message = format!(
                    "{}: {}",
                    "error_kind".bright_yellow(),
                    format!("{:?}", err).red().bold()
                );
                print_message_at_location(file_path, input_file, location, &message);
            }
            StackContext::Context(context) => {
                let message = format!("{}: {}", "context".bright_yellow(), context.cyan().bold());
                print_message_at_location(file_path, input_file, location, &message);
            }
        }
        eprintln!("  ---> stack error: {}:{}", location.line, location.column);
    }
}

fn format_error(error: &BaseErrorKind<&str, Box<dyn Error + Send + Sync>>) -> ColoredString {
    format!("{}", error).bright_yellow()
}

fn print_message_at_location(
    file_path: &str,
    input_file: &str,
    location: &Location,
    message: &str,
) {
    let lines = input_file.lines();
    let start = if location.line > 3 {
        location.line - 3
    } else {
        0
    };
    let lines = lines.enumerate().skip(start).map(|(i, l)| (i + 1, l));
    eprintln!("  ---> {}:{}:{}", file_path, location.line, location.column);
    eprintln!("{}", "      | ".bright_purple());
    for (line_num, line) in lines.take(6) {
        eprintln!(
            "{} {}",
            format!("{:05} |", line_num.to_string()).bright_purple(),
            if line_num == location.line {
                line.bright_red()
            } else {
                line.white()
            }
        );
        if line_num == location.line {
            eprint!("{}", "      | ".bright_purple());
            for _ in 0..location.column - 1 {
                eprint!(" ");
            }
            eprint!("{}", "^".yellow());
            eprint!(" ");
            eprint!("{} ({}:{})", message, location.line, location.column);
            eprint!("\n");
        }
    }
    eprintln!("{}", "      | ".bright_purple());
}

pub fn print_error(file_path: &str, input_file: &str, error_tree: &ErrorTree<Location>) {
    match error_tree {
        ErrorTree::Base { location, kind } => {
            let error_message = format_error(kind);
            let message = format!("{}: {}", "error".yellow(), error_message);
            print_message_at_location(file_path, input_file, location, &message);
        }
        ErrorTree::Stack { base, contexts } => {
            print_stack(file_path, input_file, &*base, contexts);
        }
        ErrorTree::Alt(errors) => {
            print_alternative(file_path, input_file, errors);
        }
    }
}
