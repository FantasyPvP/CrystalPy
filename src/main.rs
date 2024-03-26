use std::fmt::Display;
use std::fs;
use std::path::Component;

use python_rs::lexer::Lexer;

fn main() {
    println!("Hello, world!");

    let code = fs::read_to_string("./pysrc/srv.py").expect("failed to compile");
    let tokens = Lexer::new().tokens(code).unwrap();
    tokens.iter().for_each(|t| print!("{} ", t));
}



#[derive(Debug, Clone)]
enum Node {
    Integer(i64),
    Float(f64),
    String(String),
    Bool(bool),
    Function(Box<Function>),
    FunctionCall(usize),     // function id 
    Variable(Box<Variable>)
}

#[derive(Debug, Clone)]
struct Function {
    id: usize,
    name: String,
    args: Vec<Node>,
    returntype: Node,
}

#[derive(Debug, Clone)]
struct Variable {
    name: String,
    value: Node,
}

trait Visit {
    fn visit(&self, args: Vec<Node>) -> Node;
}
