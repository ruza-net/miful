extern crate regex;
extern crate unicode_segmentation;

use self::regex::Regex;

use parsing::token;
use self::token::TokenType;

use self::unicode_segmentation::UnicodeSegmentation;


#[derive(Debug)]
enum LexerState<'a> {
    Default,

    Build(&'a TokenType),
    Reading(&'a TokenType),

    Undefined
}

#[derive(Debug)]
pub struct Lexer<'a> {
    tokens: Vec<&'a token::Token<'a>>,

    position: (usize, usize),
    index: usize,
    span: usize,

    state: LexerState<'a>,
    string: Vec<&'a str>,

    last_token: token::Token<'a>,
    last_token_type: TokenType,

    word: Regex,
    symbol: Regex,
    float: Regex,
    int: Regex,

    ws: Regex
}

impl<'a> Lexer<'a> {
    pub fn new(input_string: &'a str, word_def: &str, symbol_def: &str, float_def: &str, int_def: &str) -> Lexer<'a> {
        Lexer {
            tokens: Vec::new(),

            position: (0, 0),
            index: 0,
            span: 1,

            state: LexerState::Default,
            string: UnicodeSegmentation::graphemes(input_string, true).collect::<Vec<&'a str>>(),

            last_token: token::Token::new(),
            last_token_type: TokenType::Undefined,

            word: Regex::new(word_def).unwrap(),
            symbol: Regex::new(symbol_def).unwrap(),
            float: Regex::new(float_def).unwrap(),
            int: Regex::new(int_def).unwrap(),

            ws: Regex::new(r"^[ \t\n\r]").unwrap()
        }
    }

    fn workspan(&self) -> String {
        self.string[self.index .. self.index + self.span].concat()
    }

    fn head(&self) -> &str {
        self.string[self.index]
    }

    fn update_token_type(&self) {
        let workspan = &self.workspan();

        self.last_token_type =
            if self.word.is_match(workspan) {
                TokenType::Word

            } else if self.symbol.is_match(workspan) {
                TokenType::Symbol

            } else if self.float.is_match(workspan) {
                TokenType::Float

            } else if self.int.is_match(workspan) {
                TokenType::Int

            } else {
                TokenType::Undefined
            };
    }

    pub fn eat_char(&mut self) {
        self.span += 1;

        let mut new_state: LexerState<'a> = LexerState::Undefined;

        match self.state {
            LexerState::Default => {
                if self.ws.is_match(&self.workspan()) {
                    self.index += self.span;
                    self.span = 1;

                    if self.head() == "\n" {
                        self.position = (self.position.0 + 1, 0);
                    }

                } else {
                    self.update_token_type();

                    match self.last_token_type {
                        TokenType::Undefined => {
                            panic!(format!("Undefined character `{}` at ({}, {})",
                                self.head(), self.position.0, self.span));
                        },

                        _ => {}
                    }

                    new_state = LexerState::Reading(&self.last_token_type);
                }
            },

            LexerState::Reading(ref tt) => {
                self.update_token_type();

                match self.last_token_type {
                    TokenType::Undefined => {
                        self.span -= 1;

                        new_state = LexerState::Build(tt);
                    },

                    _ => {}
                }
            }

            LexerState::Build(_) => { panic!("`eat_char` called in invalid state `Build`!"); },
            LexerState::Undefined => { panic!("`eat_char` called in invalid state `Undefined`!"); }
        }

        self.state = new_state;
    }

    pub fn lex_next(&mut self) -> &'a token::Token {
        let kind: &'a token::TokenType;

        loop {
            self.eat_char();

            match self.state {
                LexerState::Build(ref tt) => {
                    kind = tt;

                    break;
                },

                _ => ()
            }
        }

        let value = self.workspan();

        self.last_token = token::Token{ kind: kind, position: self.position, span: self.span, value: value };

        &self.last_token
    }

    pub fn lex_whole(&self) -> &'a Vec<&'a token::Token> {
        &self.tokens  // TODO Read the actual string.
    }
}
