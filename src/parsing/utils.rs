extern crate unicode_segmentation;
use self::unicode_segmentation::UnicodeSegmentation;

use std::cmp;


pub fn segment_text(input: &str) -> Vec<&str> {
    UnicodeSegmentation::graphemes(input, true).collect::<Vec<&str>>()
}

#[macro_export]
macro_rules! set {
    ( $( $x:expr ),* ) => {
        {
            let mut temp_set = HashSet::new();

            $(
                temp_set.insert($x);
            )*

            temp_set
        }
    };
}

#[derive(Clone)]
pub struct ParseError {
    index: usize,
    position: (usize, usize),

    message: Vec<String>,

    context: String,
    source: Vec<String>,
}

// [NOTE] Semantic errors get thrown by parser, which knows
// nothing about the source code. So it takes just index,
// and the contextual string is supplied when the error
// is thrown.
//
#[derive(Clone)]
pub struct SemanticError {
    position: (usize, usize),
    index: usize,

    message: Vec<String>,
    source: Vec<String>,
}

#[derive(Clone)]
pub enum MifulError {
    Parsing(ParseError),
    Semantics(SemanticError),
    //Runtime(RuntimeError),// TODO
}


impl MifulError {
    pub fn parse_error(message: &str, context: String, source: &Vec<String>, index: usize, position: (usize, usize)) -> MifulError {
        MifulError::Parsing(
            ParseError::new(message, context, source.clone(), index, position)
        )
    }

    pub fn semantic_error(message: &str, index: usize, position: (usize, usize)) -> MifulError {
        MifulError::Semantics(
            SemanticError::new(message, index, position)
        )
    }

    pub fn from_parse_error(e: ParseError) -> MifulError {
        MifulError::Parsing(e)
    }

    pub fn from_semantic_error(e: SemanticError) -> MifulError {
        MifulError::Semantics(e)
    }


    pub fn add_layer_top(&mut self, message: &str) {
        *self = match &self {
            MifulError::Parsing(e) => {
                let mut new_e = e.clone();

                new_e.add_layer_top(message);

                MifulError::from_parse_error(new_e)
            },

            MifulError::Semantics(e) => {
                let mut new_e = e.clone();

                new_e.add_layer_top(message);

                MifulError::from_semantic_error(new_e)
            },
        }
    }

    pub fn supply_source(&mut self, src: &Vec<String>) {
        *self = match &self {
            MifulError::Parsing(_) => { self.clone() },
            MifulError::Semantics(e) => {
                let mut new_e = e.clone();

                new_e.supply_source(src);

                MifulError::from_semantic_error(new_e)
            }
        }
    }
}

impl ParseError {
    pub fn new(message: &str, context: String, source: Vec<String>, index: usize, position: (usize, usize)) -> ParseError {
        ParseError {
            index,
            position,

            context,
            source,

            message:
                message
                    .clone()
                    .split('\n')
                    .map(ToOwned::to_owned)
                    .collect(),
        }
    }

    pub fn print_err(&self) {
        let start_pos = cmp::max(self.index, 5) - 5;
        let end_pos = cmp::min(self.index + 5, self.source.len());

        let src_line = self.source[start_pos .. end_pos]
            .to_owned()
            .join("");

        println!("Parse error occurred at {}:{}", self.position.0, self.position.1);
        println!("[context]: ... {} ...\n", self.context);

        println!("[source string]:");
        println!("... {} ...", src_line);

        print!("    ");

        for _ in 0..cmp::min(5, end_pos) {
            print!("~");
        }

        println!("^  -- here\n");

        println!("{}", self.get_message());
    }

    pub fn throw_err(&self) {
        self.print_err();

        println!("\n");

        panic!("Parse error occurred!");
    }

    pub fn add_layer_top(&mut self, message: &str) {
        let lines = message.clone().split('\n').map(ToOwned::to_owned);

        self.message = self.message.iter().map(|s| format!("| {}", s)).collect();

        self.message.splice(0..0, lines);
    }

    pub fn get_message(&self) -> String {
        self.message.join("\n")
    }
}

impl SemanticError {
    pub fn new(message: &str, index: usize, position: (usize, usize)) -> SemanticError {
        SemanticError {
            message:
                message
                    .clone()
                    .split('\n')
                    .map(ToOwned::to_owned)
                    .collect(),

            position,
            index,

            source: vec![],
        }
    }

    pub fn print_err(&self, context: String) {
        let start_pos = cmp::max(self.index, 5) - 5;
        let end_pos = cmp::min(self.index + 5, self.source.len());

        let src_line = self.source[start_pos .. end_pos]
            .to_owned()
            .join("");

        println!("Semantic error occurred at {}:{}", self.position.0, self.position.1);
        println!("[context]: ... {} ...\n", context);

        println!("[source string]:");
        println!("... {} ...", src_line);

        print!("    ");

        for _ in 0..cmp::min(5, end_pos) {
            print!("~");
        }

        println!("^ -- here\n");

        println!("{}", self.get_message());
    }

    pub fn throw_err(&self, context: String) {
        self.print_err(context);

        println!("\n");

        panic!("Semantic error occurred!");
    }

    pub fn add_layer_top(&mut self, message: &str) {
        let lines = message.clone().split('\n').map(ToOwned::to_owned);

        self.message = self.message.iter().map(|s| format!("| {}", s)).collect();

        self.message.splice(0..0, lines);
    }

    pub fn supply_source(&mut self, src: &Vec<String>) {
        self.source = src.to_vec();
    }

    fn get_message(&self) -> String {
        self.message.join("\n")
    }
}
