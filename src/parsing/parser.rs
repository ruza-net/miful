use parsing::ast as ast;

use parsing::token as tok;
use self::tok::TokenType as tok_type;

use parsing::utils::MifulError;


pub struct Parser {
    index: usize,
    hook_count: usize,

    tokens: Vec<tok::Token>,
}

impl Parser {
    pub fn new(tokens: Vec<tok::Token>) -> Parser {
        Parser {
            index: 0,
            hook_count: 0,

            tokens,
        }
    }

    pub fn new_inner(index: usize, hook_count: usize, tokens: Vec<tok::Token>) -> Parser {
        Parser {
            index,
            hook_count,

            tokens,
        }
    }

    pub fn empty() -> Parser {
        Parser {
            index: 0,
            hook_count: 0,

            tokens: vec![],
        }
    }

    // [AREA] Utilities
    //
    fn step_forward(&mut self) {
        self.index += 1;
    }

    fn get(&self) -> tok::Token {
        self.tokens[self.index].clone()
    }

    fn get_prev(&self) -> tok::Token {
        self.tokens[self.index - 1].clone()
    }

    fn eof(&self) -> bool {
        self.index >= self.tokens.len()
    }
    //
    // [END] Utilities
}


impl Iterator for Parser {
    type Item = Result<ast::NodeWrapper, MifulError>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.eof() {
            None

        } else {
            let mut hooks = Vec::new();

            let token = self.get();
            let mut idx = token.index;
            let mut pos = token.position;

            // [TODO] Unquote hooks indexing.
            //
            match token.kind {
                tok_type::Control(s) => {
                    match s.as_ref() {
                        "(" => {
                            let mut values = vec![];
                            let end_paren = tok_type::Control(")".to_owned());

                            self.step_forward();

                            loop {
                                if self.eof() {
                                    return Some(Err(MifulError::semantic_error("Unterminated list!", idx, pos)));
                                }

                                if end_paren == self.get().kind {
                                    break;
                                }

                                if let Some(result) = self.next() {
                                    match result {
                                        Ok(node) => {
                                            idx = node.index;
                                            pos = node.position;

                                            hooks.extend(node.hooks.clone());
                                            values.push(node);
                                        },

                                        Err(e) => {
                                            let mut new_e = e;

                                            new_e.add_layer_top("..while parsing list");

                                            return Some(Err(new_e));
                                        },
                                    }

                                } else {
                                    return Some(Err(MifulError::semantic_error("Unterminated list!", idx, pos)));
                                }
                            }

                            self.step_forward();

                            Some(Ok(ast::NodeWrapper::new_list(values, hooks, idx, pos)))
                        },

                        "[" => {
                            self.step_forward();

                            if self.eof() {
                                return Some(Err(MifulError::semantic_error("Incomplete invoke!", idx, pos)))
                            }

                            let target = self.get().kind;

                            if let tok_type::Word(f_name) | tok_type::Symbol(f_name) = target {
                                let mut with = vec![];

                                let end_bracket = tok_type::Control("]".to_owned());

                                self.step_forward();

                                loop {
                                    if self.eof() {
                                        return Some(Err(MifulError::semantic_error("Unterminated invoke!", idx, pos)));
                                    }

                                    if end_bracket == self.get().kind {
                                        break;
                                    }

                                    if let Some(result) = self.next() {
                                        match result {
                                            Ok(node) => {
                                                idx = node.index;
                                                pos = node.position;

                                                hooks.extend(node.hooks.clone());
                                                with.push(node);
                                            },

                                            Err(e) => {
                                                let mut new_e = e;

                                                new_e.add_layer_top("..while parsing invoke");

                                                return Some(Err(new_e));
                                            },
                                        }

                                    } else {
                                        return Some(Err(MifulError::semantic_error("Unterminated invoke!", idx, pos)));
                                    }
                                }

                                self.step_forward();

                                Some(Ok(ast::NodeWrapper::new_invoke(f_name, with, hooks, idx, pos)))

                            } else {
                                Some(Err(MifulError::semantic_error("Invalid function name type!", idx, pos)))
                            }
                        },

                        "{" => {
                            self.step_forward();

                            if self.eof() {
                                return Some(Err(MifulError::semantic_error("Incomplete invoke!", idx, pos)))
                            }

                            let target = self.get().kind;

                            if let tok_type::Word(f_name) | tok_type::Symbol(f_name) = target {
                                let mut with = vec![];

                                let end_brace = tok_type::Control("}".to_owned());

                                self.step_forward();

                                loop {
                                    if self.eof() {
                                        return Some(Err(MifulError::semantic_error("Unterminated quote!", idx, pos)));
                                    }

                                    if end_brace == self.get().kind {
                                        break;
                                    }

                                    if let Some(result) = self.next() {
                                        match result {
                                            Ok(node) => {
                                                idx = node.index;
                                                pos = node.position;

                                                hooks.extend(node.hooks.clone());
                                                with.push(node);
                                            },

                                            Err(e) => {
                                                let mut new_e = e;

                                                new_e.add_layer_top("..while parsing quote");

                                                return Some(Err(new_e));
                                            },
                                        }

                                    } else {
                                        return Some(Err(MifulError::semantic_error("Unterminated quote!", idx, pos)));
                                    }
                                }

                                self.step_forward();

                                Some(Ok(ast::NodeWrapper::new_quote(f_name, with, hooks, idx, pos)))

                            } else {
                                Some(Err(MifulError::semantic_error("Invalid function name type!", idx, pos)))
                            }
                        },

                        // [TODO]
                        //
                        "{?" => {
                            self.step_forward();

                            if let Some(result) = self.next() {
                                match result {
                                    Ok(node) => {
                                        idx = node.index;
                                        pos = node.position;

                                        // [TODO] Nested hooks?

                                        self.step_forward();
                                        self.hook_count += 1;

                                        Some(Ok(ast::NodeWrapper::new_hook(self.hook_count - 1, vec![node], idx, pos)))
                                    },

                                    Err(e) => {
                                        let mut new_e = e;

                                        new_e.add_layer_top("..while parsing unquote");

                                        Some(Err(new_e))
                                    },
                                }

                            } else {
                                Some(Err(MifulError::semantic_error("Unterminated unquote!", idx, pos)))
                            }
                        },

                        _ => { Some(Err(MifulError::semantic_error(&format!("Unexpected control token: `{}`!", s), idx, pos))) }
                    }
                },

                tok_type::Word(v) => { self.step_forward(); Some(Ok(ast::NodeWrapper::new_word(v, idx, pos))) },
                tok_type::Symbol(v) => { self.step_forward(); Some(Ok(ast::NodeWrapper::new_symbol(v, idx, pos))) },

                tok_type::Int(v) => { self.step_forward(); Some(Ok(ast::NodeWrapper::new_int(v, idx, pos))) },
                tok_type::Float(v) => { self.step_forward(); Some(Ok(ast::NodeWrapper::new_float(v, idx, pos))) },

                tok_type::Undefined => {
                    Some(Err(MifulError::semantic_error("Unexpected token `Undefined`!", idx, pos)))
                },
            }
        }
    }
}
