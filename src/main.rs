use nom::combinator::all_consuming;

mod parser;
mod pretty_printer;

fn main() {
    let contents = std::fs::read_to_string("../poly-enveloper/cpp/CMakeLists.txt").unwrap();
    let (_, contents) = all_consuming(parser::cmake_parser)(&contents).unwrap();
    eprintln!("{:#?}", contents);
    contents.print().render(80, &mut std::io::stdout()).unwrap();
}
