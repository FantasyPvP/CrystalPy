
pub mod lexer;

#[derive(Debug)]
pub struct CompileError {
    line: usize,
    col: usize,
    type_: ErrorType,
}

impl CompileError {
    pub fn new(ctx: &lexer::Lexer, type_: ErrorType) -> CompileError {
        CompileError { line: ctx.line, col: ctx.tok_col, type_ }
    }
}


#[derive(Debug)]
pub enum ErrorType {
    SyntaxError,
    IllegalCharacter,
    NameError,
    TypeError,
    Placeholder,
}
