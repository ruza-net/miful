use std::collections::HashSet;
use parsing::token::Token;


#[derive(Debug)]
pub struct Lexer<'outer, 'inner> {
    tokens: Vec<Token>,

    position: (usize, usize),
    index: usize,
    span: usize,

    string: Vec<&'outer str>,
    work_string: &'outer str,

    special_chars: HashSet<&'inner str>,

    symbols: HashSet<&'inner str>,
    number: HashSet<&'inner str>,

    ws: HashSet<&'inner str>,

    keep_ws: bool,
}

impl<'outer, 'inner> Lexer<'outer, 'inner> {
    pub fn new(input: Vec<&'outer str>, symbols: HashSet<&'inner str>) -> Lexer<'outer, 'inner> {
        let mut fused = vec!["[", "]", "{", "}", "{?", "?}", "(", ")", " ", "\n", "\t", "\r"];
        fused.extend(symbols.iter().cloned());

        let special_chars: HashSet<&'inner str> = fused.iter().map(|x| &**x).collect();

        Lexer {
            tokens: vec![],

            position: (1, 1),
            index: 0,
            span: 1,

            special_chars,

            work_string: "",
            string: input,

            symbols,
            number: set!["1", "2", "3", "4", "5", "6", "7", "8", "9", "0"],

            ws: set![" ", "\n", "\t", "\r"],
            keep_ws: false,
        }
    }

    pub fn new_ws_preserving(input: Vec<&'outer str>, symbols: HashSet<&'inner str>) -> Lexer<'outer, 'inner> {
        let mut fused = vec!["[", "]", "{", "}", "{?", "?}", "(", ")", " ", "\n", "\t", "\r"];
        fused.extend(symbols.iter().cloned());

        let special_chars: HashSet<&'inner str> = fused.iter().map(|x| &**x).collect();

        let mut all_symbols = symbols;
        all_symbols.extend([" ", "\n", "\t", "\r"].iter());

        Lexer {
            tokens: vec![],

            position: (1, 1),
            index: 0,
            span: 1,

            special_chars,

            work_string: "",
            string: input,

            symbols: all_symbols,
            number: set!["1", "2", "3", "4", "5", "6", "7", "8", "9", "0"],

            ws: set![" ", "\n", "\t", "\r"],
            keep_ws: true,
        }
    }


    // [AREA] Mutating
    //
    fn advance(&mut self, new_span: usize) {
        self.position.1 += self.span;

        self.index += self.span;
        self.span = new_span;
    }

    fn next_line(&mut self) {
        self.position.0 += 1;
        self.position.1 = 1;
    }

    fn step_forward(&mut self) {
        self.span += 1;
    }

    fn step_back(&mut self) {
        self.span -= 1;
    }
    //
    // [END] Mutating


    // [AREA] Copying Fields
    //
    fn get_workspan(&self) -> &[&'outer str] {
        &self.string[self.index .. self.index + self.span]
    }
    //
    // [END] Copying Fields


    // [AREA] Checking Tokens
    //
    fn is_space(&self, s: &Vec<&str>) -> bool {
        s.iter().all(|x| self.ws.contains(x))
    }

    fn is_word(&self, s: &Vec<&str>) -> bool {
        s.iter().all(|x| !self.is_special(x))
    }

    fn is_int(&self, s: &Vec<&str>) -> bool {
        s.iter().all(|x| self.number.contains(x))
    }

    fn is_word_symbol(&self, s: &Vec<&str>) -> bool {
        if s.len() == 1 {
            false

        } else if let (Some(fc), Some(lc)) = (s.first(), s.last()) {
            if fc == &"`" && lc == &"`" {
                true// [NOTE] Hope that's not problematic...

            } else {
                false
            }

        } else {
            true
        }
    }

    // [NOTE] Returns `true` iff the number has decimal point.
    //
    fn is_float(&self, s: &Vec<&str>) -> bool {
        let mut saw_dot = false;

        for x in s.iter() {
            if !self.number.contains(x) && (saw_dot || x.to_owned() != ".") {
                return false;

            } else if x.to_owned() == "." {
                saw_dot = true;
            }
        }

        saw_dot
    }

    fn is_literal(&self, s: &Vec<&str>) -> bool {
        let joint = s.join("");
        let joint_ref: &str = joint.as_ref();

        self.is_word(s) || self.is_int(s) || self.is_float(s) || self.symbols.contains(joint_ref)
    }

    fn is_special(&self, s: &str) -> bool {
        match s {
            _ if self.special_chars.contains(s) => { true },
            _ => { false },
        }
    }
    //
    // [END] Checking Tokens

    fn process_literal(&self, workspan: Vec<&str>, old_span: usize) -> Token {
        let old_joint = workspan.join("");
        let old_s = old_joint.as_ref();

        let pos = self.position;
        let index = self.index;

        if self.symbols.contains(old_s) {
            Token::new_symbol(old_s, pos, index, old_span - 1)

        } else if self.is_int(&workspan) {
            Token::new_int(old_s.parse::<i64>().unwrap(), pos, index, old_span)

        } else if self.is_float(&workspan) {
            Token::new_float(old_s.parse::<f64>().unwrap(), pos, index, old_span)

        } else if self.is_word_symbol(&workspan) {
            let mut window = old_s[1..].to_owned();

            window.pop();

            Token::new_symbol(&window, pos, index, old_span)

        } else if self.is_word(&workspan) {
            Token::new_word(old_s, pos, index, old_span)

        } else {
            panic!("Invalid literal!");
        }
    }
}

impl<'outer, 'inner> Iterator for Lexer<'outer, 'inner> {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if self.index >= self.string.len() {
                return None;

            } else if self.index + self.span > self.string.len() {
                self.span = self.string.len() - self.index;
            }

            let mut workspan = self.get_workspan().to_vec();

            if !self.keep_ws && self.is_space(&workspan) {
                let newlines = workspan.iter().cloned().filter(|x| x.to_owned() == "\n").count();

                for _ in 0..newlines {
                    self.next_line();
                }

                self.advance(0);

            } else {
                let joint = workspan.join("");

                let pos = self.position;
                let index = self.index;
                let span = self.span;

                match joint.as_ref() {
                    "[" if !self.keep_ws => {
                        self.advance(1);

                        return Some(Token::new_control("[", pos, index, span));
                    },

                    "]" if !self.keep_ws => {
                        self.advance(1);

                        return Some(Token::new_control("]", pos, index, span));
                    },


                    "{" if !self.keep_ws => { /* Wait for potential `{?` */ },
                    "}" if !self.keep_ws => {
                        self.advance(1);

                        return Some(Token::new_control("}", pos, index, span));
                    },


                    "{?" if !self.keep_ws => {
                        self.advance(1);

                        return Some(Token::new_control("{?", pos, index, span));
                    },

                    "?}" if !self.keep_ws => {
                        self.advance(1);

                        return Some(Token::new_control("?}", pos, index, span));
                    },


                    "(" if !self.keep_ws => {
                        self.advance(1);

                        return Some(Token::new_control("(", pos, index, span));
                    },

                    ")" if !self.keep_ws => {
                        self.advance(1);

                        return Some(Token::new_control(")", pos, index, span));
                    },


                    s => {
                        if self.keep_ws {
                            if self.is_word(&workspan) {
                                // [NOTE] Greedily eat word

                            } else {
                                let mut less_span = workspan.clone();

                                less_span.pop();

                                if less_span.len() > 0 && self.is_word(&less_span) {
                                    self.step_back();
                                    self.advance(1);

                                    let joint = less_span.join("");

                                    return Some(Token::new_symbol(&joint, pos, index, span - 1));

                                } else {
                                    self.advance(1);

                                    let joint = workspan.join("");

                                    return Some(Token::new_symbol(&joint, pos, index, span));
                                }
                            }

                        } else {
                            if s.starts_with("{") {
                                self.step_back();
                                self.advance(1);

                                return Some(Token::new_control("{", pos, index, span - 1));

                            } else if self.is_literal(&workspan) {
                                if self.index + self.span >= self.string.len() {
                                    self.advance(1);

                                    return Some(self.process_literal(workspan, span));

                                } else {
                                    // [NOTE] Greedily eat literal.
                                }

                            } else {
                                workspan.pop();
                                self.step_back();

                                self.advance(1);

                                return Some(self.process_literal(workspan, span));
                            }
                        }
                    }
                }
            }

            self.step_forward();
        }
    }
}
