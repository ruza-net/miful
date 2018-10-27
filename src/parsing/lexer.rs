extern crate regex;
extern crate unicode_segmentation;

use self::regex::Regex;

use parsing::token;
use self::token::Token;
use self::token::Because;
use self::token::TokenType;

use self::unicode_segmentation::UnicodeSegmentation;


#[derive(Debug)]
#[derive(Clone)]
enum LexerState {
    Default,

    Reading(TokenType),
    Building(TokenType),

    Eof,

    Undefined
}

#[derive(Debug)]
pub struct Lexer<'a> {
    tokens: Vec<Token>,

    position: (usize, usize),
    index: usize,
    span: usize,

    state: LexerState,
    string: Vec<&'a str>,

    last_built_token_type: TokenType,

    word: Regex,
    symbol: Regex,
    float: Regex,
    int: Regex,

    ws: Regex
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a str, word_rule: &'a str,
            symbol_rule: &'a str, float_rule: &'a str, int_rule: &'a str) -> Lexer<'a> {

        Lexer {
            tokens: Vec::new(),

            position: (0, 0),
            index: 0,
            span: 1,

            state: LexerState::Default,
            string: UnicodeSegmentation::graphemes(input, true).collect::<Vec<&'a str>>(),

            last_built_token_type: TokenType::Undefined(Because::WasNotInitialized),

            word: Regex::new(word_rule).unwrap(),
            symbol: Regex::new(symbol_rule).unwrap(),
            float: Regex::new(float_rule).unwrap(),
            int: Regex::new(int_rule).unwrap(),

            ws: Regex::new(r"^[ \t\n\r]").unwrap()
        }
    }


    // [AREA] Position Manipulation
    //
    fn advance(&mut self) {
        self.index += self.span;
        self.span = 1;
    }

    fn next_line(&mut self) {
        self.position = (self.position.0 + 1, 0);
    }

    fn step_forward(&mut self) {
        self.span += 1;
    }

    fn step_back(&mut self) {
        self.span -= 1;
    }
    //
    // [END] Position Manipulation

    fn switch_to(&mut self, new_state: LexerState) {
        self.state = new_state;
    }

    // [AREA] String Utilities
    //
    fn get_workspan(&self) -> String {
        self.string[self.index .. self.index + self.span].concat()
    }

    fn get_head(&self) -> &str {
        self.string[self.index]
    }
    //
    // [END] String Utilities

    fn try_match(&self) -> TokenType {
        let workspan = &self.get_workspan();

        if self.word.is_match(workspan) {
            TokenType::Word

        } else if self.symbol.is_match(workspan) {
            TokenType::Symbol

        } else if self.float.is_match(workspan) {
            TokenType::Float

        } else if self.int.is_match(workspan) {
            TokenType::Int

        } else if self.ws.is_match(workspan) {
            TokenType::Undefined(Because::WasWhitespace)

        } else {
            TokenType::Undefined(Because::DidNotMatch)
        }
    }

    fn eat_char(&mut self) {
        if self.index == self.string.len() {
            self.switch_to(LexerState::Eof);

        } else {
            let new_tt = self.try_match();
            let state = self.state.clone();

            match state {
                LexerState::Default => {
                    match new_tt {
                        TokenType::Undefined(Because::WasWhitespace) => {
                            if self.get_head() == "\n" {
                                self.next_line();
                            }

                            self.advance();
                        },

                        TokenType::Undefined(Because::DidNotMatch) => {
                            panic!(format!("Lexer couldn't match ```{}``` at {:?}",
                                self.get_workspan(), self.position));
                        },

                        _ => {
                            self.switch_to(LexerState::Reading(new_tt));
                        }
                    }
                },

                LexerState::Reading(tt) => {
                    match new_tt {
                        TokenType::Undefined(_) => {
                            self.step_back();

                            self.switch_to(LexerState::Building(tt));
                        },

                        _ => {}
                    }
                }

                LexerState::Building(_) => {
                    panic!("Lexer `eat_char` was called in an invalid state `Building(_)`!");
                },

                LexerState::Undefined => {
                    panic!("Lexer `eat_char` was called in an invalid state `Undefined`!");
                },

                LexerState::Eof => {
                    panic!("Lexer `eat_char` called in an invalid state `Eof`!");
                }
            }
        }
    }

    pub fn read_next_token(&mut self) -> Option<Token> {
        let tt;
        let mut reached_eof = false;

        loop {
            self.eat_char();

            match self.state {
                LexerState::Building(ref built_tt) => {
                    tt = built_tt.clone();

                    break;
                },

                LexerState::Undefined => {
                    panic!("Lexer reached the state `Undefined` while reading token!");
                },

                LexerState::Eof => {
                    tt = TokenType::Undefined(Because::WasNotInitialized);

                    reached_eof = true;

                    break;
                },

                _ => {
                    self.step_forward();
                }
            }
        }

        if reached_eof {
            None

        } else {
            let new_token = Token::new(tt, self.position, self.span, self.get_workspan());

            Some(new_token)
        }
    }

    pub fn read_all_tokens(&mut self) -> &Vec<Token> {
        loop {
            let new_token = self.read_next_token();

            match new_token {
                Some(tok) => {
                    self.tokens.push(tok);
                },

                None => {
                    break;
                }
            }
        }

        &self.tokens
    }
}
