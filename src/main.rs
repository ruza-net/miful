#[macro_use]
extern crate text_io;
extern crate unicode_segmentation;

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
            {println
                [input [string (Enter [:space] a [:space] message: [:space])]]
            }
            {println
                No
            }
        }
    ]";

    let test = "
    [define drop ((n int) (lst list))
        {if [= [:n] 0]
            {:lst}
            {drop [- [:n] 1] [tail [:lst]]}
        }
    ]

    [define elem ((n int) (lst list)) {head [drop [:n] [:lst]]}]
    [define obj-unwrap ((o (obj any))) {elem 2 [:o]}]

    [define string ((seq (list (word symbol)))) {return (`_obj` string [:seq])}]
    [define string ((num (int | float))) {return (`_obj` string ([mk-sym [:num]]))}]

    [define println ((val word)) {print [string ([:val] [:newline])]}]
    [define println ((val symbol)) {print [string ([:val] [:newline])]}]
    [define println ((val (obj string))) {print [obj-append [:val] ([:newline])]}]

    [define factorial ((n int))
        {if [= [:n] 0]
            {return 1}
            {* [:n] [factorial [- [:n] 1]]}
        }
    ]

    [println
        [obj-append
            [string (Hello: [:space])]

            ([input
                [string
                    (Enter [:space] your [:space] name: [:space])
                ]
            ])
        ]
    ]

    [println
        [string
            [length
                [obj-unwrap
                    [input
                        [string ([:l_bracket] input [:r_bracket] [:space])]
                    ]
                ]
            ]
        ]
    ]

    [println [string [factorial 12]]]
    ";

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
