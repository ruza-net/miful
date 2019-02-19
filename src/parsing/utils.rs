use parsing::token::Token;


pub fn debug_tokens(tokens: &Vec<Token>) {
    for tok in tokens {
        println!("{:?} : {:?}", tok.kind, tok.value);
    }

    println!("\n");
}
