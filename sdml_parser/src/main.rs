use chumsky::prelude::*;
use sdml_parser::parser;

fn main() {
    let usage = "Run `sdml_parser <data_model_file.sdml>`";
    let path = std::env::args().nth(1).expect(usage);
    let src = std::fs::read_to_string(&path).expect(&format!("File not found at path {path}"));

    let parse_result = parser::new().parse(&src);
    let parse_errors = parse_result.errors();
    if parse_errors.len() > 0 {
        eprintln!("Parser Errros:");
        for err in parse_errors {
            eprintln!("{err:#?}",)
        }
    } else {
        let output = parse_result.output().unwrap();
        println!("AST {output:#?}")
    }
}
