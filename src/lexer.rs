use std::fmt::Display;
use crate::{CompileError, ErrorType};

#[derive(Debug, Clone)]
enum LexerState {
    None,
    IdentOrKeyword(String),
    NumberLiteral(String),
    StringLiteral(String),
    MultilineStringLiteral(String),
    CharLiteral(Option<char>),
    Operator(char),
}

pub struct Lexer {
    pub line: usize,
    pub col: usize,    // the current col
    pub tok_col: usize, // the col where the token started.
    indent: usize,
    tokens: Vec<Token>,
    state: LexerState,
}

impl Lexer {
    pub fn new() -> Lexer {
        Lexer {
            line: 1,
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

    pub fn tokens(&mut self, code: String) -> Result<Vec<Token>, CompileError> {
        let mut code = code.replace("\t", "    ");

        for c in code.chars() {
            'inner: loop {
                println!("{:?} {:?}", self.state, self.tokens.last());
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
                            '.' => self.tokens.push(Token::new(&self, TT::Dot)),
                            '\n' => {
                                self.newline();
                            },
                            ',' => self.tokens.push(Token::new(&self, TT::Comma)),
                            ':' => self.tokens.push(Token::new(&self, TT::Colon)),
                            '(' => self.tokens.push(Token::new(&self, TT::LParen)),
                            ')' => self.tokens.push(Token::new(&self, TT::RParen)),
                            '[' => self.tokens.push(Token::new(&self, TT::LBracket)),
                            ']' => self.tokens.push(Token::new(&self, TT::RBracket)),
                            '{' => self.tokens.push(Token::new(&self, TT::LBrace)),
                            '}' => self.tokens.push(Token::new(&self, TT::RBrace)),
                            '|' => self.tokens.push(Token::new(&self, TT::BitWiseOr)),
                            '&' => self.tokens.push(Token::new(&self, TT::BitWiseAnd)),
                            '^' => self.tokens.push(Token::new(&self, TT::BitWiseXor)),
                            '~' => self.tokens.push(Token::new(&self, TT::BitWiseNot)),
                            '\"' => {
                                self.tok_col = self.col;
                                self.state = LexerState::StringLiteral(String::new());
                            },
                            '\'' => {
                                self.tok_col = self.col;
                                self.state = LexerState::CharLiteral(None);
                            }
                            _ => {
                                println!("ILLEGAL CHAR {:?}", c);
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
                    },
                    LexerState::StringLiteral(string) => {
                        if string == "\"" && c == '\"' {
                            self.tok_col = self.col - 2;
                            self.state = LexerState::MultilineStringLiteral(String::new());
                        } else if string.ends_with("\"") {
                            self.tokens.push(Token::new(&self, TT::StringLiteral(string)));
                            self.state = LexerState::None;
                            continue 'inner;
                        } else if c == '\n' {
                            return Err(CompileError::new(&self, ErrorType::SyntaxError))
                        } else {
                            self.state = LexerState::StringLiteral(format!("{string}{c}").to_string());
                        }
                    },
                    LexerState::MultilineStringLiteral(string) => {

                        if string.len() <= 3 && string == "\"\"\"" {
                            self.tokens.push(Token::new(&self, TT::StringLiteral(String::new())));
                            self.state = LexerState::None;
                            continue 'inner;
                        } else if string.ends_with("\"\"\"") {
                            let mut s = string;
                            s.pop().unwrap();
                            s.pop().unwrap();
                            s.pop().unwrap();
                            self.tokens.push(Token::new(&self, TT::StringLiteral(s)));
                            self.state = LexerState::None;
                            continue 'inner;
                        } else {
                            if c == '\n' {
                                self.newline();
                            }
                            self.state = LexerState::MultilineStringLiteral(format!("{string}{c}").to_string());
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
                    },
                    LexerState::IdentOrKeyword(val) => {
                        match c {
                            'a'..='z' | 'A'..='Z' | '0'..='9' | '_' => {
                                self.state = LexerState::IdentOrKeyword(format!("{val}{c}").to_string());
                            }
                            _ => {
                                if let Ok(keyword) = Keyword::try_from(val.as_str()) {
                                    self.tokens.push(Token::new(&self, TT::Keyword(keyword)));
                                } else {
                                    self.tokens.push(Token::new(&self, TT::Identifier(val.to_owned())));
                                };
                                self.state = LexerState::None;
                                continue 'inner;
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
                            '\n' | ' ' | '(' | ')' | '[' | ']' | '{' | '}' | ',' | ':' | '=' | '+' | '-' | '*' | '/' | '<' | '>' | '!' => {
                                if val.contains('.') {
                                    self.tokens.push(Token::new(&self, TT::FloatLiteral(val.parse::<f64>().unwrap())));
                                } else {
                                    self.tokens.push(Token::new(&self, TT::IntegerLiteral(val.parse::<i64>().unwrap())));
                                }
                                self.state = LexerState::None;
                                continue 'inner;
                            }
                            _ => {
                                println!("ILLEGAL CHAR 2 {:?}", c);
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


#[derive(Debug, Clone)]
pub struct Token {
    line: usize,
    col: usize,
    type_: TT,
}

impl Token {
    pub fn new(ctx: &Lexer, type_: TT) -> Token {
        Token { line: ctx.line, col: ctx.tok_col, type_ }
    }
}

impl Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<{:?}>", self.type_)
    }
}

#[derive(Debug, Clone)]
pub enum TT {
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

    // boolean and bitwise operators
    OpOr,
    BitwiseOr,
    OpAnd,
    BitwiseAnd,
    OpXor,
    BitwiseNot,
    OpNot,
    BitwiseXor,

    BitWiseLeftShift,
    BitWiseRightShift,

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

    // punctuation
    Colon,
    Comma,
    Dot
}

#[derive(Debug, Clone)]
pub enum Keyword {
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