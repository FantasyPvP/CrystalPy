use std::fmt::{Display, format};
use crate::{CompileError, ErrorType};

#[derive(Debug, Clone)]
enum LexerState {
    None,
    IdentOrKeyword(String),
    NumberLiteral(String),
    StringLiteral(String),
    MultilineStringLiteral(String),
    CharLiteral(Option<char>),
    Operator(String),
    NewLine(usize),
    Comment,
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
                // println!("{:?} {:?}", self.state, self.tokens.last());
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
                            '=' | '+' | '-' | '*' | '/' | '<' | '>' | '!' | '%' => {
                                self.tok_col = self.col;
                                self.state = LexerState::Operator(c.to_string());
                            },
                            ' ' => {
                                ()
                            },
                            '.' => self.tokens.push(Token::new(&self, TT::Dot)),
                            '\n' => {
                                self.newline();
                                self.tokens.push(Token::new(&self, TT::Newline));
                                self.state = LexerState::NewLine(0)
                            },
                            '#' => {
                                self.state = LexerState::Comment;
                            }
                            ',' => self.tokens.push(Token::new(&self, TT::Comma)),
                            ':' => self.tokens.push(Token::new(&self, TT::Colon)),
                            '(' => self.tokens.push(Token::new(&self, TT::LParen)),
                            ')' => self.tokens.push(Token::new(&self, TT::RParen)),
                            '[' => self.tokens.push(Token::new(&self, TT::LBracket)),
                            ']' => self.tokens.push(Token::new(&self, TT::RBracket)),
                            '{' => self.tokens.push(Token::new(&self, TT::LBrace)),
                            '}' => self.tokens.push(Token::new(&self, TT::RBrace)),
                            '|' => self.tokens.push(Token::new(&self, TT::BitwiseOr)),
                            '&' => self.tokens.push(Token::new(&self, TT::BitwiseAnd)),
                            '^' => self.tokens.push(Token::new(&self, TT::BitwiseXor)),
                            '~' => self.tokens.push(Token::new(&self, TT::BitwiseNot)),
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
                    LexerState::NewLine(x) => {
                        if c == ' ' {
                            self.state = LexerState::NewLine(x + 1)
                        } else {
                            self.tokens.push(Token::new(&self, TT::Whitespace(x)));
                            self.state = LexerState::None;
                            continue 'inner;
                        }
                    }
                    LexerState::CharLiteral(char_) => {
                        if let Some(c) = char_ {
                            if c == '\'' {
                                self.tokens.push(Token::new(&self, TT::CharacterLiteral(c)));
                            } else {
                                return Err(CompileError::new(&self, ErrorType::SyntaxError))
                            }
                        }
                    },
                    LexerState::Comment => {
                        if c == '\n' {
                            self.state = LexerState::None;
                            continue 'inner
                        }
                    }
                    LexerState::StringLiteral(string) => {
                        if string == "\"" && c == '\"' {
                            self.tok_col = self.col - 2;
                            self.state = LexerState::MultilineStringLiteral(String::new());
                        } else if string.ends_with("\"") {
                            let mut s = string;
                            s.pop().unwrap();
                            self.tokens.push(Token::new(&self, TT::StringLiteral(s)));
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
                        match c {
                            '='|'+'|'-'|'*'|'/'|'<'|'>'|'!'|'%' => {
                                self.state = LexerState::Operator(format!("{operator}{c}").to_string());
                            }
                            _ => {
                                match operator.as_str() {
                                    "+" => self.tokens.push(Token::new(&self, TT::OpAdd)),
                                    "+=" => self.tokens.push(Token::new(&self, TT::AssignAdd)),
                                    "-" => self.tokens.push(Token::new(&self, TT::OpSub)),
                                    "-=" => self.tokens.push(Token::new(&self, TT::AssignSub)),
                                    "*" => self.tokens.push(Token::new(&self, TT::OpMul)),
                                    "*=" => self.tokens.push(Token::new(&self, TT::AssignMul)),
                                    "/" => self.tokens.push(Token::new(&self, TT::OpDiv)),
                                    "/=" => self.tokens.push(Token::new(&self, TT::AssignDiv)),
                                    "%" => self.tokens.push(Token::new(&self, TT::OpMod)),
                                    "%=" => self.tokens.push(Token::new(&self, TT::AssignMod)),
                                    "**" => self.tokens.push(Token::new(&self, TT::OpPow)),
                                    "**=" => self.tokens.push(Token::new(&self, TT::AssignPow)),
                                    "//" => self.tokens.push(Token::new(&self, TT::OpFloorDiv)),
                                    "//=" => self.tokens.push(Token::new(&self, TT::AssignFloorDiv)),
                                    "<" => self.tokens.push(Token::new(&self, TT::CompLt)),
                                    ">" => self.tokens.push(Token::new(&self, TT::CompGt)),
                                    "=" => self.tokens.push(Token::new(&self, TT::Assign)),
                                    "==" => self.tokens.push(Token::new(&self, TT::CompEq)),
                                    "!=" => self.tokens.push(Token::new(&self, TT::CompNeq)),
                                    "<=" => self.tokens.push(Token::new(&self, TT::CompLte)),
                                    ">=" => self.tokens.push(Token::new(&self, TT::CompGte)),
                                    _ => return Err(CompileError::new(&self, ErrorType::SyntaxError))
                                }
                                self.state = LexerState::None;
                                continue 'inner;
                            }
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

        let mut res = self.tokens.clone();

        // eliminate lines that contain only newlines and whitespace or comments
        let mut empty= false;
        let mut line = 1;

        for c in self.tokens.iter() {
            if c.line != line {
                if empty {
                    res.retain(|x| x.line != line);
                }
                empty = true;
                line = c.line;
            }
            match c.type_ {
                TT::Whitespace(_) | TT::Newline => {},
                _ => empty = false,
            }
        }
        Ok(res)
    }
}


#[derive(Debug, Clone)]
pub struct Token {
    pub line: usize,
    pub col: usize,
    pub type_: TT,
}

impl Token {
    pub fn new(ctx: &Lexer, type_: TT) -> Token {
        Token { line: ctx.line, col: ctx.tok_col, type_ }
    }
}

impl Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.type_)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum TT {
    // literals
    IntegerLiteral(i64),
    FloatLiteral(f64),
    CharacterLiteral(char),
    StringLiteral(String),
    Whitespace(usize),


    // other
    Identifier(String),
    Keyword(Keyword),

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
    OpFloorDiv,
    AssignFloorDiv,

    // boolean and bitwise operators
    BitwiseOr,
    BitwiseAnd,
    BitwiseNot,
    BitwiseXor,

    BitWiseLeftShift,
    BitWiseRightShift,

    // Comparison operators
    CompEq,
    CompNeq,
    CompGt,
    CompGte,
    CompLt,
    CompLte,

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
    Dot,
    Newline,
}

#[derive(Debug, Clone, PartialEq)]
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

impl Display for Keyword {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", {match self {
            Keyword::Def => "def",
            Keyword::For => "for",
            Keyword::While => "while",
            Keyword::If => "if",
            Keyword::Elif => "elif",
            Keyword::Else => "else",
            Keyword::Import => "import",
            Keyword::Try => "try",
            Keyword::Except => "except",
        }})
    }
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

impl Display for TT {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f, "{}", { match self {
                TT::IntegerLiteral(val) => val.to_string(),
                TT::FloatLiteral(val) => val.to_string(),
                TT::CharacterLiteral(val) => format!("'{val}'").to_string(),
                TT::StringLiteral(val) => format!("\"{val}\"").to_string(),
                TT::Identifier(val) => val.to_string(),
                TT::Keyword(val) => val.to_string(),
                TT::Whitespace(val) => format!("{} {}", val, " ".repeat(*val)),
                TT::Assign => "=".to_string(),
                TT::OpAdd => "+".to_string(),
                TT::AssignAdd => "+=".to_string(),
                TT::OpSub => "-".to_string(),
                TT::AssignSub => "-=".to_string(),
                TT::OpMul => "*".to_string(),
                TT::AssignMul => "*=".to_string(),
                TT::OpDiv => "/".to_string(),
                TT::AssignDiv => "/=".to_string(),
                TT::OpPow => "**".to_string(),
                TT::AssignPow => "**=".to_string(),
                TT::OpMod => "%".to_string(),
                TT::AssignMod => "%=".to_string(),
                TT::OpFloorDiv => "//".to_string(),
                TT::AssignFloorDiv => "//=".to_string(),
                TT::BitwiseOr => "|".to_string(),
                TT::BitwiseAnd => "&".to_string(),
                TT::BitwiseNot => "~".to_string(),
                TT::BitwiseXor => "^".to_string(),
                TT::BitWiseLeftShift => "<<".to_string(),
                TT::BitWiseRightShift => ">>".to_string(),
                TT::CompEq => "==".to_string(),
                TT::CompNeq => "!=".to_string(),
                TT::CompGt => ">".to_string(),
                TT::CompGte => ">=".to_string(),
                TT::CompLt => "<".to_string(),
                TT::CompLte => "<=".to_string(),
                TT::LParen => "(".to_string(),
                TT::RParen => ")".to_string(),
                TT::LBracket => "[".to_string(),
                TT::RBracket => "]".to_string(),
                TT::LBrace => "{".to_string(),
                TT::RBrace => "}".to_string(),
                TT::Colon => ":".to_string(),
                TT::Comma => ",".to_string(),
                TT::Dot => ".".to_string(),
                TT::Newline => " \\n\n".to_string(),
            }}
        )
    }
}

























