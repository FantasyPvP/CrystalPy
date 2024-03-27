use std::fmt::Display;
use std::fs;
use std::path::Component;

use python_rs::lexer::Lexer;
use python_rs::parser::Parser;

fn main() {
    println!("Hello, world!");

    let code = fs::read_to_string("./pysrc/srv.py").expect("failed to compile");
    let tokens = Lexer::new().tokens(code).unwrap();
    tokens.iter().for_each(|t| print!("{} ", t));
    println!("\n\n\n\n\n");

    let p = Parser::new();
    p.parse(tokens);
}


