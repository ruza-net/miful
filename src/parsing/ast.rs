use std::fmt::{self, Formatter, Display};


fn format_node(node: &Node, layer: usize, include_initial_indent: bool) -> String {
    let indent = "   ".repeat(layer).to_owned();
    let mut output;

    if include_initial_indent {
        output = indent.clone();

    } else {
        output = "".to_owned();
    }

    match node {
        Node::Word(val) => {
            output.push_str(&val);
        },


        Node::Symbol(val) => {
            output.push_str(&val);
        },


        Node::Float(val) => {
            output.push_str(&val.to_string());
        },


        Node::Int(val) => {
            output.push_str(&val.to_string());
        },

        Node::List(lst) => {
            output.push_str("(");

            for wrapper in lst {
                let current = format_node(&wrapper.node, layer + 1, false) + " ";

                output.push_str(&current);
            }

            output.pop();

            output.push_str(")");
        },

        Node::Block(lst) => {
            output.push_str("{\n");

            for wrapper in lst {
                let current = format_node(&wrapper.node, layer + 1, true) + "\n";

                output.push_str(&current);
            }

            output.push_str(&(indent + "}"));
        },

        Node::Invoke{ target, with } => {
            output.push_str("[");
            output.push_str(&(target.clone() + "\n"));

            for wrapper in with {
                let current = format_node(&wrapper.node, layer + 1, true) + "\n";

                output.push_str(&current);
            }

            output.push_str(&(indent + "]"));
        },
    }

    output
}


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
        write!(f, "{}", format_node(&self.node, 0, false))
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
