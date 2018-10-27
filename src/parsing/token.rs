#[derive(PartialEq)]
#[derive(Debug)]
pub enum TokenType {
    Undefined,

    LeftParen,
    RightParen,

    LeftBracket,
    RightBracket,

    LeftBrace,
    RightBrace,

    Word,
    Int,
    Float,
    Symbol
}

#[derive(Debug)]
pub struct Token<'a> {
    pub kind: &'a TokenType,

    pub position: (usize, usize),
    pub span: usize,

    pub value: String
}

impl<'a> Token<'a> {
    pub fn new() -> Token<'a> {
        Token {
            kind: &TokenType::Undefined,

            position: (0, 0),
            span: 0,

            value: String::new()
        }
    }
}

