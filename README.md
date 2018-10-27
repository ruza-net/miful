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
