use parsing::ast as ast;
use parsing::token as tok;


enum ParserContext {
    Default,

    InsideInvoke,
    InsideList(tok::TokenType)
}

struct Parser {
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

    fn get(&self, index: usize) -> tok::Token {
        self.tokens[index].clone()
    }

    // Parses a single value
    //
    fn parse_single_value(&mut self, index: usize) -> (ast::NodeWrapper, usize) {
        let current = self.get(index);

        match current.kind {
            tok::TokenType::LeftParen => {
                self.parse(index + 1, ParserContext::InsideList(tok::TokenType::RightParen))
            },

            tok::TokenType::LeftBracket => {
                self.parse(index + 1, ParserContext::InsideInvoke)
            },

            tok::TokenType::Word => {
                (ast::NodeWrapper::new_word(current.value, current.position), 1)
            },

            tok::TokenType::Symbol => {
                (ast::NodeWrapper::new_symbol(current.value, current.position), 1)
            },

            tok::TokenType::Float => {
                (ast::NodeWrapper::new_float(&current.value, current.position), 1)
            },

            tok::TokenType::Int => {
                (ast::NodeWrapper::new_int(&current.value, current.position), 1)
            },

            _ => {
                panic!(format!("Invalid node ```{:?}``` at {:?}!", current.kind, current.position));
            }
        }
    }

    // Parses a sequence of tokens closed by `end_token`
    //
    fn parse_list(&mut self, end_token: tok::TokenType, index: usize) -> (ast::NodeWrapper, usize) {
        let mut elements = Vec::<ast::NodeWrapper>::new();
        let mut span = 0;

        loop {
            if self.get(index).kind == end_token {
                break;
            }

            let (elem, len) = self.parse(index + span, ParserContext::Default);

            elements.push(elem);

            span += len;
        }

        (ast::NodeWrapper::new_list(elements, self.get(index).position), span)
    }

    // Parses the invoke structure
    //
    fn parse_invoke(&mut self, index: usize) -> (ast::NodeWrapper, usize) {
        let target = self.get(index);

        let (lst, len) =
            self.parse(index + 1, ParserContext::InsideList(tok::TokenType::RightBrace));

        let with =
            match lst.node {
                ast::Node::List(l) => Some(l),
                _ => None
            }.unwrap();

        (ast::NodeWrapper::new_invoke(target.value, with, target.position), len)
    }

    // General parsing method
    //
    fn parse(&mut self, index: usize, context: ParserContext) -> (ast::NodeWrapper, usize) {
        match context {
            ParserContext::Default => {
                self.parse_single_value(index)
            },

            ParserContext::InsideInvoke => {
                self.parse_invoke(index)
            },

            ParserContext::InsideList(expect) => {
                self.parse_list(expect, index)
            }
        }
    }

    pub fn construct_tree(&mut self) -> &Vec<ast::NodeWrapper> {
        let mut index = 0;

        loop {
            let (value, len) = self.parse(index, ParserContext::Default);

            self.nodes.push(value);

            index += len;

            if index == self.tokens.len() {
                break;
            }
        }

        &self.nodes
    }
}
