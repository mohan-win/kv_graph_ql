use sdml_parser::parse;

fn main() {
  let usage = "Run `sdml_parser <data_model_file.sdml>`";
  let path = std::env::args().nth(1).expect(usage);
  let src =
    std::fs::read_to_string(&path).expect(&format!("File not found at path {path}"));

  // Parse the source file.
  let parse_result = parse(&src);
  if parse_result.is_err() {
    eprintln!("Parser errors : {:#?}", parse_result);
  } else {
    println!("AST: {:#?}", parse_result.unwrap());
  }
}
