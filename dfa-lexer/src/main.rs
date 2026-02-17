use std::io::{self, Read};

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

#[derive(Copy, Clone, Debug)]
enum Class {
    Letter,
    Digit,
    Operator,
    LParen,
    RParen,
    Whitespace,
    Other,
}

fn classify(c: char) -> Class {
    match c {
        'a'..='z' | 'A'..='Z' | '_' => Class::Letter,
        '0'..='9' => Class::Digit,
        '+' | '-' | '*' | '/' | '=' => Class::Operator,
        '(' => Class::LParen,
        ')' => Class::RParen,
        ' ' | '\n' | '\t' | '\r' => Class::Whitespace,
        _ => Class::Other,
    }
}

const STATE_COUNT: usize = 7;
const CLASS_COUNT: usize = 7;

const S_START: usize = 0;
const S_IDENT: usize = 1;
const S_NUMBER: usize = 2;
const S_OPERATOR: usize = 3;
const S_LPAREN: usize = 4;
const S_RPAREN: usize = 5;
const S_ERROR: usize = 6;

const C_LETTER: usize = 0;
const C_DIGIT: usize = 1;
const C_OPERATOR: usize = 2;
const C_LPAREN: usize = 3;
const C_RPAREN: usize = 4;
const C_WHITESPACE: usize = 5;
const C_OTHER: usize = 6;

const TRANSITIONS: [[State; CLASS_COUNT]; STATE_COUNT] = [
    [
        State::Ident,
        State::Number,
        State::Operator,
        State::LParen,
        State::RParen,
        State::Start,
        State::Error,
    ],
    [
        State::Ident,
        State::Ident,
        State::Error,
        State::Error,
        State::Error,
        State::Error,
        State::Error,
    ],
    [
        State::Error,
        State::Number,
        State::Error,
        State::Error,
        State::Error,
        State::Error,
        State::Error,
    ],
    [State::Error; CLASS_COUNT],
    [State::Error; CLASS_COUNT],
    [State::Error; CLASS_COUNT],
    [State::Error; CLASS_COUNT],
];

fn state_index(s: State) -> usize {
    match s {
        State::Start => S_START,
        State::Ident => S_IDENT,
        State::Number => S_NUMBER,
        State::Operator => S_OPERATOR,
        State::LParen => S_LPAREN,
        State::RParen => S_RPAREN,
        State::Error => S_ERROR,
    }
}

fn class_index(c: Class) -> usize {
    match c {
        Class::Letter => C_LETTER,
        Class::Digit => C_DIGIT,
        Class::Operator => C_OPERATOR,
        Class::LParen => C_LPAREN,
        Class::RParen => C_RPAREN,
        Class::Whitespace => C_WHITESPACE,
        Class::Other => C_OTHER,
    }
}

#[derive(Debug)]
enum TokenKind {
    Ident,
    Number,
    Operator(char),
    LParen,
    RParen,
}

#[derive(Debug)]
struct Token {
    kind: TokenKind,
    start: usize,
    end: usize,
}

fn main() {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input).unwrap();
}
