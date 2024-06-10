use chumsky::prelude::*;
use sdml_parser::parser;

fn main() {
    let usage = "Run `sdml_parser <data_model_file.sdml>`";
    let src = std::fs::read_to_string(std::env::args().nth(1).expect(usage)).expect(usage);

    let ast = parser::new_parser().parse(&src).into_result();
    match ast {
        Ok(ast) => println!("AST {:#?}", ast),
        Err(e) => eprintln!("Parser error {:?}", e),
    }
}
