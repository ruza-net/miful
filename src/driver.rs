use parsing;
use parsing::ast;
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
    symbols: HashSet<&'b str>,

    scope: HashMap<String, ast::NodeWrapper>,// [TODO] Inner scopes?
    functions: HashMap<String, ast::NodeWrapper>,// [TODO] Argument binding
}


impl<'a, 'b> Driver<'a, 'b> {
    pub fn new(input: &'a str) -> Driver<'a, 'b> {
        Driver {
            input,
            symbols: set![":", "@", "&", "|", "#", "~", "?", "\\"],

            scope: HashMap::new(),
            functions: HashMap::new(),
        }
    }

    pub fn process(&mut self) -> Result<ast::NodeWrapper, MifulError> {
        let symbols = self.symbols.clone();
        let segmented_text = segment_text(self.input);
        let owned_text = segmented_text.iter().cloned().map(ToOwned::to_owned).collect();

        let lexer = parsing::lexer::Lexer::new(segmented_text, symbols);
        let tokens: Vec<tok::Token> = lexer.collect();

        let parser = parsing::parser::Parser::new(tokens);

        let result: Result<Vec<_>, _> = parser.collect();

        match result {
            Ok(ast) => {
                self.run(ast)
            },

            Err(e) => {
                let mut new_e = e;

                new_e.add_layer_top("..while interpreting the source");
                new_e.supply_source(&owned_text);

                Err(new_e)
            }
        }
    }

    fn run(&mut self, ast: Vec<ast::NodeWrapper>) -> Result<ast::NodeWrapper, MifulError> {
        for n in ast {
            println!("{}", n);
        }

        Ok(ast::NodeWrapper::new_float(3.14, 0, (0, 0)))// [TODO]
    }
}
