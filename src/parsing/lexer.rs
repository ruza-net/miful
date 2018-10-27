extern crate regex;
extern crate unicode_segmentation;

use self::regex::Regex;

use parsing::token;
use self::token::Token;
use self::token::TokenType;

use self::unicode_segmentation::UnicodeSegmentation;


#[derive(Debug)]
enum LexerState {
    Default,

    Reading(TokenType),
    Building(TokenType),

    Undefined
}

#[derive(Debug)]
pub struct Lexer<'a> {
    tokens: Vec<Token<'a>>,

    position: (usize, usize),
    index: usize,
    span: usize,

    state: LexerState,
    string: Vec<&'a str>,

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

            word: Regex::new(word_rule).unwrap(),
            symbol: Regex::new(symbol_rule).unwrap(),
            float: Regex::new(float_rule).unwrap(),
            int: Regex::new(int_rule).unwrap(),

            ws: Regex::new(r"^[ \t\n\r]").unwrap()
        }
    }

    fn switch_to(&mut self, new_state: LexerState) {
        self.state = new_state;
    }
}
