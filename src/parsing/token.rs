#[derive(Debug, Clone, PartialEq)]
pub enum TokenType {
    Undefined,
    Control(String),

    Word(String),
    Int(i64),
    Float(f64),
    Symbol(String)
}

#[derive(Debug, Clone)]
pub struct Token {
    pub kind: TokenType,

    pub position: (usize, usize),
    pub index: usize,
    pub span: usize,
}

impl Token {
    pub fn empty() -> Token {
        Token {
            kind: TokenType::Undefined,

            position: (0, 0),
            index: 0,
            span: 0,
        }
    }

    pub fn new_undefined(position: (usize, usize), index: usize, span: usize) -> Token {
        Token {
            kind: TokenType::Undefined,

            position,
            index,
            span,
        }
    }

    pub fn new_control(sym: &str, position: (usize, usize), index: usize, span: usize) -> Token {
        Token {
            kind: TokenType::Control(sym.to_owned()),

            position,
            index,
            span,
        }
    }

    pub fn new_word(val: &str, position: (usize, usize), index: usize, span: usize) -> Token {
        Token {
            kind: TokenType::Word(val.to_owned()),

            position,
            index,
            span,
        }
    }

    pub fn new_symbol(sym: &str, position: (usize, usize), index: usize, span: usize) -> Token {
        Token {
            kind: TokenType::Symbol(sym.to_owned()),

            position,
            index,
            span,
        }
    }

    pub fn new_int(val: i64, position: (usize, usize), index: usize, span: usize) -> Token {
        Token {
            kind: TokenType::Int(val),

            position,
            index,
            span,
        }
    }

    pub fn new_float(val: f64, position: (usize, usize), index: usize, span: usize) -> Token {
        Token {
            kind: TokenType::Float(val),

            position,
            index,
            span,
        }
    }
}
