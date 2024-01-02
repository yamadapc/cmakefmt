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

use clap::{arg, command, Arg, ArgAction};

pub struct Options {
    pub verbose: bool,
    pub inplace: bool,
    pub input_file: String,
    pub width: usize,
}

pub fn parse_options() -> Options {
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
        .arg(
            Arg::new("verbose")
                .long("verbose")
                .help("Print debug logs")
                .action(ArgAction::SetTrue),
        )
        .arg(arg!([file] "Target file").required(true))
        .get_matches();

    let verbose = matches.get_flag("verbose");
    let inplace = matches.get_flag("inplace");
    let input_file: &String = matches.get_one("file").expect("No input file provided");
    let width = matches
        .get_one("max-width")
        .cloned()
        .map(|s: String| s.parse::<usize>().ok())
        .flatten()
        .unwrap_or(80);
    Options {
        verbose,
        inplace,
        input_file: input_file.clone(),
        width,
    }
}
