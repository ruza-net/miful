use parsing;
use parsing::token as tok;
use parsing::ast::{ NodeWrapper, NodeKind, MifulType };
use parsing::utils::{ MifulError, segment_text, input };

use std::collections::{ HashSet, HashMap };


/*

# Miful Default Driver

    aka MD2

## Functions

    MD2 implements these global functions:

* if (value) {quote:1} {quote:2}
    > runs {quote:1} if (value) returns sym(&) and runs {quote:2} otherwise
    > Example:
        ```
        [if [> [: age] 18] {display [: adult-content]} {display [: denial]}]
        ```

* + (int:1) (int:2)
    > adds (int:1) and (int:2)

* + (float:1) (float:2)
    > adds (float:1) and (float:2)

* + (word:1) (word:2)
    > concatenates (word:1) and (word:2)

* - (int:1) (int:2)
    > subtracts (int:1) and (int:2)

* - (float:1) (float:2)
    > subtracts (float:1) and (float:2)

* * (int:1) (int:2)
    > multiplies (int:1) and (int:2)

* * (float:1) (float:2)
    > multiplies (float:1) and (float:2)

* / (int:1) (int:2)
    > divides (int:1) and (int:2) and rounds the result down

* / (float:1) (float:2)
    > divides (float:1) and (float:2)

* % (int:1) (int:2)
    > performs integer division of (int:1) and (int:2) and returns the remainder

* % (float:1) (float:2)
    > performs integer division of (float:1) and (float:2) and returns the remainder

* floor (float)
    > rounds (float) towards negative infinity

* ceil (float)
    > rounds (float) towards infinity

* round (float)
    > rounds (float) to the nearest integer

* let (word) (value) {quote}
    > binds (value) to (word), every call to [: (word)] inside {quote} will result into (value)
    > NOTE: Similar behaviour to an unquoted block {? ... ?}
    > NOTE: Value can be read multiple times.

* define (word) (list<list<word, (type)>>) {quote}
    > creates a function binding for (word) with arguments specified in (list<...>),
    associated with {quote}
    > NOTE: This binding is valid after this definition (independent of scope).
    > NOTE: Redefining (shadowing) a function is not prohibited.
    > NOTE: Argument definition: 2-tuple - 1st element is name, 2nd is type (see ${Type structure})

* undefine (word)
    > removes a function binding for (word)
    > NOTE: If (word) doesn't have a binding, throws runtime error.

* split_at (word) (int)
    > returns a 2-tuple - 1st element is first (int) letters of (word), 2nd is the rest
    > NOTE: Ignores index overflow
    > NOTE: Runtime error at negative index

* struct (list<>)

* @ (structure) (word)
    > returns the associated function named (word) of (structure)


## Type structure

    In MD2, types are implemented as behavioural tuples.
They have this structure:

    ( (internal state constants), (public functions) )


## Unquote vs Function Arguments

    They are fundamentally the same, function arguments are syntactic sugar in the same way
as `let` bindings are.

*/


pub struct Driver<'a> {
    input: &'a str,

    owned_text: Vec<String>,
    keep_ws: bool,

    index: usize,
    ast: Vec<NodeWrapper>,

    // [NOTE] Modifying outer scope is prohibited.
    //
    scope: HashMap<String, NodeWrapper>,

    // [NOTE] Argument is a tuple (name, type), where type is typically a word:
    //    word, symbol, int, float, tuple, quote, any
    // ..or list:
    //    (tuple (..types..)), (obj word)
    //
    // [NOTE] Argument types allow no ambiguity.
    //
    functions: HashMap<(String, Vec<MifulType>), (Vec<String>, NodeWrapper)>,
}


impl<'a> Driver<'a> {
    pub fn new(input: &'a str) -> Driver<'a> {
        Driver {
            input,

            owned_text: vec![],
            keep_ws: false,

            index: 0,
            ast: vec![],

            scope: map!{
                String::from("space") => NodeWrapper::new_symbol(" ".to_owned(), 0, (0, 0)),
                String::from("tab") => NodeWrapper::new_symbol("\t".to_owned(), 0, (0, 0)),
                String::from("newline") => NodeWrapper::new_symbol("\n".to_owned(), 0, (0, 0)),
                String::from("carriage_ret") => NodeWrapper::new_symbol("\r".to_owned(), 0, (0, 0)),
                String::from("l_bracket") => NodeWrapper::new_symbol("[".to_owned(), 0, (0, 0)),
                String::from("r_bracket") => NodeWrapper::new_symbol("]".to_owned(), 0, (0, 0)),
                String::from("l_brace") => NodeWrapper::new_symbol("{".to_owned(), 0, (0, 0)),
                String::from("r_brace") => NodeWrapper::new_symbol("}".to_owned(), 0, (0, 0)),
                String::from("l_paren") => NodeWrapper::new_symbol("(".to_owned(), 0, (0, 0)),
                String::from("r_paren") => NodeWrapper::new_symbol(")".to_owned(), 0, (0, 0)),
            },

            functions: map!{},
        }
    }

    fn over_input_line(input: &'a str) -> Driver<'a> {
        Driver {
            input,

            owned_text: vec![],
            keep_ws: true,

            index: 0,
            ast: vec![],

            scope: map!{},
            functions: map!{},
        }
    }

    fn over(owned_text: Vec<String>, ast: Vec<NodeWrapper>, scope: HashMap<String, NodeWrapper>,
        functions: HashMap<(String, Vec<MifulType>), (Vec<String>, NodeWrapper)>) -> Driver<'a> {

            Driver {
                input: "",

                owned_text,
                keep_ws: false,

                index: 0,
                ast,

                scope,
                functions,
            }
    }

    pub fn process(&mut self) -> Result<Vec<NodeWrapper>, MifulError> {
        let symbols = Driver::symbols();
        let segmented_text = segment_text(self.input);

        self.owned_text = segmented_text.iter().cloned().map(ToOwned::to_owned).collect();

        if self.keep_ws {
            let lexer = parsing::lexer::Lexer::new_ws_preserving(segmented_text, symbols);
            let mut tokens: Vec<tok::Token> = lexer.collect();

            tokens.pop();// [NOTE] Drop last newline.

            let mut nodes = vec![];

            for t in tokens {
                let result: Result<Vec<_>, _> = parsing::parser::Parser::new(vec![t]).collect();

                match result {
                    Ok(lst) => {
                        nodes.push(lst[0].clone());
                    },

                    Err(e) => {
                        let mut new_e = e;

                        new_e.add_layer_top("..while interpreting input line");
                        new_e.supply_source(&self.owned_text);

                        return Err(new_e);
                    },
                }
            }

            Ok(nodes)

        } else {
            let lexer = parsing::lexer::Lexer::new(segmented_text, symbols);
            let tokens: Vec<tok::Token> = lexer.collect();

            let parser = parsing::parser::Parser::new(tokens);
            let result: Result<Vec<_>, _> = parser.collect();

            match result {
                Ok(ast) => {
                    self.ast = ast.clone();

                    Ok(ast)
                },

                Err(e) => {
                    let mut new_e = e;

                    new_e.add_layer_top("..while interpreting the source");
                    new_e.supply_source(&self.owned_text);

                    Err(new_e)
                },
            }
        }
    }

    // [AREA] Constant Utils
    //
    // [NOTE] Because of Rust's "Fuck you, you can't have a const set"
    //

    #[inline]
    fn symbols<'b>() -> HashSet<&'b str> {
        //
        // [NOTE] `?` and `@` are required to be symbols.

        set![":", "@", "&", "|", "#", "~", "?", "\\"]
    }

    #[inline]
    fn builtin_functions<'b>() -> HashSet<&'b str> {
        set!["print", "input", "mk-sym", ":", "return", "define", "obj-append", "length", "head",
        "tail", "reverse", "=", "+", "-", "*", "if"]
    }

    //
    // [END] Constant Utils

    // [TODO] Nested hooks?
    //
    fn resolve_hooks(&self, tree: NodeWrapper, hooks: &Vec<NodeWrapper>) -> NodeWrapper {
        let kind = tree.node.clone();

        match kind {
            NodeKind::List(lst) => {
                let mut new_lst = vec![];

                for val in lst {
                    let new_val = self.resolve_hooks(val, hooks);

                    new_lst.push(new_val);
                }

                NodeWrapper::new_list(new_lst, vec![], tree.index, tree.position)
            },

            NodeKind::Invoke{ target, with } => {
                let mut new_with = vec![];

                for arg in with {
                    let new_arg = self.resolve_hooks(arg, hooks);

                    new_with.push(new_arg);
                }

                NodeWrapper::new_invoke(target, new_with, vec![], tree.index, tree.position)
            },

            NodeKind::LambdaHook(idx) => {
                hooks[idx].clone()
            },

            _ => {
                tree
            },
        }
    }

    fn current(&self) -> Option<NodeWrapper> {
        if self.index < self.ast.len() {
            Some(self.ast[self.index].clone())

        } else {
            None
        }
    }


    // [AREA] Error Utils
    //

    fn invalid_param_count(&self, exp_count: usize, got_count: usize, n: NodeWrapper) -> MifulError {
        MifulError::runtime_error(&format!("Expected {} parameters; got {}!", exp_count, got_count), &self.owned_text, n.index, n.position)
    }

    fn param_eval(&self, e: MifulError) -> MifulError {
        let mut new_e = e;

        new_e.add_layer_top("..while evaluating invoke parameters");

        new_e
    }

    fn param_type(&self, exp_t: &str, idx: usize, pos: (usize, usize)) -> MifulError {
        MifulError::runtime_error(&format!("Invalid parameter type, expecting ` {} `!", exp_t), &self.owned_text, idx, pos)
    }

    fn type_signature(&self, val_node: &NodeWrapper) -> MifulError {
        MifulError::runtime_error("Invalid type signature!", &self.owned_text, val_node.index, val_node.position)
    }

    //
    // [END] Error Utils


    // [AREA] Type Utils
    //

    fn check_type(&self, val_node: &NodeWrapper, t: &MifulType) -> bool {
        let val = &val_node.node;

        match t {
            MifulType::Simple(s) => {
                if s == "any" {
                    true

                } else {
                    match val {
                        NodeKind::Float(_) => { s == "float" },
                        NodeKind::Int(_) => { s == "int" },

                        NodeKind::Word(_) => { s == "word" },
                        NodeKind::Symbol(_) => { s == "symbol" },

                        NodeKind::List(_) => { s == "list" },

                        // [TODO] Maybe add optional check for parameter signature?
                        //
                        NodeKind::Quote{ target: _, with: _ } => { s == "quote" },

                        _ => { panic!("Unprocessed value node!"); },
                    }
                }
            },

            MifulType::Object(class_name) => {
                self.check_obj_type(&val, class_name)
            },

            MifulType::Tuple(types) => {
                if let NodeKind::List(inner_lst) = val {
                    if types.len() == inner_lst.len() {
                        inner_lst.iter()
                            .zip(types.iter())
                            .all(|(inner_val, inner_t)| self.check_type(inner_val, inner_t))

                    } else {
                        false
                    }

                } else {
                    false
                }
            },

            MifulType::List(types) => {
                if let NodeKind::List(inner_lst) = val {
                    for v in inner_lst {
                        if types.iter().all(|t| !self.check_type(v, t)) {
                            return false;
                        }
                    }

                    true

                } else {
                    false
                }
            }

            MifulType::AnyOf(types) => {
                for t in types {
                    if self.check_type(val_node, t) {
                        return true;
                    }
                }

                false
            }
        }
    }

    fn check_obj_type(&self, val: &NodeKind, t: &str) -> bool {
        if let NodeKind::List(st) = val {
            if st.len() == 3 {
                if let NodeKind::Symbol(v) = &st[0].node {
                    if let NodeKind::Word(s) | NodeKind::Symbol(s) = &st[1].node {
                        if let NodeKind::List(_) = &st[2].node {
                            return v == "_obj" && (t == "any" || s == t);
                        }
                    }
                }
            }
        }

        false
    }

    fn make_object(&self, t: String, lst: Vec<NodeWrapper>, index: usize, position: (usize, usize)) -> NodeWrapper {
        let obj_sym = NodeWrapper::new_symbol("_obj".to_owned(), index, position);
        let obj_type = NodeWrapper::new_word(t, index, position);

        let mut hooks = vec![];

        for t in &lst {
            hooks.extend(t.hooks.clone());
        }

        let obj_val = NodeWrapper::new_list(lst, hooks.clone(), index, position);

        NodeWrapper::new_list(vec![obj_sym, obj_type, obj_val], hooks, index, position)
    }

    fn make_nil(&self) -> NodeWrapper {
        self.make_object("nil".to_owned(), vec![], 0, (0, 0))
    }

    fn make_true(&self) -> NodeWrapper {
        NodeWrapper::new_symbol("true".to_owned(), 0, (0, 0))
    }

    fn make_false(&self) -> NodeWrapper {
        NodeWrapper::new_symbol("false".to_owned(), 0, (0, 0))
    }

    fn get_obj_val(&self, obj: &NodeWrapper) -> Result<Vec<NodeWrapper>, MifulError> {
        if let NodeKind::List(ref obj_struct) = obj.node {
            if obj_struct.len() == 3 {
                if self.check_obj_type(&obj.node, "any") {
                    if let NodeKind::List(lst) = &obj_struct[2].node {
                        Ok(lst.to_vec())

                    } else {
                        unreachable!();
                    }// [UNREACHABLE]

                } else {
                    Err(MifulError::runtime_error("Invalid object structure!", &self.owned_text, obj.index, obj.position))
                }// [ERR] Invalid object structure

            } else {
                Err(MifulError::runtime_error("Object structure has invalid length!", &self.owned_text, obj.index, obj.position))
            }// [Err] Invalid object length

        } else {
            Err(MifulError::runtime_error("Object node is of invalid kind!", &self.owned_text, obj.index, obj.position))
        }// [Err] Invalid object kind
    }

    fn get_obj_type(&self, obj: &NodeWrapper) -> Result<String, MifulError> {
        if let NodeKind::List(ref obj_struct) = obj.node {
            if obj_struct.len() == 3 {
                if self.check_obj_type(&obj.node, "any") {
                    if let NodeKind::Word(name) | NodeKind::Symbol(name) = &obj_struct[1].node {
                        Ok(name.to_owned())

                    } else {
                        unreachable!();
                    }// [UNREACHABLE]

                } else {
                    Err(MifulError::runtime_error("Invalid object structure!", &self.owned_text, obj.index, obj.position))
                }// [ERR] Invalid object structure

            } else {
                Err(MifulError::runtime_error("Object structure has invalid length!", &self.owned_text, obj.index, obj.position))
            }// [Err] Invalid object length

        } else {
            Err(MifulError::runtime_error("Object node is of invalid kind!", &self.owned_text, obj.index, obj.position))
        }// [Err] Invalid object kind
    }

    fn obj_append(&self, obj: &NodeWrapper, lst: Vec<NodeWrapper>) -> Result<NodeWrapper, MifulError> {
        let result_name = self.get_obj_type(obj);
        let result_val = self.get_obj_val(obj);

        match result_name {
            Ok(obj_name) => {
                match result_val {
                    Ok(val) => {
                        let obj_val =
                            val.iter()
                                .cloned()
                                .chain(lst.iter().cloned())
                                .collect();

                        Ok(self.make_object(obj_name, obj_val, obj.index, obj.position))
                    },

                    Err(e) => {
                        let mut new_e = e;

                        new_e.add_layer_top("..while processing object structure");

                        Err(new_e)
                    },
                }
            },

            Err(e) => {
                let mut new_e = e;

                new_e.add_layer_top("..while processing object name");

                Err(new_e)
            },
        }
    }

    fn list_to_types(&self, lst: &Vec<NodeWrapper>) -> Result<Vec<MifulType>, MifulError> {
        let mut types = vec![];
        let mut next_union = false;

        for node in lst {
            match node.node.clone() {
                NodeKind::Word(t_name) | NodeKind::Symbol(t_name) => {
                    if t_name == "|" {
                        if types.len() == 0 {
                            return Err(MifulError::runtime_error("Invalid type union syntax!", &self.owned_text, node.index, node.position));
                        }// [ERR] Union syntax

                        next_union = true;

                        continue;

                    } else {
                        types.push(MifulType::Simple(t_name));
                    }
                },

                NodeKind::List(lst) => {
                    if lst.len() == 2 {
                        if let NodeKind::Word(t_name) | NodeKind::Symbol(t_name) = &lst[0].node {
                            match t_name.as_ref() {
                                "tuple" => {
                                    if let NodeKind::List(t_list) = &lst[1].node {
                                        let result = self.list_to_types(&t_list);

                                        match result {
                                            Ok(inner_types) => {
                                                types.push(MifulType::Tuple(inner_types));
                                            },

                                            Err(e) => {
                                                let mut new_e = e;

                                                new_e.add_layer_top("..while processing compound type");

                                                return Err(new_e);
                                            }// [ERR] Processing compound type
                                        }

                                    } else {
                                        return Err(self.type_signature(&lst[1]));
                                    }// [ERR] Type signature
                                },

                                "list" => {
                                    if let NodeKind::List(t_list) = &lst[1].node {
                                        let result = self.list_to_types(&t_list);

                                        match result {
                                            Ok(inner_types) => {
                                                types.push(MifulType::List(inner_types));
                                            },

                                            Err(e) => {
                                                let mut new_e = e;

                                                new_e.add_layer_top("..while processing compound type");

                                                return Err(new_e);
                                            }// [ERR] Processing compound type
                                        }

                                    } else {
                                        return Err(self.type_signature(&lst[1]));
                                    }// [ERR] Type signature
                                },

                                "obj" => {
                                    if let NodeKind::Word(obj_name) | NodeKind::Symbol(obj_name) = &lst[1].node {
                                        types.push(MifulType::Object(obj_name.to_owned()));

                                    } else {
                                        return Err(self.type_signature(&lst[1]));
                                    }// [ERR] Type signature
                                },

                                _ => { return Err(self.type_signature(&lst[0])) },
                            }

                        } else {
                            return Err(self.type_signature(&node));
                        }// [ERR] Type signature

                    } else {
                        let result = self.list_to_types(&lst);

                        match result {
                            Ok(inner_ts) => {
                                if inner_ts.len() == 1 {
                                    types.push(inner_ts[0].clone())

                                } else {
                                    return Err(self.type_signature(&node));
                                }// [ERR] Type signature
                            },

                            Err(e) => {
                                return Err(e);
                            },
                        }
                    }// [ERR] Type signature
                },

                _ => { return Err(self.type_signature(&node)); },
            }

            if next_union {
                next_union = false;

                let last = types.pop().unwrap();
                let before_last = types.pop().unwrap();

                if let MifulType::AnyOf(lst) = before_last {
                    let mut new_lst = lst;

                    new_lst.push(last);

                    types.push(MifulType::AnyOf(new_lst));

                } else {
                    types.push(MifulType::AnyOf(vec![before_last, last]));
                }
            }
        }

        Ok(types)
    }

    //
    // [END] Type Utils


    // [AREA] Function Utils
    //

    fn print_fn(&self, val: NodeWrapper) -> Result<NodeWrapper, MifulError> {
        let kind = &val.node;

        if let NodeKind::Word(s) | NodeKind::Symbol(s) = kind {
            print!("{}", s);

            Ok(self.make_nil())

        } else if self.check_obj_type(kind, "string") {
            let obj_struct = self.get_obj_val(&val).unwrap();

            for v in obj_struct {
                match self.print_fn(v) {//[TODO]
                    Ok(_) => {},

                    Err(e) => {
                        return Err(self.param_eval(e));
                    },
                }
            }

            Ok(self.make_nil())

        } else {
            Err(self.param_type("(obj string)", val.index, val.position))
        }// [ERR] Parameter type
    }

    fn args_compatible(&self, exp_args: &Vec<MifulType>, sup_args: &Vec<NodeWrapper>) -> bool {
        if exp_args.len() == sup_args.len() {
            for (val, t) in sup_args.iter().zip(exp_args.iter()) {
                if !self.check_type(val, t) {
                    return false;
                }
            }

            true

        } else {
            false
        }
    }

    fn choose_function(&self, name: &str, params: Vec<NodeWrapper>, n: &NodeWrapper) -> Result<(HashMap<String, NodeWrapper>, NodeWrapper), MifulError> {
        let mut available = vec![];

        for ((f_name, exp_args), (arg_names, body_invoke)) in &self.functions {
            if name == f_name {
                if self.args_compatible(exp_args, &params) {
                    let args = arg_names.iter()
                        .cloned()
                        .zip(params.iter().cloned())
                        .collect();

                    return Ok((args, body_invoke.clone()));

                } else {
                    let mut af = format!("{} :: ", f_name);

                    for t in exp_args {
                        af.push_str(&format!("{}, ", t));
                    }

                    af.pop();
                    af.pop();

                    if exp_args.len() == 0 {
                        af.pop();
                        af.pop();
                    }

                    available.push(af);
                }// Extending available functions
            }
        }

        // [TODO] Maybe print given parameter types?
        //
        Err(MifulError::runtime_error(
            &format!("Did not find function ` {} ` with desired parameter types.\n\t[NOTE] Following are available:\n\t{}",
                name,
                available.join("\n\t")),
            &self.owned_text,
            n.index,
            n.position
        ))
    }

    fn call_function(&self, name: &str, params: Vec<NodeWrapper>, n: &NodeWrapper) -> Result<NodeWrapper, MifulError> {
        let result = self.choose_function(name, params, n);

        match result {
            Ok((args, body)) => {
                let mut loc_scope = self.scope.clone();

                loc_scope.extend(args);

                // [TODO] Local functions?
                // (from the scope of the function declaration)
                //
                // [IDEA] Maybe insert the outer function body when processing quote.
                //
                let call_driver = Driver::over(self.owned_text.clone(), vec![body.clone()], loc_scope, self.functions.clone());
                let call_result: Result<Vec<_>, _> = call_driver.collect();

                match call_result {
                    Ok(ret) => {
                        return Ok(ret[0].clone());
                    },

                    Err(e) => {
                        let mut new_e = e;

                        new_e.add_layer_top(&format!("..while calling function {}", name));

                        return Err(new_e);
                    },
                }
            },

            Err(e) => {
                Err(e)
            },
        }
    }

    fn define_function(&mut self, name: &str, raw_signature: Vec<NodeWrapper>, body: NodeWrapper) -> Result<NodeWrapper, MifulError> {
        //
        // [NOTE] `body` is already converted from quote to invoke.

        if !Driver::builtin_functions().contains(name) {
            let mut names = vec![];
            let mut raw_types = vec![];

            for raw_arg in raw_signature {
                if let NodeKind::List(raw_pair) = &raw_arg.node {
                    if raw_pair.len() == 2 {
                        if let NodeKind::Word(name) | NodeKind::Symbol(name) = &raw_pair[0].node {
                            names.push(name.to_owned());
                            raw_types.push(raw_pair[1].clone());

                        } else {
                            return Err(self.type_signature(&raw_pair[0]));
                        }// [ERR] Type signature

                    } else {
                        return Err(self.type_signature(&raw_arg));
                    }// [ERR] Type signature

                } else {
                    return Err(self.type_signature(&raw_arg));
                }// [ERR] Type signature
            }

            match self.list_to_types(&raw_types) {
                Ok(types) => {
                    // let result = self.inline_invokes(&body);
                    //
                    // match result {
                    //     Ok(processed_body) => {
                    //         self.functions.insert((name.to_owned(), types), (names, processed_body));
                    //
                    //         Ok(self.make_nil())
                    //     },
                    //
                    //     Err(e) => {
                    //         let mut new_e = e;
                    //
                    //         new_e.add_layer_top("..while defining function");
                    //
                    //         Err(new_e)
                    //     }// [ERR] While defining function
                    // }

                    self.functions.insert((name.to_owned(), types), (names, body));

                    Ok(self.make_nil())
                },

                Err(e) => {
                    let mut new_e = e;

                    new_e.add_layer_top("..while defining function");

                    Err(new_e)
                }// [ERR] Function definition
            }

        } else {
            let current = self.current().unwrap();

            return Err(MifulError::runtime_error("Cannot override built-in function!", &self.owned_text, current.index, current.position))
        }// [ERR] Built-in override
    }

    fn values_equal(&self, v1: &NodeWrapper, v2: &NodeWrapper) -> Result<NodeWrapper, MifulError> {
        let current = self.current().unwrap();

        match (&v1.node, &v2.node) {
            (NodeKind::Int(i1), NodeKind::Int(i2)) => {
                if i1 == i2 {
                    Ok(self.make_true())

                } else {
                    Ok(self.make_false())
                }
            },

            (NodeKind::Float(f1), NodeKind::Float(f2)) => {
                if f1 == f2 {
                    Ok(self.make_true())

                } else {
                    Ok(self.make_false())
                }
            },

            (NodeKind::Word(w1), NodeKind::Word(w2)) => {
                if w1 == w2 {
                    Ok(self.make_true())

                } else {
                    Ok(self.make_false())
                }
            },

            (NodeKind::Symbol(s1), NodeKind::Symbol(s2)) => {
                if s1 == s2 {
                    Ok(self.make_true())

                } else {
                    Ok(self.make_false())
                }
            },

            (NodeKind::List(l1), NodeKind::List(l2)) => {
                if l1.len() == l2.len() {
                    for (a, b) in l1.iter().zip(l2.iter()) {
                        let result = self.values_equal(a, b);

                        match result {
                            Ok(ret) => {
                                if let NodeKind::Symbol(b) = ret.node {
                                    if b == "false" {
                                        return Ok(self.make_false());
                                    }

                                } else {
                                    unreachable!();
                                }// [UNREACHABLE]
                            },

                            Err(e) => {
                                let mut new_e = e;

                                new_e.add_layer_top("..while checking list equality");

                                return Err(new_e);
                            }// [ERR] While checking list equality
                        }
                    }

                    Ok(self.make_true())

                } else {
                    Ok(self.make_false())
                }
            },

            (NodeKind::LambdaHook(i1), NodeKind::LambdaHook(i2)) => {
                if i1 == i2 {
                    Ok(self.make_true())

                } else {
                    Ok(self.make_false())
                }
            },

            (NodeKind::Invoke{ target: _, with: _ }, _) => { panic!("[values_equal] Unprocessed node!"); },
            (_, NodeKind::Invoke{ target: _, with: _ }) => { panic!("[values_equal] Unprocessed node!"); },

            (NodeKind::Quote{ target: _, with: _ }, _) => { Err(MifulError::runtime_error("Can't check equality of quote!", &self.owned_text, current.index, current.position)) },
            (_, NodeKind::Quote{ target: _, with: _ }) => { Err(MifulError::runtime_error("Can't check equality of quote!", &self.owned_text, current.index, current.position)) },

            _ => { Ok(self.make_false()) }
        }
    }

    //
    // [END] Function Utils
}


impl<'a> Iterator for Driver<'a> {
    type Item = Result<NodeWrapper, MifulError>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.ast.len() {
            None

        } else {
            let n = self.ast[self.index].clone();

            let kind = n.node.clone();
            let hooks = n.hooks.clone();

            let own_text = self.owned_text.clone();
            let loc_scope = self.scope.clone();
            let loc_functions = self.functions.clone();

            self.index += 1;

            match kind {
                NodeKind::Word(_) | NodeKind::Symbol(_) | NodeKind::Int(_) | NodeKind::Float(_) => {
                    Some(Ok(n))
                },

                // [TODO] Hooks
                //
                NodeKind::List(ref lst) => {
                    let inner_driver = Driver::over(own_text, lst.to_vec(), loc_scope, loc_functions);
                    let result = inner_driver.collect();

                    match result {
                        Ok(ret) => {
                            Some(Ok(NodeWrapper::new_list(ret, vec![], n.index, n.position)))
                        },

                        Err(e) => {
                            let mut new_e = e;

                            new_e.add_layer_top("..while evaluating list elements");

                            Some(Err(new_e))
                        },
                    }
                },

                // [TODO] Hooks
                //
                NodeKind::Invoke{ target, with } => {
                    let inner_driver = Driver::over(own_text, with.to_vec(), loc_scope, loc_functions);
                    let result: Result<Vec<_>, _> = inner_driver.collect();

                    match result {
                        Ok(args) => {
                            match target.as_ref() {
                                "print" => {
                                    //
                                    // Prints (word:1) or (symbol:1) or ((obj string):1)

                                    if args.len() == 1 {
                                        Some(self.print_fn(args[0].clone()))

                                    } else {
                                        Some(Err(self.invalid_param_count(1, args.len(), n)))
                                    }// [ERR] Parameter count
                                },

                                "input" => {
                                    //
                                    // Prints (printable:1), reads line from stdin, and returns it as (obj string)

                                    if args.len() == 1 {
                                        let result = self.print_fn(args[0].clone());

                                        match result {
                                            Ok(_v) => {
                                                let read_line = input();

                                                let line_result = Driver::over_input_line(&read_line).process();

                                                match line_result {
                                                    Ok(node_lst) => {
                                                        Some(Ok(self.make_object("string".to_owned(), node_lst, n.index, n.position)))
                                                    },

                                                    Err(e) => {
                                                        Some(Err(e))
                                                    }
                                                }
                                            },

                                            Err(e) => {
                                                Some(Err(e))
                                            },
                                        }

                                    } else {
                                        Some(Err(self.invalid_param_count(1, args.len(), n)))
                                    }// [ERR] Parameter count
                                },

                                "mk-sym" => {
                                    //
                                    // Creates a symbol from (word:1)
                                    //
                                    // [NOTE] Synonymous with `(word:1)`

                                    if args.len() == 1 {
                                        if let NodeKind::Word(w) = &args[0].node {
                                            Some(Ok(NodeWrapper::new_symbol(w.to_owned(), n.index, n.position)))

                                        } else if let NodeKind::Int(i) = &args[0].node {
                                            Some(Ok(NodeWrapper::new_symbol(i.to_string(), n.index, n.position)))

                                        } else if let NodeKind::Float(f) = &args[0].node {
                                            Some(Ok(NodeWrapper::new_symbol(f.to_string(), n.index, n.position)))

                                        } else {
                                            Some(Err(self.param_type("word", args[0].index, args[0].position)))
                                        }// [ERR] Parameter type

                                    } else {
                                        Some(Err(self.invalid_param_count(1, args.len(), n)))
                                    }// [ERR] Parameter count
                                },

                                // "export" => {
                                //     //
                                // },

                                ":" => {
                                    //
                                    // Returns the value in scope under the name of (word:1)

                                    if args.len() == 1 {
                                        if let NodeKind::Word(v) | NodeKind::Symbol(v) = &args[0].node {
                                            if let Some(val) = self.scope.get(v) {
                                                Some(Ok(val.clone()))

                                            } else {
                                                Some(Err(MifulError::runtime_error("Undefined constant!", &self.owned_text, n.index, n.position)))
                                            }// [ERR] Undefined constant

                                        } else {
                                            Some(Err(self.param_type("(word | symbol)", n.index, n.position)))
                                        }// [ERR] Parameter type

                                    } else {
                                        Some(Err(self.invalid_param_count(1, args.len(), n)))
                                    }// [ERR] Parameter count
                                },

                                "=" => {
                                    //
                                    // Returns whether (val:1) and (val:2) are equal.

                                    if args.len() == 2 {
                                        Some(self.values_equal(&args[0], &args[1]))

                                    } else {
                                        Some(Err(self.invalid_param_count(1, args.len(), n)))
                                    }// [ERR] Parameter count
                                },

                                // [TODO] Hooks
                                //
                                "+" => {
                                    //
                                    // Returns (int:1) + (int:2), (float:1) + (float:2),
                                    // or is synonymous with [obj-append [:obj1] [obj-unwrap [:obj2]]],
                                    // where (obj:1) and (obj:2) are of the same type.

                                    if args.len() == 2 {
                                        let a = &args[0];
                                        let b = &args[1];

                                        match (&a.node, &b.node) {
                                            (NodeKind::Int(i1), NodeKind::Int(i2)) => {
                                                Some(Ok(NodeWrapper::new_int(i1 + i2, n.index, n.position)))
                                            },

                                            (NodeKind::Float(f1), NodeKind::Float(f2)) => {
                                                Some(Ok(NodeWrapper::new_float(f1 + f2, n.index, n.position)))
                                            },

                                            (NodeKind::List(l1), NodeKind::List(l2)) => {
                                                if self.check_obj_type(&a.node, "any")
                                                    && self.check_obj_type(&b.node, "any") {

                                                    let t1 = self.get_obj_type(&l1[1]).unwrap();
                                                    let t2 = self.get_obj_type(&l2[1]).unwrap();

                                                    if t1 == t2 {
                                                        Some(Ok(self.obj_append(a, l2.to_vec()).unwrap()))

                                                    } else {
                                                        Some(Err(MifulError::runtime_error("Can't concat two different objects!", &self.owned_text, n.index, n.position)))
                                                    }// [ERR] Different objects concat

                                                } else {
                                                    let mut new_l = l1.clone();
                                                    let mut new_hooks = a.hooks.clone();

                                                    new_l.append(&mut l2.clone());
                                                    new_hooks.append(&mut b.hooks.clone());

                                                    Some(Ok(NodeWrapper::new_list(new_l, new_hooks, n.index, n.position)))
                                                }
                                            },

                                            _ => {
                                                Some(Err(self.param_type("(int | float | (obj any) | list)", b.index, b.position)))
                                            }// [ERR] Parameter type
                                        }

                                    } else {
                                        Some(Err(self.invalid_param_count(1, args.len(), n)))
                                    }// [ERR] Parameter count
                                },

                                "-" => {
                                    //
                                    // Returns (int:1) - (int:2) or (float:1) - (float:2).

                                    if args.len() == 2 {
                                        let a = &args[0];
                                        let b = &args[1];

                                        match (&a.node, &b.node) {
                                            (NodeKind::Int(i1), NodeKind::Int(i2)) => {
                                                Some(Ok(NodeWrapper::new_int(i1 - i2, n.index, n.position)))
                                            },

                                            (NodeKind::Float(f1), NodeKind::Float(f2)) => {
                                                Some(Ok(NodeWrapper::new_float(f1 - f2, n.index, n.position)))
                                            },

                                            _ => {
                                                Some(Err(self.param_type("(int | float)", b.index, b.position)))
                                            }// [ERR] Parameter type
                                        }

                                    } else {
                                        Some(Err(self.invalid_param_count(1, args.len(), n)))
                                    }// [ERR] Parameter count
                                },

                                "*" => {
                                    //
                                    // Returns (int:1) * (int:2), (float:1) * (float:2).

                                    if args.len() == 2 {
                                        let a = &args[0];
                                        let b = &args[1];

                                        match (&a.node, &b.node) {
                                            (NodeKind::Int(i1), NodeKind::Int(i2)) => {
                                                Some(Ok(NodeWrapper::new_int(i1 * i2, n.index, n.position)))
                                            },

                                            (NodeKind::Float(f1), NodeKind::Float(f2)) => {
                                                Some(Ok(NodeWrapper::new_float(f1 * f2, n.index, n.position)))
                                            },

                                            _ => {
                                                Some(Err(self.param_type("(int | float)", b.index, b.position)))
                                            }// [ERR] Parameter type
                                        }

                                    } else {
                                        Some(Err(self.invalid_param_count(1, args.len(), n)))
                                    }// [ERR] Parameter count
                                },

                                "if" => {
                                    //
                                    // Runs (quote:2) when (value:1) is `true`, or runs (quote:3) otherwise.

                                    if args.len() == 3 {
                                        let cond_node = &args[0];
                                        let true_node = &args[1];
                                        let false_node = &args[2];

                                        if let NodeKind::Quote{ target: t_target, with: t_with } = &true_node.node {
                                            if let NodeKind::Quote{ target: f_target, with: f_with } = &false_node.node {
                                                if let NodeKind::Symbol(s) = &cond_node.node {
                                                    if s == "true" {
                                                        let t_invoke = NodeWrapper::new_invoke(t_target.to_string(),
                                                            t_with.to_vec(), true_node.hooks.clone(), true_node.index, true_node.position);

                                                        let inner_driver = Driver::over(self.owned_text.clone(), vec![t_invoke], self.scope.clone(), self.functions.clone());
                                                        let arg_result: Result<Vec<_>, _> = inner_driver.collect();

                                                        match arg_result {
                                                            Ok(ret) => {
                                                                return Some(Ok(ret[0].clone()));
                                                            },

                                                            Err(e) => {
                                                                let mut new_e = e;

                                                                new_e.add_layer_top("..while evaluating parameters");

                                                                return Some(Err(new_e));
                                                            }// [ERR] While param eval
                                                        }
                                                    }
                                                }

                                                let f_invoke = NodeWrapper::new_invoke(f_target.to_string(),
                                                    f_with.to_vec(), false_node.hooks.clone(), false_node.index, false_node.position);

                                                let inner_driver = Driver::over(self.owned_text.clone(), vec![f_invoke], self.scope.clone(), self.functions.clone());
                                                let arg_result: Result<Vec<_>, _> = inner_driver.collect();

                                                match arg_result {
                                                    Ok(ret) => {
                                                        return Some(Ok(ret[0].clone()));
                                                    },

                                                    Err(e) => {
                                                        let mut new_e = e;

                                                        new_e.add_layer_top("..while evaluating parameters");

                                                        return Some(Err(new_e));
                                                    }// [ERR] While param eval
                                                }

                                            } else {
                                                Some(Err(self.param_type("quote", true_node.index, false_node.position)))
                                            }// [ERR] Parameter type

                                        } else {
                                            Some(Err(self.param_type("quote", true_node.index, true_node.position)))
                                        }// [ERR] Parameter type

                                    } else {
                                        Some(Err(self.invalid_param_count(3, args.len(), n)))
                                    }// [ERR] Parameter count
                                },

                                "return" => {
                                    //
                                    // Returns the given (value:1)

                                    if args.len() == 1 {
                                        Some(Ok(args[0].clone()))

                                    } else {
                                        Some(Err(self.invalid_param_count(1, args.len(), n)))
                                    }// [ERR] Parameter count
                                },

                                "define" => {
                                    //
                                    // Adds to the function scope a new function with the name (word:1),
                                    // parameter signature ((tuple *(tuple (word, type))):2),
                                    // and body (quote:3).

                                    if with.len() == 3 {
                                        let raw_1 = &args[0];
                                        let raw_2 = &args[1];
                                        let raw_3 = &args[2];// [TODO] Substitute into locally defined functions

                                        if let NodeKind::Word(def_name) | NodeKind::Symbol(def_name) = &raw_1.node {
                                            if let NodeKind::List(raw_lst) = &raw_2.node {
                                                if let NodeKind::Quote{ target, with: params } = &raw_3.node {
                                                    let body_invoke = NodeWrapper::new_invoke(target.to_owned(), params.to_vec(), raw_3.hooks.clone(), raw_3.index, raw_3.position);

                                                    Some(self.define_function(&def_name, raw_lst.to_vec(), body_invoke))

                                                } else {
                                                    Some(Err(self.param_type("quote", raw_3.index, raw_3.position)))
                                                }// [ERR] 3rd parameter type

                                            } else {
                                                Some(Err(self.param_type("(list *(list (word type)))", raw_2.index, raw_2.position)))
                                            }// [ERR] 2nd parameter type

                                        } else {
                                            Some(Err(self.param_type("(word | symbol)", raw_1.index, raw_1.position)))
                                        }// [ERR] 1st parameter type

                                    } else {
                                        Some(Err(self.invalid_param_count(3, args.len(), n)))
                                    }// [ERR] Parameter count
                                },

                                "obj-append" => {
                                    //
                                    // Appends (list:2) to the contents of (obj:1)

                                    if args.len() == 2 {
                                        let obj = &args[0];
                                        let lst_node = &args[1];

                                        if let NodeKind::List(lst) = &lst_node.node {
                                            Some(self.obj_append(obj, lst.to_vec()))

                                        } else {
                                            Some(Err(self.param_type("list", lst_node.index, lst_node.position)))
                                        }// [ERR] Parameter type

                                    } else {
                                        Some(Err(self.invalid_param_count(2, with.len(), n)))
                                    }// [ERR] Parameter count
                                },

                                "length" => {
                                    //
                                    // Returns (int), which is the length of (tuple:1)

                                    if args.len() == 1 {
                                        let lst_node = &args[0];

                                        if let NodeKind::List(lst) = &lst_node.node {
                                            Some(Ok(NodeWrapper::new_int(lst.len() as i64, n.index, n.position)))

                                        } else {
                                            Some(Err(self.param_type("(list (any))", lst_node.index, lst_node.position)))
                                        }// [ERR] Parameter type

                                    } else {
                                        Some(Err(self.invalid_param_count(1, args.len(), n)))
                                    }// [ERR] Parameter count
                                },

                                "head" => {
                                    //
                                    // Returns the first element of (list:1)

                                    if args.len() == 1 {
                                        let lst_node = &args[0];

                                        if let NodeKind::List(lst) = &lst_node.node {
                                            if lst.len() > 0 {
                                                Some(Ok(lst[0].clone()))

                                            } else {
                                                Some(Err(MifulError::runtime_error("Cannot get head of empty tuple!", &self.owned_text, lst_node.index, lst_node.position)))
                                            }// [ERR] Head of empty tuple

                                        } else {
                                            Some(Err(self.param_type("(list (any))", lst_node.index, lst_node.position)))
                                        }// [ERR] Parameter type

                                    } else {
                                        Some(Err(self.invalid_param_count(1, args.len(), n)))
                                    }// [ERR] Parameter count
                                },

                                // [TODO] Resolve hooks (distribution between head and tail + indexing)
                                //
                                "tail" => {
                                    //
                                    // Returns the (list:1) without head.

                                    if args.len() == 1 {
                                        let lst_node = &args[0];

                                        if let NodeKind::List(lst) = &lst_node.node {
                                            if lst.len() > 0 {
                                                let (_, tail) = lst.split_first().unwrap();

                                                Some(Ok(NodeWrapper::new_list(tail.to_vec(), lst_node.hooks.clone(), n.index, n.position)))

                                            } else {
                                                Some(Err(MifulError::runtime_error("Cannot get tail of empty tuple!", &self.owned_text, lst_node.index, lst_node.position)))
                                            }// [ERR] Head of empty tuple

                                        } else {
                                            Some(Err(self.param_type("(list (any))", lst_node.index, lst_node.position)))
                                        }// [ERR] Parameter type

                                    } else {
                                        Some(Err(self.invalid_param_count(1, args.len(), n)))
                                    }// [ERR] Parameter count
                                },

                                "reverse" => {
                                    //
                                    // Returns the (list:1), reversed.

                                    if args.len() == 1 {
                                        let lst_node = &args[0];

                                        if let NodeKind::List(lst) = &lst_node.node {
                                            let mut mut_lst = lst.clone();

                                            mut_lst.reverse();

                                            Some(Ok(NodeWrapper::new_list(mut_lst, lst_node.hooks.clone(), n.index, n.position)))

                                        } else {
                                            Some(Err(self.param_type("(list (any))", lst_node.index, lst_node.position)))
                                        }// [ERR] Parameter type

                                    } else {
                                        Some(Err(self.invalid_param_count(1, args.len(), n)))
                                    }// [ERR] Parameter count
                                },

                                f_name => {
                                    Some(self.call_function(f_name, args, &n))
                                },
                            }
                        },

                        Err(e) => {
                            Some(Err(self.param_eval(e)))
                        }// [ERR] Param eval
                    }
                },

                NodeKind::Quote{ target, with } => {
                    let mut new_with = vec![];

                    for arg in with {
                        let new_arg = self.resolve_hooks(arg, &hooks);

                        new_with.push(new_arg);
                    }

                    Some(Ok(NodeWrapper::new_quote(target, new_with, vec![], n.index, n.position)))
                },// [TODO] Inline local function calls

                // NodeKind::LambdaHook(n) => {
                //     //
                // },

                _ => { None },// [FIXME] Add the rest.
            }
        }
    }
}
