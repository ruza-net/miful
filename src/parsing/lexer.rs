extern crate regex;
use self::regex::Regex;

extern crate unicode_segmentation;
use self::unicode_segmentation::UnicodeSegmentation;

use parsing::token::Kind;
use parsing::token::Token;
use parsing::token::Because;
use parsing::token::TokenType;
use parsing::token::ControlToken::Paren;
use parsing::token::ControlToken::Brace;
use parsing::token::ControlToken::Bracket;


#[derive(Debug)]
#[derive(Clone)]
enum LexerState {
    Default,

    Reading(TokenType),
    Building(TokenType),

    Eof(TokenType),

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
    pub fn new(input: &'a str, symbol_list: Vec<&str>)
        -> Lexer<'a> {

        let symbol_seq: String = symbol_list.join("");

        Lexer {
            tokens: Vec::new(),

            position: (0, 0),
            index: 0,
            span: 1,

            state: LexerState::Default,
            string: UnicodeSegmentation::graphemes(input, true).collect::<Vec<&'a str>>(),

            last_built_token_type: TokenType::Undefined(Because::WasNotInitialized),

            word: Regex::new(&format!(r"^[^ \n\r\t\[\]{{}}(){}]+$", symbol_seq)).unwrap(),
            symbol: Regex::new(&format!(r"^[{}]$", symbol_seq)).unwrap(),
            float: Regex::new(r"^[0-9]+\.[0-9]+$").unwrap(),
            int: Regex::new(r"^[0-9]+$").unwrap(),

            ws: Regex::new(r"^[ \t\n\r]").unwrap()
        }
    }


    // [AREA] Position Manipulation
    //
    fn advance(&mut self, new_span: usize) {
        self.index += self.span;
        self.span = new_span;

        self.position.1 = self.index;
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
            match workspan.as_ref() {
                "(" => TokenType::Control(Paren(Kind::Left)),
                ")" => TokenType::Control(Paren(Kind::Right)),

                "[" => TokenType::Control(Bracket(Kind::Left)),
                "]" => TokenType::Control(Bracket(Kind::Right)),

                "{" => TokenType::Control(Brace(Kind::Left)),
                "}" => TokenType::Control(Brace(Kind::Right)),

                _ => TokenType::Undefined(Because::DidNotMatch)
            }
        }
    }

    fn eat_char(&mut self) {
        let reached_eof = self.index + self.span == self.string.len();

        if reached_eof {
            println!("A {:?}", self.get_workspan());
        }

        let new_tt = self.try_match();
        let state = self.state.clone();



        if reached_eof {
            println!("B {:?}", self.get_workspan());
        }

        match state {
            LexerState::Default => {
                match new_tt {
                    TokenType::Undefined(Because::WasWhitespace) => {
                        if self.get_head() == "\n" {
                            self.next_line();
                        }

                        self.advance(0);
                    },

                    TokenType::Undefined(Because::DidNotMatch) => {
                        panic!(format!("Lexer couldn't match ```{}``` at {:?}",
                            self.get_workspan(), self.position));
                    },

                    TokenType::Control(_) => {
                        if reached_eof {
                            self.switch_to(LexerState::Eof(new_tt));

                        } else {
                            self.switch_to(LexerState::Building(new_tt));
                        }
                    },

                    _ => {
                        if reached_eof {
                            self.switch_to(LexerState::Eof(new_tt));

                        } else {
                            self.switch_to(LexerState::Reading(new_tt));
                        }
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

            LexerState::Eof(_) => {
                panic!("Lexer `eat_char` called in an invalid state `Eof`!");
            }
        }
    }

    pub fn read_next_token(&mut self) -> Token {
        let tt;

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

                LexerState::Eof(ref new_tt) => {
                    tt = new_tt.clone();

                    break;
                },

                _ => {
                    self.step_forward();
                }
            }
        }

        Token::new(tt, self.position, self.span, self.get_workspan())
    }

    pub fn read_all_tokens(&mut self) -> &Vec<Token> {
        loop {
            let new_token = self.read_next_token();

            self.tokens.push(new_token);

            match self.state {
                LexerState::Eof(_) => {
                    break;
                }

                _ => {
                    self.switch_to(LexerState::Default);

                    self.advance(1);
                }
            }
        }

        &self.tokens
    }
}
