use std::io::{self, Read};

/*
// LEXER
*/

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

#[derive(Debug)]
enum TokenKind {
    Ident(String),
    Number(i64),
    Operator(char),
    LParen,
    RParen,
}

#[derive(Debug)]
struct Token {
    kind: TokenKind,
}

fn tokenize(input: &str) -> Result<Vec<Token>, String> {
    let chars: Vec<char> = input.chars().collect();
    let mut i = 0;
    let mut state = State::Start;
    let mut start = 0;
    let mut tokens = Vec::new();

    while i < chars.len() {
        let next = TRANSITIONS[state_index(state)][class_index(classify(chars[i]))];

        if next == State::Error {
            match state {
                State::Ident => {
                    let text: String = chars[start..i].iter().collect();
                    tokens.push(Token { kind: TokenKind::Ident(text) });
                }
                State::Number => {
                    let text: String = chars[start..i].iter().collect();
                    let value = text.parse::<i64>().map_err(|_| "invalid number")?;
                    tokens.push(Token { kind: TokenKind::Number(value) });
                }
                State::Operator => {
                    tokens.push(Token { kind: TokenKind::Operator(chars[start]) });
                }
                State::LParen => tokens.push(Token { kind: TokenKind::LParen }),
                State::RParen => tokens.push(Token { kind: TokenKind::RParen }),
                State::Start | State::Error => {
                    return Err(format!("invalid character '{}' at {}", chars[i], i));
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
                tokens.push(Token { kind: TokenKind::Ident(text) });
            }
            State::Number => {
                let text: String = chars[start..i].iter().collect();
                let value = text.parse::<i64>().map_err(|_| "invalid number")?;
                tokens.push(Token { kind: TokenKind::Number(value) });
            }
            State::Operator => {
                tokens.push(Token { kind: TokenKind::Operator(chars[start]) });
            }
            State::LParen => tokens.push(Token { kind: TokenKind::LParen }),
            State::RParen => tokens.push(Token { kind: TokenKind::RParen }),
            _ => {}
        }
    }

    Ok(tokens)
}

/*
// AST
*/

#[derive(Debug)]
enum Expr {
    Number(i64),
    Ident(String),
    Binary {
        op: char,
        left: Box<Expr>,
        right: Box<Expr>,
    },
}

/*
// PARSER
*/

struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, pos: 0 }
    }

    fn peek(&self) -> Option<&TokenKind> {
        self.tokens.get(self.pos).map(|t| &t.kind)
    }

    fn next(&mut self) -> Option<TokenKind> {
        if self.pos >= self.tokens.len() {
            return None;
        }

        let token = std::mem::replace(
            &mut self.tokens[self.pos].kind,
            TokenKind::LParen,
        );

        self.pos += 1;
        Some(token)
    }

    fn parse_expression(&mut self, min_bp: u8) -> Result<Expr, String> {
        let mut left = match self.next() {
            Some(TokenKind::Number(n)) => Expr::Number(n),
            Some(TokenKind::Ident(s)) => Expr::Ident(s),
            Some(TokenKind::LParen) => {
                let expr = self.parse_expression(0)?;
                match self.next() {
                    Some(TokenKind::RParen) => expr,
                    _ => return Err("expected ')'".into()),
                }
            }
            _ => return Err("unexpected token".into()),
        };

        loop {
            let op = match self.peek() {
                Some(TokenKind::Operator(op)) => *op,
                _ => break,
            };

            let (l_bp, r_bp) = match op {
                '=' => (1, 0),             // right-associative
                '+' | '-' => (10, 11),
                '*' | '/' => (20, 21),
                _ => break,
            };

            if l_bp < min_bp {
                break;
            }

            self.next();

            let right = self.parse_expression(r_bp)?;

            left = Expr::Binary {
                op,
                left: Box::new(left),
                right: Box::new(right),
            };
        }

        Ok(left)
    }
}

/*
// DEBUG PRINT
*/

fn print_expr(expr: &Expr, indent: usize) {
    let pad = "  ".repeat(indent);

    match expr {
        Expr::Number(n) => println!("{}Number({})", pad, n),
        Expr::Ident(s) => println!("{}Ident({})", pad, s),
        Expr::Binary { op, left, right } => {
            println!("{}Binary({})", pad, op);
            print_expr(left, indent + 1);
            print_expr(right, indent + 1);
        }
    }
}

/*
// MAIN
*/

fn main() {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input).unwrap();

    match tokenize(&input) {
        Ok(tokens) => {
            let mut parser = Parser::new(tokens);
            match parser.parse_expression(0) {
                Ok(expr) => print_expr(&expr, 0),
                Err(e) => eprintln!("parse error: {}", e),
            }
        }
        Err(e) => eprintln!("lex error: {}", e),
    }
}
