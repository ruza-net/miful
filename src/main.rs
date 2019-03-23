#[macro_use]
mod parsing;
mod driver;

use parsing::utils::{ MifulError, ParseError, SemanticError };


fn main() {
    let test_long =
    "[define
        foo
        {if [> [first (1 2 3)] [last (1 2 3)]]
            {print
                [string [input [string (Enter a message:) \\space]] \\space]
            }
            {print
                [string No]
            }
        }
    ]";

    let test = "[print 2]";

    let mut driver = driver::Driver::new(test);

    let result = driver.process();

    match result {
        Ok(ret) => {
            println!("{}", ret);
        },

        Err(MifulError::Parsing(e)) => {
            e.throw_err();
        },

        Err(MifulError::Semantics(e)) => {
            e.throw_err("[TODO:context missing]".to_owned());
        },
    }
}
