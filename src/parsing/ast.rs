use std::fmt::{self, Formatter, Display};


pub enum Node {
    Word(String),
    Symbol(String),

    Float(f64),
    Int(i64),

    List(Vec<NodeWrapper>),

    Block(Vec<NodeWrapper>),

    Invoke {
        target: String,
        with: Vec<NodeWrapper>
    }
}


pub struct NodeWrapper {
    pub node: Node,
    pub position: (usize, usize)
}

impl Display for NodeWrapper {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self.node {
            Node::Word(ref val) => {
                write!(f, "{}", val)
            },

            Node::Symbol(ref val) => {
                write!(f, "{}", val)
            },

            Node::Float(val) => {
                write!(f, "{}", val)
            },

            Node::Int(val) => {
                write!(f, "{}", val)
            },

            Node::List(ref elements) => {
                let mut lst_string: String = "".to_owned();

                for elem in elements {
                    lst_string.push_str(&format!("{} ", elem));
                }

                lst_string.pop();

                write!(f, "({})", lst_string)
            },

            Node::Block(ref invokes) => {
                let mut lst_string: String = "".to_owned();

                for invoke in invokes {
                    lst_string.push_str(&format!("{}\n", invoke));
                }

                write!(f, "{{\n{}}}", lst_string)
            },

            Node::Invoke { ref target, ref with } => {
                let mut lst_string: String = "".to_owned();

                for arg in with {
                    lst_string.push_str(&format!("{}\n", arg));
                }

                write!(f, "[{}\n{}]", target, lst_string)
            }
        }
    }
}

impl<'a> NodeWrapper {
    pub fn empty() -> NodeWrapper {
        NodeWrapper {
            node: Node::List(Vec::new()),
            position: (0, 0)
        }
    }

    pub fn new_list(elements: Vec<NodeWrapper>, position: (usize, usize)) -> NodeWrapper {
        NodeWrapper {
            node: Node::List(elements),
            position
        }
    }

    pub fn new_word(value: String, position: (usize, usize)) -> NodeWrapper {
        NodeWrapper {
            node: Node::Word(value),
            position
        }
    }

    pub fn new_symbol(value: String, position: (usize, usize)) -> NodeWrapper {
        NodeWrapper {
            node: Node::Symbol(value),
            position
        }
    }

    pub fn new_float(value: &str, position: (usize, usize)) -> NodeWrapper {
        let f_value = value.parse::<f64>().unwrap();

        NodeWrapper {
            node: Node::Float(f_value),
            position
        }
    }

    pub fn new_int(value: &str, position: (usize, usize)) -> NodeWrapper {
        let i_value = value.parse::<i64>().unwrap();

        NodeWrapper {
            node: Node::Int(i_value),
            position
        }
    }

    pub fn new_invoke(target: String, with: Vec<NodeWrapper>, position: (usize, usize))
        -> NodeWrapper {

        NodeWrapper {
            node: Node::Invoke { target, with },
            position
        }
    }

    pub fn new_block(invokes: Vec<NodeWrapper>, position: (usize, usize))
        -> NodeWrapper {

        NodeWrapper {
            node: Node::Block(invokes),
            position
        }
    }
}
