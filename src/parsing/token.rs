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
pub enum Kind {
    Left,
    Right,
}

#[derive(Debug)]
#[derive(Clone)]
#[derive(PartialEq)]
pub enum ControlToken {
    Paren(Kind),
    Bracket(Kind),
    Brace(Kind)
}

#[derive(Debug)]
#[derive(Clone)]
#[derive(PartialEq)]
pub enum TokenType {
    Undefined(Because),
    Control(ControlToken),

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
