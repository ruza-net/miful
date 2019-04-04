use parsing;
use parsing::ast::{ NodeWrapper, NodeKind, MifulType };
use parsing::token as tok;
use parsing::utils::{ MifulError, segment_text };

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


pub struct Driver<'a, 'b> {
    input: &'a str,
    owned_text: Vec<String>,

    symbols: HashSet<&'b str>,

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


impl<'a, 'b> Driver<'a, 'b> {
    pub fn new(input: &'a str) -> Driver<'a, 'b> {
        Driver {
            input,
            owned_text: vec![],

            // [NOTE] `?` and `@` are required to be symbols.
            //
            symbols: set![":", "@", "&", "|", "#", "~", "?", "\\"],

            index: 0,
            ast: vec![],

            scope: map!{
                "space".to_owned() => NodeWrapper::new_symbol(" ".to_owned(), 0, (0, 0)),
                "tab".to_owned() => NodeWrapper::new_symbol("\t".to_owned(), 0, (0, 0)),
                "newline".to_owned() => NodeWrapper::new_symbol("\n".to_owned(), 0, (0, 0)),
                "carriage_ret".to_owned() => NodeWrapper::new_symbol("\r".to_owned(), 0, (0, 0)),
            },
            functions: map!{},
        }
    }

    fn over(owned_text: Vec<String>, ast: Vec<NodeWrapper>, scope: HashMap<String, NodeWrapper>,
        functions: HashMap<(String, Vec<MifulType>), (Vec<String>, NodeWrapper)>) -> Driver<'a, 'b> {

            Driver {
                input: "",
                owned_text,

                symbols: set![],

                index: 0,
                ast,

                scope,
                functions,
            }
    }

    pub fn process(&mut self) -> Result<(), MifulError> {
        let symbols = self.symbols.clone();
        let segmented_text = segment_text(self.input);

        self.owned_text = segmented_text.iter().cloned().map(ToOwned::to_owned).collect();

        let lexer = parsing::lexer::Lexer::new(segmented_text, symbols);
        let tokens: Vec<tok::Token> = lexer.collect();

        let parser = parsing::parser::Parser::new(tokens);

        let result: Result<Vec<_>, _> = parser.collect();

        match result {
            Ok(ast) => {
                self.ast = ast;

                Ok(())
            },

            Err(e) => {
                let mut new_e = e;

                new_e.add_layer_top("..while interpreting the source");
                new_e.supply_source(&self.owned_text);

                Err(new_e)
            }
        }
    }

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

    fn param_type(&self, idx: usize, pos: (usize, usize)) -> MifulError {
        MifulError::runtime_error("Invalid parameter type!", &self.owned_text, idx, pos)
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

                        NodeKind::List(_) => { s == "tuple" },

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

            MifulType::List(types) => {
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
        }
    }

    fn check_obj_type(&self, val: &NodeKind, t: &str) -> bool {
        if let NodeKind::List(st) = val {
            if st.len() == 3 {
                if let NodeKind::Symbol(v) = &st[0].node {
                    if let NodeKind::Word(s) | NodeKind::Symbol(s) = &st[1].node {
                        return v == "_obj" && s == t;
                    }
                }
            }
        }

        false
    }

    fn make_object(&self, t: String, val: NodeWrapper) -> NodeWrapper {
        let idx = val.index;
        let pos = val.position;

        let obj_sym = NodeWrapper::new_symbol("_obj".to_owned(), idx, pos);
        let obj_type = NodeWrapper::new_word(t, idx, pos);
        let hooks = val.hooks.clone();

        NodeWrapper::new_list(vec![obj_sym, obj_type, val], hooks, idx, pos)
    }

    fn make_nil(&self) -> NodeWrapper {
        self.make_object("nil".to_owned(), NodeWrapper::new_list(vec![], vec![], 0, (0, 0)))
    }

    fn get_obj_val(&self, obj: &NodeWrapper) -> NodeWrapper {
        if let NodeKind::List(ref obj_struct) = obj.node {
            if obj_struct.len() == 3 {
                obj_struct[2].clone()

            } else {
                panic!("Invalid object structure size!");
            }// [PANIC] Invalid object length

        } else {
            panic!("Invalid object structure kind!");
        }// [PANIC] Invalid object kind
    }

    // [TODO] Resolve hooks.
    //
    fn concat_list(&self, l1: NodeWrapper, l2: NodeWrapper) -> NodeWrapper {
        if let NodeKind::List(a) = l1.node {
            if let NodeKind::List(mut b) = l2.node {
                let mut new_list = a;

                new_list.append(&mut b);

                NodeWrapper::new_list(new_list, vec![], l1.index, l1.position)

            } else {
                panic!("Second concat argument is non-list!");
            }// [PANIC] 2nd parameter is non-list

        } else {
            panic!("First concat argument is non-list!");
        }// [PANIC] 1st parameter is non-list
    }

    fn list_to_type(&self, node: &NodeWrapper) -> Result<Vec<MifulType>, MifulError> {
        if let NodeKind::List(lst) = &node.node {
            let mut types = vec![];

            for val in lst {
                //
            }

            Ok(types)

        } else {
            Err(self.type_signature(node))
        }
    }

    //
    // [END] Type Utils


    // [AREA] Function Utils
    //

    // [NOTE] This function assumes immutability of `self.scope` in its session.
    //
    fn print_fn(&self, val: NodeWrapper) -> Result<NodeWrapper, MifulError> {
        let own_text = self.owned_text.clone();
        let loc_scope = self.scope.clone();
        let loc_functions = self.functions.clone();

        let inner_driver = Driver::over(own_text, vec![val.clone()], loc_scope, loc_functions);
        let result: Result<Vec<_>, _> = inner_driver.collect();

        match result {
            Ok(ret) => {
                let node = ret[0].clone();
                let kind = &node.node;

                if let NodeKind::Word(s) | NodeKind::Symbol(s) = kind {
                    print!("{}", s);

                    Ok(self.make_nil())

                } else if self.check_obj_type(kind, "string") {
                    let obj_struct = self.get_obj_val(&node).node;

                    if let NodeKind::List(lst) = obj_struct {
                        for v in lst {
                            match self.print_fn(v) {//[TODO]
                                Ok(_) => {},

                                Err(e) => {
                                    return Err(self.param_eval(e));
                                },
                            }
                        }

                        Ok(self.make_nil())

                    } else {
                        panic!("Corrupted object structure!");
                    }// [PANIC] Corrupt object

                } else {
                    Err(self.param_type(val.index, val.position))
                }// [ERR] Parameter type
            },

            Err(e) => {
                Err(self.param_eval(e))
            }
        }
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

    fn call_function(&self, name: &str, with: Vec<NodeWrapper>, n: NodeWrapper) -> Result<NodeWrapper, MifulError> {
        let own_text = self.owned_text.clone();

        // Process given arguments.
        //
        let inner_driver = Driver::over(own_text.clone(), with, self.scope.clone(), self.functions.clone());
        let result: Result<Vec<_>, _> = inner_driver.collect();

        match result {
            Ok(params) => {
                for ((f_name, exp_args), (arg_names, body_quote)) in &self.functions {
                    if name == f_name && self.args_compatible(exp_args, &params) {
                        let inner_scope =
                            arg_names.iter()
                                .cloned()
                                .zip(params.iter().cloned())
                                .collect();

                        // [TODO] Local functions?
                        // (from the scope of the function declaration)
                        //
                        // [IDEA] Maybe insert the outer function body when processing quote.
                        //
                        let call_driver = Driver::over(own_text, vec![body_quote.clone()], inner_scope, map!{});
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
                    }
                }

                // [TODO] Maybe print the given types?
                //
                Err(MifulError::runtime_error(&format!("Did not find function ` {} ` with desired parameter types!", name), &own_text, n.index, n.position))
            },

            Err(e) => {
                Err(self.param_eval(e))
            },
        }
    }

    fn define_function(&self, name: &str, raw_signature: NodeWrapper, body: NodeWrapper) -> Result<NodeWrapper, MifulError> {
        match self.list_to_type(&raw_signature) {
            Ok(lst) => {
                //
            },

            Err(e) => {
                let mut new_e = e;

                new_e.add_layer_top("..while defining function");

                Err(new_e)
            }
        }
    }

    //
    // [END] Function Utils
}


impl<'a, 'b> Iterator for Driver<'a, 'a> {
    type Item = Result<NodeWrapper, MifulError>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.ast.len() {
            None

        } else {
            let n = self.ast[self.index].clone();

            let kind = n.node.clone();
            let hooks = n.hooks.clone();

            let own_text = self.owned_text.clone();
            let mut loc_scope = self.scope.clone();
            let mut loc_functions = self.functions.clone();

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
                    match target.as_ref() {
                        "print" => {
                            //
                            // Prints (word:1) or (symbol:1) or ((obj string):1)

                            if with.len() == 1 {
                                Some(self.print_fn(with[0].clone()))

                            } else {
                                Some(Err(self.invalid_param_count(1, with.len(), n)))
                            }// [ERR] Parameter count
                        },

                        "mk-sym" => {
                            //
                            // Creates a symbol from (word:1)

                            if with.len() == 1 {
                                let inner_driver = Driver::over(own_text, with.to_vec(), loc_scope, loc_functions);
                                let result: Result<Vec<_>, _> = inner_driver.collect();

                                match result {
                                    Ok(ret) => {
                                        let ret_node = &ret[0];

                                        if let NodeKind::Word(w) = ret_node.node {
                                            Some(Ok(NodeWrapper::new_symbol(w.to_owned(), n.index, n.position)))

                                        } else {
                                            Some(Err(self.param_type(ret_node.index, ret_node.position)))
                                        }// [ERR] Parameter type
                                    },

                                    Err(e) => {
                                        Some(Err(self.param_eval(e)))
                                    }
                                }

                            } else {
                                Some(Err(self.invalid_param_count(1, with.len(), n)))
                            }// [ERR] Parameter count
                        }

                        // "export" => {
                        //     //
                        // },

                        ":" => {
                            //
                            // Returns the value in scope under the name of (word:1)

                            if with.len() == 1 {
                                let inner_driver = Driver::over(own_text, with.to_vec(), loc_scope, loc_functions);
                                let result: Result<Vec<_>, _> = inner_driver.collect();

                                match result {
                                    Ok(ret) => {
                                        if let NodeKind::Word(v) | NodeKind::Symbol(v) = &ret[0].node {
                                            if let Some(val) = self.scope.get(v) {
                                                Some(Ok(val.clone()))

                                            } else {
                                                Some(Err(MifulError::runtime_error("Undefined constant!", &self.owned_text, n.index, n.position)))
                                            }// [ERR] Undefined constant

                                        } else {
                                            Some(Err(self.param_type(n.index, n.position)))
                                        }// [ERR] Parameter type
                                    },

                                    Err(e) => {
                                        Some(Err(self.param_eval(e)))
                                    },
                                }

                            } else {
                                Some(Err(self.invalid_param_count(1, with.len(), n)))
                            }// [ERR] Parameter count
                        },

                        "define" => {
                            //
                            // Adds to the function scope a new function with the name (word:1),
                            // parameter signature ((tuple *(tuple (word, type))):2),
                            // and body (quote:3)

                            if with.len() == 3 {
                                let raw_1 = with[0];
                                let raw_signature = with[1];
                                let raw_3 = with[2];

                                if let NodeKind::Word(def_name) | NodeKind::Symbol(def_name) = raw_1.node {
                                    if let NodeKind::Quote{ target, with: params } = raw_3.node {
                                        let body_invoke = NodeWrapper::new_invoke(target, params, raw_3.hooks, raw_3.index, raw_3.position);

                                        Some(self.define_function(&def_name, raw_signature, body_invoke))

                                    } else {
                                        Some(Err(self.param_type(raw_3.index, raw_3.position)))
                                    }// [ERR] Parameter type

                                } else {
                                    Some(Err(self.param_type(raw_1.index, raw_1.position)))
                                }// [ERR] Parameter type

                            } else {
                                Some(Err(self.invalid_param_count(3, with.len(), n)))
                            }// [ERR] Parameter count
                        },

                        f_name => {
                            Some(self.call_function(f_name, with, n))
                        },
                    }
                },

                NodeKind::Quote{ target, with } => {
                    let mut new_with = vec![];

                    for arg in with {
                        let new_arg = self.resolve_hooks(arg, &hooks);

                        new_with.push(new_arg);
                    }

                    Some(Ok(NodeWrapper::new_quote(target, new_with, vec![], n.index, n.position)))
                },

                // NodeKind::LambdaHook(n) => {
                //     //
                // },

                _ => { None },// [FIXME] Add the rest.
            }
        }
    }
}
