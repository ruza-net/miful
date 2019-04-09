use std::fmt::{ self, Formatter, Display };


fn format_node(node: &NodeKind, hooks: &Vec<NodeWrapper>, layer: usize, include_initial_indent: bool) -> String {
    let indent = "   ".repeat(layer).to_owned();
    let mut output;

    if include_initial_indent {
        output = indent.clone();

    } else {
        output = "".to_owned();
    }

    match node {
        NodeKind::Word(val) => {
            output.push_str(&val);
        },


        NodeKind::Symbol(val) => {
            output.push_str(&val);
        },


        NodeKind::Float(val) => {
            output.push_str(&val.to_string());
        },


        NodeKind::Int(val) => {
            output.push_str(&val.to_string());
        },

        NodeKind::List(lst) => {
            output.push_str("(");

            for wrapper in lst {
                let mut inner_hooks = wrapper.hooks.clone();
                inner_hooks.splice(0..0, hooks.clone());

                let current = format_node(&wrapper.node, &inner_hooks, layer + 1, false) + " ";

                output.push_str(&current);
            }

            if output.len() > 1 {
                output.pop();
            }

            output.push_str(")");
        },

        NodeKind::Quote{ target, with } => {
            output.push_str("{");
            output.push_str(&(target.to_string() + "\n"));

            for wrapper in with {
                let mut inner_hooks = wrapper.hooks.clone();
                inner_hooks.splice(0..0, hooks.clone());

                let current = format_node(&wrapper.node, &inner_hooks, layer + 1, true) + "\n";

                output.push_str(&current);
            }

            output.push_str(&(indent + "}"));
        },

        NodeKind::LambdaHook(v_idx) => {
            let idx = v_idx.clone();
            let node = &hooks[idx];

            output.push_str("{? ");

            output.push_str(&format_node(&node.node, &vec![], layer + 1, false));// [TODO] Nested hooks?

            output.push_str(" ?}");
        },

        NodeKind::Invoke{ target, with } => {
            output.push_str("[");
            output.push_str(&(target.to_string() + "\n"));

            for wrapper in with {
                let mut inner_hooks = wrapper.hooks.clone();
                inner_hooks.splice(0..0, hooks.clone());

                let current = format_node(&wrapper.node, &inner_hooks, layer + 1, true) + "\n";

                output.push_str(&current);
            }

            output.push_str(&(indent + "]"));
        },
    }

    output
}


#[derive(Clone, Debug)]
pub enum NodeKind {
    Word(String),
    Symbol(String),

    Float(f64),
    Int(i64),

    List(Vec<NodeWrapper>),

    LambdaHook(usize),
    Quote{ target: String, with: Vec<NodeWrapper> },
    Invoke{ target: String, with: Vec<NodeWrapper> },
}

impl Display for NodeKind {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match &self {
            NodeKind::Word(_) => write!(f, "Word"),
            NodeKind::Symbol(_) => write!(f, "Symbol"),

            NodeKind::Float(_) => write!(f, "Float"),
            NodeKind::Int(_) => write!(f, "Int"),

            NodeKind::List(_) => write!(f, "List"),

            NodeKind::LambdaHook(_) => write!(f, "LambdaHook"),
            NodeKind::Quote{ target:_, with:_ } => write!(f, "Quote"),
            NodeKind::Invoke{ target:_, with:_ } => write!(f, "Invoke"),
        }
    }
}


#[derive(Clone, Debug)]
pub struct NodeWrapper {
    pub node: NodeKind,
    pub hooks: Vec<NodeWrapper>,

    pub position: (usize, usize),
    pub index: usize,
}

impl Display for NodeWrapper {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}", format_node(&self.node, &self.hooks, 0, false))
    }
}

impl NodeWrapper {
    pub fn empty() -> NodeWrapper {
        NodeWrapper {
            node: NodeKind::List(vec![]),
            hooks: vec![],

            position: (0, 0),
            index: 0,
        }
    }


    // [AREA] Structure Nodes
    //
    pub fn new_list(elements: Vec<NodeWrapper>, hooks: Vec<NodeWrapper>, index: usize, position: (usize, usize)) -> NodeWrapper {
        NodeWrapper {
            node: NodeKind::List(elements),
            hooks,

            position,
            index,
        }
    }

    pub fn new_invoke(target: String, with: Vec<NodeWrapper>, hooks: Vec<NodeWrapper>, index: usize, position: (usize, usize))
        -> NodeWrapper {

        NodeWrapper {
            node: NodeKind::Invoke { target: target.to_owned(), with },
            hooks,

            position,
            index,
        }
    }

    pub fn new_quote(target: String, with: Vec<NodeWrapper>, hooks: Vec<NodeWrapper>, index: usize, position: (usize, usize))
        -> NodeWrapper {

        NodeWrapper {
            node: NodeKind::Quote { target: target.to_owned(), with },
            hooks,

            position,
            index,
        }
    }

    pub fn new_hook(v_idx: usize, hooks: Vec<NodeWrapper>, index: usize, position: (usize, usize)) -> NodeWrapper {
        NodeWrapper {
            node: NodeKind::LambdaHook(v_idx),
            hooks,// [NOTE] Not nested hooks, just the contained value.

            position,
            index,
        }
    }
    //
    // [END] Structure Nodes


    // [AREA] Value Nodes
    //
    pub fn new_word(value: String, index: usize, position: (usize, usize)) -> NodeWrapper {
        NodeWrapper {
            node: NodeKind::Word(value.to_owned()),
            hooks: vec![],

            position,
            index,
        }
    }

    pub fn new_symbol(value: String, index: usize, position: (usize, usize)) -> NodeWrapper {
        NodeWrapper {
            node: NodeKind::Symbol(value.to_owned()),
            hooks: vec![],

            position,
            index,
        }
    }

    pub fn new_float(value: f64, index: usize, position: (usize, usize)) -> NodeWrapper {
        NodeWrapper {
            node: NodeKind::Float(value),
            hooks: vec![],

            position,
            index,
        }
    }

    pub fn new_int(value: i64, index: usize, position: (usize, usize)) -> NodeWrapper {
        NodeWrapper {
            node: NodeKind::Int(value),
            hooks: vec![],

            position,
            index,
        }
    }
    //
    // [END] Value Node
}


#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum MifulType {
    Simple(String),// [NOTE] Covers `quote` and `list`, too.
    Object(String),

    Tuple(Vec<MifulType>),// [NOTE] Checks element count.
    List(Vec<MifulType>),// [NOTE] Doesn't check element count.

    AnyOf(Vec<MifulType>)
}

impl Display for MifulType {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match &self {
            MifulType::Simple(t) => {
                write!(f, "{}", t)
            },

            MifulType::Object(t) => {
                write!(f, "(obj {})", t)
            },

            MifulType::Tuple(ts) => {
                let mut s = String::from("(tuple (");

                for t in ts {
                    s.push_str(&format!("{} ", t));
                }

                s.pop();

                write!(f, "{}))", s)
            },

            MifulType::List(ts) => {
                let mut s = String::from("(list (");

                for t in ts {
                    s.push_str(&format!("{} ", t));
                }

                s.pop();

                write!(f, "{}))", s)
            },

            MifulType::AnyOf(ts) => {
                let mut s = String::from("(");

                for t in ts {
                    s.push_str(&format!("{} | ", t));
                }

                s.pop();
                s.pop();
                s.pop();

                write!(f, "{})", s)
            },
        }
    }
}
