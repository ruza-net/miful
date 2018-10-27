use parsing::ast as ast;

use parsing::token as tok;


struct Parser<'a, T> {
    tokens: Vec<tok::Token>,
    nodes: Vec<ast::InvokeNode<'a, T>>
}
