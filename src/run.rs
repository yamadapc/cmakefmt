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

use nom_supreme::final_parser::final_parser;

use crate::options::Options;
use crate::writer::DefaultWriter;
use crate::{errors, parser};

pub fn run_cmakefmt(opts: Options) {
    let Options {
        verbose,
        inplace,
        input_file,
        width,
    } = opts;

    let file_contents = std::fs::read_to_string(&input_file).expect("Failed to open file");
    let mut parser = final_parser(parser::cmake_parser);

    if verbose {
        println!("{file_contents:#?}");
    }
    match parser(&file_contents) {
        Ok(contents) => {
            if verbose {
                println!("{contents:#?}");
            }

            let mut writer = DefaultWriter::new(inplace, input_file.as_str());
            contents
                .print()
                .render(width, &mut writer)
                .expect("Failed to format file");
        }
        Err(err) => {
            errors::print_error(&input_file, file_contents.as_str(), &err);
            std::process::exit(1);
        }
    };
}
