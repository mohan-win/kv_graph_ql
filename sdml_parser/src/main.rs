use chumsky::prelude::*;
use sdml_parser::parser;

fn main() {
    let usage = "Run `sdml_parser <data_model_file.sdml>`";
    let path = std::env::args().nth(1).expect(usage);
    let src = std::fs::read_to_string(&path).expect(&format!("File not found at path {path}"));

    let ast = parser::new().parse(&src).into_result();
    match ast {
        Ok(ast) => println!("AST {:#?}", ast),
        Err(e) => eprintln!("Parser error {:#?}", e),
    }
}
