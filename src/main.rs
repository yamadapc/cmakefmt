use nom::combinator::all_consuming;

mod parser;
mod pretty_printer;

fn main() {
    let target = std::env::args().nth(1);
    if target.is_none() {
        eprintln!("Usage: cmakefmt <target>");
        std::process::exit(1);
    }

    let contents = std::fs::read_to_string(target.unwrap()).expect("Failed to open file");
    let (_, contents) =
        all_consuming(parser::cmake_parser)(&contents).expect("Failed to parse input file");
    contents
        .print()
        .render(80, &mut std::io::stdout())
        .expect("Failed to format file");
}
