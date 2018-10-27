# Miful

Miniature Functional Language

## About

I created Miful as a part of my school project, and I intended it to be a scripting language for my command-line RPG game.

## Syntax

Miful is LISP-like. It knows exactly 6 syntax elements:

* word
* symbol
* float
* int
* list *(of any number of any of these types)*
* function invocation *(with a single name and any number of parameters)*

An example of Miful code would be:
```
[define factorial (num)
    [if [= num 0]
        0
        [* [factorial [- num 1]] num]
    ]
]
```

## Integration

Miful itself is indeed just a language specification (provided with a lexer and parser), and it needs a driver which provides some functionality (like managing functions, constants, built-in functions like conditions, ...).

This repository contains a default driver which provides conditions, function and constant management, arithmetic, and other basic functionalities. However, you can create your own driver in order to adjust Miful to your needs. To do this, either you need to fork this repository and modify the driver, or you can clone the `parsing` directory and connect it directly to your driver.

What you want to do is read the source code, initialize the lexer via `Lexer::new`, call `lexer.read_all_tokens`, and then give the result to `Parser::new`. The output AST for your driver results from the method `parser.construct_tree`.
