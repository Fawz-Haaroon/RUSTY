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
    [State::Ident, State::Number, State::Operator, State::LParen, State::RParen, State::Start, State::Error],
    [State::Ident, State::Ident, State::Error, State::Error, State::Error, State::Error, State::Error],
    [State::Error, State::Number, State::Error, State::Error, State::Error, State::Error, State::Error],
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

enum TokenKind {
    Ident,
    Number,
    Operator(char),
    LParen,
    RParen,
}

struct Token {
    kind: TokenKind,
    start: usize,
    end: usize,
}

fn main() {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input).unwrap();

    let chars: Vec<char> = input.chars().collect();
    let mut i = 0;

    let mut state = State::Start;
    let mut start = 0;
    let mut tokens: Vec<Token> = Vec::new();

    while i < chars.len() {
        let next = TRANSITIONS[state_index(state)][class_index(classify(chars[i]))];

        if next == State::Error {
            match state {
                State::Ident => tokens.push(Token { kind: TokenKind::Ident, start, end: i }),
                State::Number => tokens.push(Token { kind: TokenKind::Number, start, end: i }),
                State::Operator => tokens.push(Token {
                    kind: TokenKind::Operator(chars[start]),
                    start,
                    end: start + 1,
                }),
                State::LParen => tokens.push(Token { kind: TokenKind::LParen, start, end: start + 1 }),
                State::RParen => tokens.push(Token { kind: TokenKind::RParen, start, end: start + 1 }),
                State::Start => {
                    i += 1;
                }
                State::Error => unreachable!(),
            }
            state = State::Start;
            start = i;
            continue;
        }

        if state == State::Start && next != State::Start {
            start = i;
        }

        state = next;
        i += 1;
    }

    if state != State::Start {
        match state {
            State::Ident => tokens.push(Token { kind: TokenKind::Ident, start, end: i }),
            State::Number => tokens.push(Token { kind: TokenKind::Number, start, end: i }),
            State::Operator => tokens.push(Token {
                kind: TokenKind::Operator(chars[start]),
                start,
                end: start + 1,
            }),
            State::LParen => tokens.push(Token { kind: TokenKind::LParen, start, end: start + 1 }),
            State::RParen => tokens.push(Token { kind: TokenKind::RParen, start, end: start + 1 }),
            _ => {}
        }
    }

    for t in tokens {
        match t.kind {
            TokenKind::Ident => println!("IDENT   [{}..{}]", t.start, t.end),
            TokenKind::Number => println!("NUMBER  [{}..{}]", t.start, t.end),
            TokenKind::Operator(op) => println!("OP '{}' [{}..{}]", op, t.start, t.end),
            TokenKind::LParen => println!("LPAREN  [{}..{}]", t.start, t.end),
            TokenKind::RParen => println!("RPAREN  [{}..{}]", t.start, t.end),
        }
    }
}
