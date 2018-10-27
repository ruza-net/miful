use parsing::token::Token;
use parsing::token::TokenType;


trait Value {
    type Kind;

    fn get_value(&self) -> &Self::Kind;
}

trait ControlStructure {
    type Kind;
}

// [AREA] Declarations
//
#[derive(Debug)]
#[derive(Clone)]
pub struct Word<'a> {
    value: &'a str,

    position: (usize, usize)
}

#[derive(Debug)]
#[derive(Clone)]
pub struct Symbol<'a> {
    value: &'a str,

    position: (usize, usize)
}

#[derive(Debug)]
#[derive(Clone)]
pub struct Float {
    value: f64,

    position: (usize, usize)
}

#[derive(Debug)]
#[derive(Clone)]
pub struct Int {
    value: i64,

    position: (usize, usize)
}
//
// [END] Declarations

// [AREA] Implementations
//
impl<'a> Value for Word<'a> {
    type Kind = &'a str;

    fn get_value(&self) -> &&'a str {
        &self.value
    }
}

impl<'a> Value for Symbol<'a> {
    type Kind = &'a str;

    fn get_value(&self) -> &&'a str {
        &self.value
    }
}

impl Value for Float {
    type Kind = f64;

    fn get_value(&self) -> &f64 {
        &self.value
    }
}

impl Value for Int {
    type Kind = i64;

    fn get_value(&self) -> &i64 {
        &self.value
    }
}
//
// [END] Implementations

// [AREA] Structure Nodes
//
pub struct ListNode<T> {
    elements: Vec<Box<Value<Kind = T>>>,

    position: (usize, usize)
}

pub struct InvokeNode<'a, T> {
    target: Word<'a>,

    arguments: Vec<Box<Value<Kind = T>>>,

    position: (usize, usize)
}

impl<T> Value for ListNode<T> {
    type Kind = Vec<Box<Value<Kind = T>>>;

    fn get_value(&self) -> &Vec<Box<Value<Kind = T>>> {
        &self.elements
    }
}
//
// [END] Structure Nodes
