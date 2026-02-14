#[derive(Copy, Clone, Debug, PartialEq)]
enum State {
    Start,
    Ident,
    Number,
    Operator,
    LParen,
    RParen,
    Error,
}

use std::io::{self, Read};

fn main() {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input).unwrap();
}
