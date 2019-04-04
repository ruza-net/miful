#[macro_use]
mod parsing;
mod driver;

use parsing::utils::Error;


fn main() {
    let test_long =
    "[define
        foo
        ()
        {if [> [first (1 2 3)] [last (1 2 3)]]
            {print
                [input [string (Enter [: space] a [: space] message:)]]
            }
            {print
                No
            }
        }
    ]";

    let test = "
    [define println ((val word)) {print (`_obj` string ([:val] [:newline]))}]
    [define println ((val symbol)) {print (`_obj` string ([:val] [:newline]))}]
    [define println ((val (obj string))) {print [obj-append val ([:newline])]}]

    [print (`_obj` string (Hello [:space] world! [:newline]))]";

    let mut driver = driver::Driver::new(test);

    let may_err = driver.process();

    if let Err(e) = may_err {
        e.throw_err();
    }

    let result: Result<Vec<_>, _> = driver.collect();

    match result {
        Ok(ret) => {
            for v in ret {
                println!("[{}]: ` {} `", v.node, v);
            }
        },

        Err(e) => {
            e.throw_err();// [TODO] Maybe don't panic?
        },
    }
}
