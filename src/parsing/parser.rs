use parsing::ast as ast;

use parsing::token as tok;
use self::tok::TokenType as tok_type;

use self::tok::Kind;
use self::tok::ControlToken::Paren;
use self::tok::ControlToken::Brace;
use self::tok::ControlToken::Bracket;


fn print_depth(depth: usize) {
    for _ in 0..depth {
        print!("|");
    }
}

pub struct Parser {
    tokens: Vec<tok::Token>,
    nodes: Vec<ast::NodeWrapper>
}

impl Parser {
    pub fn new(tokens: Vec<tok::Token>) -> Parser {
        Parser {
            tokens,

            nodes: Vec::new()
        }
    }

    pub fn empty() -> Parser {
        Parser {
            tokens: Vec::new(),
            nodes: Vec::new()
        }
    }

    fn get(&self, index: usize) -> tok::Token {
        self.tokens[index].clone()
    }

    // Parses a single value
    //
    fn parse_single_value(&mut self, index: usize, depth: usize) -> (ast::NodeWrapper, usize) {
        let current = self.get(index);

        match current.kind {
            tok_type::Control(Paren(Kind::Left)) => {
                self.parse_list(index + 1, tok_type::Control(Paren(Kind::Right)), depth)
            },

            tok_type::Control(Bracket(Kind::Left)) => {
                self.parse_invoke(index + 1, depth)
            },

            tok_type::Control(Brace(Kind::Left)) => {
                self.parse_block(index + 1, depth)
            },

            tok::TokenType::Word => {
                (ast::NodeWrapper::new_word(current.value, current.position), index + 1)
            },

            tok::TokenType::Symbol => {
                (ast::NodeWrapper::new_symbol(current.value, current.position), index + 1)
            },

            tok::TokenType::Float => {
                (ast::NodeWrapper::new_float(&current.value, current.position), index + 1)
            },

            tok::TokenType::Int => {
                (ast::NodeWrapper::new_int(&current.value, current.position), index + 1)
            },

            _ => {
                panic!(format!("Invalid node ```{:?}``` at {:?}!", current.kind, current.position));
            }
        }
    }

    // Parses a sequence of tokens closed by `end_token`
    //
    fn parse_list(&mut self, mut index: usize, end_token: tok::TokenType, depth: usize) -> (ast::NodeWrapper, usize) {
        let mut elements = Vec::<ast::NodeWrapper>::new();

        loop {
            if index == self.tokens.len() {
                panic!(format!("Interminated list at {:?}, expecting {:?}!", self.get(index - 1).position, end_token));

            } else if self.get(index).kind == end_token {
                break;
            }

            let (elem, new_index) = self.parse_single_value(index, depth + 1);

            elements.push(elem);

            index = new_index;
        }

        (ast::NodeWrapper::new_list(elements, self.get(index).position), index + 1)
    }

    // Parses the invoke structure
    //
    fn parse_invoke(&mut self, index: usize, depth: usize) -> (ast::NodeWrapper, usize) {
        let target = self.get(index);

        let (lst, new_index) =
            self.parse_list(index + 1, tok_type::Control(Bracket(Kind::Right)), depth);

        let with =
            match lst.node {
                ast::Node::List(l) => Some(l),
                _ => None
            }.unwrap();

        (ast::NodeWrapper::new_invoke(target.value, with, target.position), new_index)
    }

    // Parses a block
    //
    fn parse_block(&mut self, index: usize, depth: usize) -> (ast::NodeWrapper, usize) {
        let target = self.get(index);

        let (lst, new_index) =
            self.parse_list(index, tok_type::Control(Brace(Kind::Right)), depth);

        let invokes =
            match lst.node {
                ast::Node::List(l) => Some(l),
                _ => None
            }.unwrap();

        (ast::NodeWrapper::new_block(invokes, target.position), new_index)
    }


    pub fn construct_tree(&mut self) -> &Vec<ast::NodeWrapper> {
        let mut index = 0;

        loop {
            let (value, new_index) = self.parse_single_value(index, 0);

            self.nodes.push(value);

            index = new_index;

            if index == self.tokens.len() {
                break;
            }
        }

        &self.nodes
    }
}
