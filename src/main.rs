use std::fmt::Display;
use std::fs;
use std::path::Component;
use crate::ErrorType::IllegalCharacter;

fn main() {
    println!("Hello, world!");

    let code = fs::read_to_string("./pysrc/main.py").expect("failed to compile");
    println!("code:\n{:?}", code);
    let tokens = Lexer::new().tokens(code).unwrap();
    tokens.iter().for_each(|t| println!("{}", t));
}

#[derive(Debug, Clone)]
enum LexerState {
    None,
    IdentOrKeyword(String),
    NumberLiteral(String),
    StringLiteral(String),
    CharLiteral(Option<char>),
    Operator(char),
}

struct Lexer {
    pub line: usize,
    pub col: usize,    // the current col
    pub tok_col: usize, // the col where the token started.
    indent: usize,
    tokens: Vec<Token>,
    state: LexerState,
}

impl Lexer {
    fn new() -> Lexer {
        Lexer {
            line: 0,
            col: 1,
            tok_col: 0,
            indent: 0,
            tokens: Vec::new(),
            state: LexerState::None,
        }
    }

    fn newline(&mut self) {
        self.line += 1;
        self.col = 1;
    }

    fn tokens(&mut self, code: String) -> Result<Vec<Token>, CompileError> {
        let mut chars = code.chars();

        while let Some(c) = chars.next() {
            'inner: loop {
                println!("{:?}", self.state);
                match self.state.clone() {
                    LexerState::None => {
                        match c {
                            'a'..='z' | 'A'..='Z' | '_' => {
                                self.tok_col = self.col;
                                self.state = LexerState::IdentOrKeyword(c.to_string());
                            },
                            '0'..='9' => {
                                self.tok_col = self.col;
                                self.state = LexerState::NumberLiteral(c.to_string());
                            },
                            '=' | '+' | '-' | '*' | '/' => {
                                self.tok_col = self.col;
                                self.state = LexerState::Operator(c);
                            },
                            ' ' => {
                                ()
                            },
                            '\n' => {
                                self.newline();
                            },
                            ':' => self.tokens.push(Token::new(&self, TT::Colon)),
                            '(' => self.tokens.push(Token::new(&self, TT::LParen)),
                            ')' => self.tokens.push(Token::new(&self, TT::RParen)),
                            '[' => self.tokens.push(Token::new(&self, TT::LBracket)),
                            ']' => self.tokens.push(Token::new(&self, TT::RBracket)),
                            '{' => self.tokens.push(Token::new(&self, TT::LBrace)),
                            '}' => self.tokens.push(Token::new(&self, TT::RBrace)),
                            '\"' => {
                                self.tok_col = self.col;
                                self.state = LexerState::StringLiteral(String::new(), false);
                            },
                            '\'' => {
                                self.tok_col = self.col;
                                self.state = LexerState::CharLiteral(None);
                            }
                            _ => {
                                return Err(CompileError::new(&self, ErrorType::IllegalCharacter))
                            }
                        }
                    },
                    LexerState::CharLiteral(char_) => {
                        if let Some(c) = char_ {
                            if c == '\'' {
                                self.tokens.push(Token::new(&self, TT::CharacterLiteral(c)));
                            } else {
                                return Err(CompileError::new(&self, ErrorType::SyntaxError))
                            }
                        }
                    }
                    LexerState::Operator(operator) => {
                        if c == ' ' || c == '\n' {
                            match operator {
                                '+' => self.tokens.push(Token::new(&self, TT::OpAdd)),
                                '-' => self.tokens.push(Token::new(&self, TT::OpSub)),
                                '*' => self.tokens.push(Token::new(&self, TT::OpMul)),
                                '/' => self.tokens.push(Token::new(&self, TT::OpDiv)),
                                '=' => self.tokens.push(Token::new(&self, TT::Assign)),
                                _ => panic!("This should be unreachable!"),
                            };
                            self.state = LexerState::None;
                            continue 'inner;
                        }
                    }
                    LexerState::IdentOrKeyword(val) => {
                        match c {
                            'a'..='z' | 'A'..='Z' | '0'..='9' | '_' => {
                                self.state = LexerState::IdentOrKeyword(format!("{val}{c}").to_string());
                            }
                            '\n' | ' ' => {
                                if let Ok(keyword) = Keyword::try_from(val.as_str()) {
                                    self.tokens.push(Token::new(&self, TT::Keyword(keyword)));
                                } else {
                                    self.tokens.push(Token::new(&self, TT::Identifier(val.to_owned())));
                                };

                                self.state = LexerState::None;
                                continue 'inner;
                            }
                            _ => {
                                ()
                            }
                        }
                    },
                    LexerState::NumberLiteral(val) => {
                        match c {
                            '0'..='9' => {
                                self.state = LexerState::NumberLiteral(format!("{val}{c}").to_string());
                            },
                            '.' => {
                                if !val.contains('.') {
                                    self.state = LexerState::NumberLiteral(format!("{val}.").to_string());
                                } else {
                                    return Err(CompileError::new(&self, ErrorType::SyntaxError))
                                }
                            },
                            '\n' | ' ' => {
                                if val.contains('.') {
                                    self.tokens.push(Token::new(&self, TT::FloatLiteral(val.parse::<f64>().unwrap())));
                                } else {
                                    self.tokens.push(Token::new(&self, TT::IntegerLiteral(val.parse::<i64>().unwrap())));
                                }
                                self.state = LexerState::None;
                                continue 'inner;
                            }
                            _ => {
                                return Err(CompileError::new(&self, ErrorType::IllegalCharacter))
                            }
                        }
                    }
                    _ => {},
                };
                break; // break out of 'inner' loop - "continue 'inner" will manually re-trigger the loop
            }
            self.col += 1;
        }
        Ok(self.tokens.clone())
    }
}

#[derive(Debug)]
struct CompileError {
    line: usize,
    col: usize,
    type_: ErrorType,
}

impl CompileError {
    fn new(ctx: &Lexer, type_: ErrorType) -> CompileError {
        CompileError { line: ctx.line, col: ctx.tok_col, type_ }
    }
}

#[derive(Debug)]
enum ErrorType {
    SyntaxError,
    IllegalCharacter,
    NameError,
    TypeError,
    Placeholder,
}

#[derive(Debug, Clone)]
struct Token {
    line: usize,
    col: usize,
    type_: TT,
}

impl Token {
    fn new(ctx: &Lexer, type_: TT) -> Token {
        Token { line: ctx.line, col: ctx.tok_col, type_ }
    }
}

impl Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "At: [{}:{}] {:?}", self.line, self.col, self.type_)
    }
}

#[derive(Debug, Clone)]
enum TT {
    // literals
    IntegerLiteral(i64),
    FloatLiteral(f64),
    CharacterLiteral(char),
    StringLiteral(String),

    // other
    Identifier(String),
    Keyword(Keyword),
    LineStart(usize), // for indentation

    // Arithmetic and Assignment operators
    Assign,
    OpAdd,
    AssignAdd,
    OpSub,
    AssignSub,
    OpMul,
    AssignMul,
    OpDiv,
    AssignDiv,
    OpPow,
    AssignPow,
    OpMod,
    AssignMod,
    OpOr,
    OpAnd,
    OpNot,

    // Comparison operators
    Eq,
    Neq,
    Gt,
    Gte,
    Lt,
    Lte,

    // Parenthesis / brackets
    LParen,
    RParen,
    LBracket,
    RBracket,
    LBrace,
    RBrace,

    Colon,
}

#[derive(Debug, Clone)]
enum Keyword {
    Def,
    For,
    While,
    If,
    Elif,
    Else,
    Import,
    Try,
    Except,
}

impl TryFrom<&str> for Keyword {
    type Error = ();
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "def" => Ok(Keyword::Def),
            "for" => Ok(Keyword::For),
            "while" => Ok(Keyword::While),
            "if" => Ok(Keyword::If),
            "elif" => Ok(Keyword::Elif),
            "else" => Ok(Keyword::Else),
            "import" => Ok(Keyword::Import),
            "try" => Ok(Keyword::Try),
            "except" => Ok(Keyword::Except),
            _ => Err(()),
        }
    }
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
