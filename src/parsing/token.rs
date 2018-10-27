#[derive(Debug)]
#[derive(Clone)]
#[derive(PartialEq)]
pub enum Because {
    DidNotMatch,
    WasWhitespace,
    WasNotInitialized
}

#[derive(Debug)]
#[derive(Clone)]
#[derive(PartialEq)]
pub enum TokenType {
    Undefined(Because),

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
#[derive(Clone)]
pub struct Token {
    pub kind: TokenType,

    pub position: (usize, usize),
    pub span: usize,

    pub value: String
}

impl Token {
    pub fn empty() -> Token {
        Token {
            kind: TokenType::Undefined(Because::WasNotInitialized),

            position: (0, 0),
            span: 0,

            value: String::new()
        }
    }

    pub fn new(kind: TokenType, position: (usize, usize), span: usize, value: String) -> Token {
        Token {
            kind,

            position,
            span,

            value
        }
    }
}
