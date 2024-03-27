use crate::CompileError;
use crate::ErrorType::Placeholder;
use crate::lexer::{Token, TT};

pub struct Parser {
    tokens: Vec<Token>,
}

impl Parser {
    pub fn new() -> Parser {
        Parser {
            tokens: Vec::new(),
        }
    }

    fn lines(&self, tokens: &Vec<Token>) -> Vec<Vec<Token>> {
        let mut lines: Vec<Vec<Token>> = Vec::new();
        let mut line: Vec<Token> = Vec::new();
        for token in tokens {
            if token.type_ == TT::Newline {
                lines.push(line);
                line = Vec::new();
            } else {
                line.push(token.to_owned());
            }
        }
        lines
    }

    pub fn parse(&self, tokens: Vec<Token>) -> Node {
        Node::Integer(2)
    }

    fn scopes(&mut self, lines: &Vec<Vec<Token>>, idx: usize, indent: usize) -> Result<Vec<Node>, CompileError> {
        let current = lines[idx].clone();
        if let TT::Whitespace(x) = current[0].type_ {
            if x != indent {
                return Err(CompileError::Placeholder);
            }
        }

        Ok(Vec::new())
    }
}





#[derive(Debug, Clone)]
pub enum Node {
    Scope(
        usize,      // UID
        Box<Node>,  // header (the line before the first statement)
        Vec<Node>   // Statements
    ),
    Integer(i64),
//     Float(f64),
//     String(String),
//     Bool(bool),
//     Function(
//         usize,    // UID
//         String,   // Name
//         Vec<Node> // Args
//     ),
//     BinaryOperation(Box<Node>, Operator, Box<Node>),
//     UnaryOperation(Operator, Box<Node>),
//     FunctionCall(usize),     // function id
//     Variable(Box<Variable>)
}

pub enum Operator {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Pow,
    FloorDiv
}

#[derive(Debug, Clone)]
pub struct Function {
    id: usize,
    name: String,
    args: Vec<Node>,
    returntype: Node,
}

#[derive(Debug, Clone)]
pub struct Variable {
    name: String,
    value: Node,
}

pub trait Visit {
    fn visit(&self, args: Vec<Node>) -> Node;
}
