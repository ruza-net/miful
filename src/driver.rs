use parsing;

/*

# Miful Default Driver

aka MD2

## Functions

MD2 implements these global functions:

* if (value) {block:1} {block:2}
    > runs {block:1} if (value) returns sym(&) and runs {block:2} otherwise
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
    > rounds (float) down

* ceil (float)
    > rounds (float) up

* round (float)
    > rounds (float) to the nearest integer

* let (word) (value) {block}
    > binds (value) to (word), every call to [: (word)] inside {block} will result into (value)

* define (word) (list<>)

*/


pub struct Driver<'a> {
    parser: parsing::parser::Parser,
    lexer: parsing::lexer::Lexer<'a>
}

impl<'a> Driver<'a> {
    pub fn new(input: &'a str) -> Driver<'a> {
        let symbols = vec![
            r"&",
            r"\^",
            r"~",
            r"|",
            r"\\"
        ];

        Driver {
            parser: parsing::parser::Parser::empty(),
            lexer: parsing::lexer::Lexer::new(input, symbols)
        }
    }

    pub fn process(&mut self) {
        let tokens = self.lexer.read_all_tokens();

        self.parser = parsing::parser::Parser::new(tokens.to_vec());

        let ast = self.parser.construct_tree();

        for node in ast {
            println!("{}\n", node);
        }
    }
}
