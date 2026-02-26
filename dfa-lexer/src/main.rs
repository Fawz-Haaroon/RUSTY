use std::io::{self, Read};

#[derive(Copy, Clone, PartialEq)]
enum State {
    Start,
    Ident,
    Number,
    Operator,
    LParen,
    RParen,
    Error,
}

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
    s as usize
}

fn class_index(c: Class) -> usize {
    match c {
        Class::Letter => 0,
        Class::Digit => 1,
        Class::Operator => 2,
        Class::LParen => 3,
        Class::RParen => 4,
        Class::Whitespace => 5,
        Class::Other => 6,
    }
}

enum Keyword {
    If,
    Else,
    While,
    Return,
}

enum TokenKind {
    Ident,
    Number,
    Keyword(Keyword),
    Operator(char),
    LParen,
    RParen,
}

struct Token {
    kind: TokenKind,
    start: usize,
    end: usize,
}

fn classify_keyword(s: &str) -> Option<Keyword> {
    match s {
        "if" => Some(Keyword::If),
        "else" => Some(Keyword::Else),
        "while" => Some(Keyword::While),
        "return" => Some(Keyword::Return),
        _ => None,
    }
}

fn tokenize(input: &str) -> Result<Vec<Token>, String> {
    let chars: Vec<char> = input.chars().collect();
    let mut i = 0;

    let mut state = State::Start;
    let mut start = 0;
    let mut tokens: Vec<Token> = Vec::new();

    while i < chars.len() {
        let next = TRANSITIONS[state_index(state)][class_index(classify(chars[i]))];

        if next == State::Error {
            match state {
                State::Ident => {
                    let text: String = chars[start..i].iter().collect();
                    let kind = match classify_keyword(&text) {
                        Some(k) => TokenKind::Keyword(k),
                        None => TokenKind::Ident,
                    };
                    tokens.push(Token { kind, start, end: i });
                }
                State::Number => tokens.push(Token { kind: TokenKind::Number, start, end: i }),
                State::Operator => tokens.push(Token {
                    kind: TokenKind::Operator(chars[start]),
                    start,
                    end: start + 1,
                }),
                State::LParen => tokens.push(Token { kind: TokenKind::LParen, start, end: start + 1 }),
                State::RParen => tokens.push(Token { kind: TokenKind::RParen, start, end: start + 1 }),
                State::Start | State::Error => {
                    return Err(format!("invalid character '{}' at position {}", chars[i], i));
                }
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
            State::Ident => {
                let text: String = chars[start..i].iter().collect();
                let kind = match classify_keyword(&text) {
                    Some(k) => TokenKind::Keyword(k),
                    None => TokenKind::Ident,
                };
                tokens.push(Token { kind, start, end: i });
            }
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

    Ok(tokens)
}

fn main() {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input).unwrap();

    match tokenize(&input) {
        Ok(tokens) => {
            for t in tokens {
                let lexeme = &input[t.start..t.end];
                println!("[{}..{}] {}", t.start, t.end, lexeme);
            }
        }
        Err(e) => {
            eprintln!("{}", e);
        }
    }
}
