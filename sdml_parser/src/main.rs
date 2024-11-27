use chumsky::prelude::*;
use sdml_parser::parser;

fn main() {
  let usage = "Run `sdml_parser <data_model_file.sdml>`";
  let path = std::env::args().nth(1).expect(usage);
  let src =
    std::fs::read_to_string(&path).expect(&format!("File not found at path {path}"));

  // Parse the source file.
  let parse_result = parser::delcarations().parse(&src);
  if parse_result.has_errors() {
    eprintln!("Parser errors : {:#?}", parse_result.into_result());
  } else {
    let declarations = parse_result.into_result().unwrap();
    let ast_result = parser::semantic_analysis(declarations);
    match ast_result {
      Err(semantic_errs) => println!("Semantic errors: {:#?}", semantic_errs),
      Ok(ast) => println!("AST: {:#?}", ast),
    }
  }
}
