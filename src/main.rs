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
use nom::combinator::all_consuming;
use nom::error::convert_error;

mod parser;
mod pretty_printer;

fn main() {
    let target = std::env::args().nth(1);
    if target.is_none() {
        eprintln!("Usage: cmakefmt <target>");
        std::process::exit(1);
    }

    let file_contents = std::fs::read_to_string(target.unwrap()).expect("Failed to open file");
    match all_consuming(parser::cmake_parser)(&file_contents) {
        Ok((_, contents)) => {
            contents
                .print()
                .render(80, &mut std::io::stdout())
                .expect("Failed to format file");
        }
        Err(nom::Err::Error(err)) => {
            eprintln!("Failed to parse file");
            eprintln!("{}", convert_error(file_contents.as_str(), err));
            std::process::exit(1);
        }
        Err(nom::Err::Failure(err)) => {
            eprintln!("Failed to parse file");
            eprintln!("{}", convert_error(file_contents.as_str(), err));
            std::process::exit(1);
        }
        Err(nom::Err::Incomplete(err)) => {
            eprintln!("EOF");
            eprintln!("{:?}", err);
            std::process::exit(1);
        }
    };
}

#[cfg(test)]
mod test {
    use std::collections::HashMap;
    use std::fs::DirEntry;

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
            let output = super::all_consuming(super::parser::cmake_parser)(input);
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
            let output = super::all_consuming(super::parser::cmake_parser)(&input_file);
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
