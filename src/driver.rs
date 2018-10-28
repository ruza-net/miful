use parsing;

/*

# Miful Default Driver

aka MD2

## Functions

MD2 implements these global functions:

* if (value) (invoke:1) (invoke:2)
    > runs (invoke:1) if (value) returns sym(&) and runs (invoke:2) otherwise
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

*/


struct Driver {
    parser: parsing::parser::Parser,
}
