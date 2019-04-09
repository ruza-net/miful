extern crate unicode_segmentation;
use self::unicode_segmentation::UnicodeSegmentation;

use std::cmp;
use std::io::Write;
use std::collections::{ HashMap, HashSet };


const ERR_CONTEXT_LEN: usize = 10;


pub fn segment_text(input: &str) -> Vec<&str> {
    UnicodeSegmentation::graphemes(input, true).collect::<Vec<&str>>()
}

pub fn input<'a>() -> String {
    let mut in_s = String::new();

    let stdin = std::io::stdin();
    let mut stdout = std::io::stdout();

    let _ = stdout.flush();
    stdin.read_line(&mut in_s).expect("Failed to read input!");

    in_s
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

#[macro_export]
macro_rules! map {
    ( $( $k:expr => $v:expr ),* ) => {
        {
            let mut temp_map = HashMap::new();

            $(
                temp_map.insert($k, $v);
            )*

            temp_map
        }
    };

    // [NOTE] Enabling trailing comma.
    //
    ( $( $k:expr => $v:expr ),* , ) => {
        {
            let mut temp_map = HashMap::new();

            $(
                temp_map.insert($k, $v);
            )*

            temp_map
        }
    };
}


pub trait Error {
    fn print_err(&self) {
        let source = self.get_source();

        let start_pos = cmp::max(self.get_index(), ERR_CONTEXT_LEN) - ERR_CONTEXT_LEN;
        let end_pos = cmp::min(self.get_index() + ERR_CONTEXT_LEN, source.len());

        let mut extra_offset = 0;

        let src_line = source[start_pos .. end_pos]
            .to_owned()
            .iter()
            .map(|s| match s.as_ref() {
                "\n" => { extra_offset += 1; "\\n" },
                "\t" => { extra_offset += 1; "\\t" },
                "\r" => { extra_offset += 1; "\\r" },

                _ => { s },
            })
            .collect::<Vec<&str>>()
            .join("");

        let pos = self.get_position();

        println!("{} error occurred at {}:{}\n", self.get_kind(), pos.0, pos.1);

        println!("[source-string]:");
        println!("... {} ...", src_line);

        print!("    ");

        let tilde_count = extra_offset + (end_pos - start_pos) / 2;

        for _ in 0..tilde_count {
            print!("~");
        }

        println!("^ -- here\n");

        println!("{}", self.get_message());
    }

    fn throw_err(&self) {
        self.print_err();

        println!("\n");

        panic!(format!("{} error occurred!", self.get_kind()));
    }

    fn add_layer_top(&mut self, &str);

    fn get_kind(&self) -> &str;
    fn get_index(&self) -> usize;
    fn get_message(&self) -> String;
    fn get_source(&self) -> &Vec<String>;
    fn get_position(&self) -> (usize, usize);
}


// [NOTE] Parse errors get thrown by lexer.
//
#[derive(Clone, Debug)]
pub struct ParseError {
    index: usize,
    position: (usize, usize),
    source: Vec<String>,

    message: Vec<String>,
}

// [NOTE] Semantic errors get thrown by parser, which knows
// nothing about the source code. So it takes just index,
// and the contextual string is supplied when the error
// is thrown.
//
#[derive(Clone, Debug)]
pub struct SemanticError {
    index: usize,
    position: (usize, usize),

    message: Vec<String>,
    source: Vec<String>,
}

// [NOTE] Runtime errors get thrown by driver.
//
#[derive(Clone, Debug)]
pub struct RuntimeError {
    index: usize,
    position: (usize, usize),
    source: Vec<String>,

    message: Vec<String>,
}

#[derive(Clone, Debug)]
pub enum MifulError {
    Parsing(ParseError),
    Semantics(SemanticError),
    Runtime(RuntimeError),
}


impl MifulError {
    pub fn parse_error(message: &str, source: &Vec<String>, index: usize, position: (usize, usize)) -> MifulError {
        MifulError::Parsing(
            ParseError::new(message, source.to_vec(), index, position)
        )
    }

    pub fn semantic_error(message: &str, index: usize, position: (usize, usize)) -> MifulError {
        MifulError::Semantics(
            SemanticError::new(message, index, position)
        )
    }

    pub fn runtime_error(message: &str, source: &Vec<String>, index: usize, position: (usize, usize)) -> MifulError {
        MifulError::Runtime(
            RuntimeError::new(message, source.to_vec(), index, position)
        )
    }


    pub fn from_parse_error(e: ParseError) -> MifulError {
        MifulError::Parsing(e)
    }

    pub fn from_semantic_error(e: SemanticError) -> MifulError {
        MifulError::Semantics(e)
    }

    pub fn from_runtime_error(e: RuntimeError) -> MifulError {
        MifulError::Runtime(e)
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

            MifulError::Runtime(e) => {
                let mut new_e = e.clone();

                new_e.add_layer_top(message);

                MifulError::from_runtime_error(new_e)
            },
        }
    }

    pub fn supply_source(&mut self, src: &Vec<String>) {
        *self = match &self {
            MifulError::Semantics(e) => {
                let mut new_e = e.clone();

                new_e.supply_source(src);

                MifulError::from_semantic_error(new_e)
            },

            _ => { self.clone() }
        }
    }
}

impl Error for MifulError {
    fn add_layer_top(&mut self, message: &str) {
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

            MifulError::Runtime(e) => {
                let mut new_e = e.clone();

                new_e.add_layer_top(message);

                MifulError::from_runtime_error(new_e)
            },
        };
    }

    fn get_kind(&self) -> &str {
        match &self {
            MifulError::Parsing(e) => e.get_kind(),
            MifulError::Semantics(e) => e.get_kind(),
            MifulError::Runtime(e) => e.get_kind(),
        }
    }

    fn get_index(&self) -> usize {
        match &self {
            MifulError::Parsing(e) => e.get_index(),
            MifulError::Semantics(e) => e.get_index(),
            MifulError::Runtime(e) => e.get_index(),
        }
    }

    fn get_message(&self) -> String {
        match &self {
            MifulError::Parsing(e) => e.get_message(),
            MifulError::Semantics(e) => e.get_message(),
            MifulError::Runtime(e) => e.get_message(),
        }
    }

    fn get_source(&self) -> &Vec<String> {
        match &self {
            MifulError::Parsing(e) => e.get_source(),
            MifulError::Semantics(e) => e.get_source(),
            MifulError::Runtime(e) => e.get_source(),
        }
    }

    fn get_position(&self) -> (usize, usize) {
        match &self {
            MifulError::Parsing(e) => e.get_position(),
            MifulError::Semantics(e) => e.get_position(),
            MifulError::Runtime(e) => e.get_position(),
        }
    }
}


impl ParseError {
    pub fn new(message: &str, source: Vec<String>, index: usize, position: (usize, usize)) -> ParseError {
        ParseError {
            index,
            position,
            source,

            message:
                message
                    .clone()
                    .split('\n')
                    .map(ToOwned::to_owned)
                    .collect(),
        }
    }
}

impl Error for ParseError {
    fn add_layer_top(&mut self, message: &str) {
        let lines = message.clone().split('\n').map(ToOwned::to_owned);

        self.message = self.message.iter().map(|s| format!("| {}", s)).collect();

        self.message.splice(0..0, lines);
    }

    fn get_kind(&self) -> &str {
        "Parse"
    }

    fn get_index(&self) -> usize {
        self.index
    }

    fn get_message(&self) -> String {
        self.message.join("\n")
    }

    fn get_source(&self) -> &Vec<String> {
        &self.source
    }

    fn get_position(&self) -> (usize, usize) {
        self.position
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

    pub fn supply_source(&mut self, src: &Vec<String>) {
        self.source = src.to_vec();
    }
}

impl Error for SemanticError {
    fn add_layer_top(&mut self, message: &str) {
        let lines = message.clone().split('\n').map(ToOwned::to_owned);

        self.message = self.message.iter().map(|s| format!("| {}", s)).collect();

        self.message.splice(0..0, lines);
    }

    fn get_kind(&self) -> &str {
        "Semantic"
    }

    fn get_index(&self) -> usize {
        self.index
    }

    fn get_message(&self) -> String {
        self.message.join("\n")
    }

    fn get_source(&self) -> &Vec<String> {
        &self.source
    }

    fn get_position(&self) -> (usize, usize) {
        self.position
    }
}


impl RuntimeError {
    pub fn new(message: &str, source: Vec<String>, index: usize, position: (usize, usize)) -> RuntimeError {
        RuntimeError {
            index,
            position,
            source,

            message:
                message
                    .clone()
                    .split('\n')
                    .map(ToOwned::to_owned)
                    .collect(),
        }
    }
}

impl Error for RuntimeError {
    fn add_layer_top(&mut self, message: &str) {
        let lines = message.clone().split('\n').map(ToOwned::to_owned);

        self.message = self.message.iter().map(|s| format!("| {}", s)).collect();

        self.message.splice(0..0, lines);
    }

    fn get_kind(&self) -> &str {
        "Runtime"
    }

    fn get_index(&self) -> usize {
        self.index
    }

    fn get_message(&self) -> String {
        self.message.join("\n")
    }

    fn get_source(&self) -> &Vec<String> {
        &self.source
    }

    fn get_position(&self) -> (usize, usize) {
        self.position
    }
}
