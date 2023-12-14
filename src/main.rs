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

use clap::{arg, command, Arg, ArgAction};
use colored::{ColoredString, Colorize};
use nom_supreme::error::{BaseErrorKind, ErrorTree, StackContext};
use nom_supreme::final_parser::{final_parser, Location};

mod parser;
mod pretty_printer;

fn print_alternative(input_file: &str, errors: &[ErrorTree<Location>]) {
    println!("tried alternatives:\n");
    for (i, error) in errors.iter().enumerate() {
        println!("======================================================================");
        println!("alternative {}:", i + 1);
        print_error(input_file, &error);
        println!();
    }
}

fn print_stack(
    input_file: &str,
    base: &ErrorTree<Location>,
    contexts: &[(Location, StackContext<&str>)],
) {
    if contexts.len() > 1 {
        println!(
            "base error for STACK len={}: ===========================",
            contexts.len()
        );
    }
    for context in contexts.iter() {
        let (location, context) = context;
        match context {
            StackContext::Kind(err) => {
                println!("  ---> error_kind: {:?}", err);
            }
            StackContext::Context(context) => {
                println!(
                    "{}: {}",
                    format!("  ---> context").bright_white(),
                    context.bright_cyan().bold()
                );
            }
        }
        println!("  ---> stack error: {}:{}", location.line, location.column);
    }
    print_error(input_file, base);
}

fn format_error(error: &BaseErrorKind<&str, Box<dyn Error + Send + Sync>>) -> ColoredString {
    format!("{}", error).bright_yellow()
}

fn print_base(
    input_file: &str,
    location: &Location,
    error: &BaseErrorKind<&str, Box<dyn Error + Send + Sync>>,
) {
    let lines = input_file.lines();
    let start = location.line - 3;
    let lines = lines.enumerate().skip(start).map(|(i, l)| (i + 1, l));
    for (line_num, line) in lines.take(6) {
        println!(
            "{} {}",
            format!("{:05} |", line_num.to_string()).bright_purple(),
            if line_num == location.line {
                line.bright_red()
            } else {
                line.white()
            }
        );
        if line_num == location.line {
            print!("{}", "      | ".bright_purple());
            for _ in 0..location.column - 1 {
                print!(" ");
            }
            print!("{}", "^".yellow());
            print!(" ");
            print!(
                "{}: {} ({}:{})",
                "error".yellow(),
                format_error(error),
                location.line,
                location.column
            );
            print!("\n");
        }
    }
}

fn print_error(input_file: &str, error_tree: &ErrorTree<Location>) {
    match error_tree {
        ErrorTree::Base { location, kind } => {
            print_base(input_file, location, kind);
        }
        ErrorTree::Stack { base, contexts } => {
            print_stack(input_file, &*base, contexts);
        }
        ErrorTree::Alt(errors) => {
            print_alternative(input_file, errors);
        }
    }
}

fn main() {
    let matches = command!() // requires `cargo` feature
        .arg(
            Arg::new("inplace")
                .short('i')
                .long("in-place")
                .help("Write to the input file after formatting")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("max-width")
                .long("max-width")
                .num_args(1)
                .default_value("80")
                .help("The column limit to be used"),
        )
        .arg(arg!([file] "Target file").required(true))
        .get_matches();

    let input_file: &String = matches.get_one("file").expect("No input file provided");
    let file_contents = std::fs::read_to_string(input_file).expect("Failed to open file");
    let mut parser = final_parser(parser::cmake_parser);
    match parser(&file_contents) {
        Ok(contents) => {
            let mut writer: Box<dyn std::io::Write> = if matches.get_flag("inplace") {
                Box::new(std::fs::File::create(input_file).expect("Failed to open file"))
            } else {
                Box::new(std::io::stdout())
            };
            let width = matches
                .get_one("max-width")
                .cloned()
                .map(|s: String| s.parse::<usize>().ok())
                .flatten()
                .unwrap_or(80);
            contents
                .print()
                .render(width, &mut writer)
                .expect("Failed to format file");
        }
        Err(err) => {
            print_error(file_contents.as_str(), &err);
            std::process::exit(1);
        }
    };
}

#[cfg(test)]
mod test {
    use std::collections::HashMap;
    use std::fs::DirEntry;

    use nom::combinator::all_consuming;

    #[test]
    fn test_smoke_tests() {
        let input_output: Vec<&str> = vec![
            r#"
cmake_minimum_required(VERSION 3.26)
project(pyramid_envelope VERSION 0.0.1 LANGUAGES CXX)
add_subdirectory(
  ${CMAKE_CURRENT_LIST_DIR}/vendor/juce ${CMAKE_BINARY_DIR}/juce
  EXCLUDE_FROM_ALL
  # don't build examples etc, also don't install
)
            "#,
        ]
        .iter()
        .map(|s| s.trim())
        .collect();

        for input in input_output.iter() {
            let output = all_consuming(super::parser::cmake_parser)(input);
            let output = output.unwrap().1.print();
            let mut writer = vec![];
            {
                output.render(80, &mut writer).unwrap();
            }
            let output = String::from_utf8(writer).unwrap();
            assert_eq!(input, &output.trim());
        }
    }

    #[test]
    fn test_snapshots() {
        struct Group {
            input_file: String,
            output_file: String,
        }

        // read samples directory relative to cargo manifest directory
        let cargo_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
        let files = std::fs::read_dir(format!("{}/samples", cargo_dir)).unwrap();
        let all_files: Vec<DirEntry> = files.filter_map(|f| f.ok()).collect();
        let mut groups: HashMap<String, Group> = HashMap::new();
        all_files
            .iter()
            .map(|f| f.file_name().to_str().unwrap().to_string())
            .for_each(|f| {
                let mut split = f.split(".");
                let base = split.next().unwrap();
                let is_input = split.next().unwrap() == "input";
                let entry = groups.entry(base.to_string()).or_insert(Group {
                    input_file: "".to_string(),
                    output_file: "".to_string(),
                });
                let content = std::fs::read_to_string(format!("{}/samples/{}", cargo_dir, f))
                    .unwrap()
                    .trim()
                    .to_string();
                if is_input {
                    entry.input_file = content
                } else {
                    entry.output_file = content;
                }
            });

        for (
            _label,
            Group {
                input_file,
                output_file,
            },
        ) in groups.iter()
        {
            let output = all_consuming(super::parser::cmake_parser)(&input_file);
            let output = output.unwrap().1.print();
            let mut writer = vec![];
            {
                output.render(80, &mut writer).unwrap();
            }
            let output = String::from_utf8(writer).unwrap();
            assert_eq!(&output, output_file);
        }
    }
}
