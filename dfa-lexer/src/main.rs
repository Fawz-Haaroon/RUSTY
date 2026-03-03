use std::collections::HashMap;
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
        '+' | '-' | '*' | '/' | '=' | '<' | '>' | '!' => Class::Operator,
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
    Operator(String),
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
    let mut tokens = Vec::new();

    while i < chars.len() {
        match chars[i] {
            ' ' | '\n' | '\t' | '\r' => {
                i += 1;
            }

            'a'..='z' | 'A'..='Z' | '_' => {
                let start = i;
                while i < chars.len()
                    && matches!(chars[i], 'a'..='z' | 'A'..='Z' | '_' | '0'..='9')
                {
                    i += 1;
                }

                let text: String = chars[start..i].iter().collect();
                tokens.push(Token {
                    kind: TokenKind::Ident(text),
                });
            }

            '0'..='9' => {
                let start = i;
                while i < chars.len() && matches!(chars[i], '0'..='9') {
                    i += 1;
                }

                let text: String = chars[start..i].iter().collect();
                let value = text.parse::<i64>().map_err(|_| "invalid number")?;

                tokens.push(Token {
                    kind: TokenKind::Number(value),
                });
            }

            '(' => {
                tokens.push(Token {
                    kind: TokenKind::LParen,
                });
                i += 1;
            }

            ')' => {
                tokens.push(Token {
                    kind: TokenKind::RParen,
                });
                i += 1;
            }

            '+' | '-' | '*' | '/' => {
                tokens.push(Token {
                    kind: TokenKind::Operator(chars[i].to_string()),
                });
                i += 1;
            }

            '=' => {
                if i + 1 < chars.len() && chars[i + 1] == '=' {
                    tokens.push(Token {
                        kind: TokenKind::Operator("==".into()),
                    });
                    i += 2;
                } else {
                    tokens.push(Token {
                        kind: TokenKind::Operator("=".into()),
                    });
                    i += 1;
                }
            }

            '!' => {
                if i + 1 < chars.len() && chars[i + 1] == '=' {
                    tokens.push(Token {
                        kind: TokenKind::Operator("!=".into()),
                    });
                    i += 2;
                } else {
                    return Err("unexpected '!'".into());
                }
            }

            '<' => {
                if i + 1 < chars.len() && chars[i + 1] == '=' {
                    tokens.push(Token {
                        kind: TokenKind::Operator("<=".into()),
                    });
                    i += 2;
                } else {
                    tokens.push(Token {
                        kind: TokenKind::Operator("<".into()),
                    });
                    i += 1;
                }
            }

            '>' => {
                if i + 1 < chars.len() && chars[i + 1] == '=' {
                    tokens.push(Token {
                        kind: TokenKind::Operator(">=".into()),
                    });
                    i += 2;
                } else {
                    tokens.push(Token {
                        kind: TokenKind::Operator(">".into()),
                    });
                    i += 1;
                }
            }

            _ => {
                return Err(format!("invalid character '{}'", chars[i]));
            }
        }
    }

    Ok(tokens)
}

#[derive(Debug)]
enum Expr {
    Number(i64),
    Ident(String),

    Unary {
        op: String,
        expr: Box<Expr>,
    },

    Binary {
        op: String,
        left: Box<Expr>,
        right: Box<Expr>,
    },
}

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

            Some(TokenKind::Operator(op)) if op == "-" || op == "+" => {
                let expr = self.parse_expression(30)?;
                Expr::Unary {
                    op,
                    expr: Box::new(expr),
                }
            }

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
                Some(TokenKind::Operator(op)) => op.clone(),
                _ => break,
            };

            let (l_bp, r_bp) = match op.as_str() {
                "=" => (1, 0),
                "==" | "!=" => (3, 4),
                "<" | ">" | "<=" | ">=" => (5, 6),
                "+" | "-" => (10, 11),
                "*" | "/" => (20, 21),
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

fn eval(expr: &Expr, env: &mut HashMap<String, i64>) -> Result<i64, String> {
    match expr {
        Expr::Number(n) => Ok(*n),

        Expr::Ident(name) => env
            .get(name)
            .copied()
            .ok_or_else(|| format!("undefined variable '{}'", name)),

        Expr::Unary { op, expr } => {
            let v = eval(expr, env)?;

            match op.as_str() {
                "-" => Ok(-v),
                "+" => Ok(v),
                _ => Err("unknown unary operator".into()),
            }
        }

        Expr::Binary { op, left, right } => {
            if op == "=" {
                if let Expr::Ident(name) = &**left {
                    let value = eval(right, env)?;
                    env.insert(name.clone(), value);
                    return Ok(value);
                } else {
                    return Err("left side of assignment must be identifier".into());
                }
            }

            let l = eval(left, env)?;
            let r = eval(right, env)?;

            match op.as_str() {
                "+" => Ok(l + r),
                "-" => Ok(l - r),
                "*" => Ok(l * r),
                "/" => Ok(l / r),

                "<" => Ok((l < r) as i64),
                ">" => Ok((l > r) as i64),
                "<=" => Ok((l <= r) as i64),
                ">=" => Ok((l >= r) as i64),

                "==" => Ok((l == r) as i64),
                "!=" => Ok((l != r) as i64),

                _ => Err("unknown operator".into()),
            }
        }
    }
}

fn main() {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input).unwrap();

    let mut env: HashMap<String, i64> = HashMap::new();

    for line in input.lines() {
        if line.trim().is_empty() {
            continue;
        }

        match tokenize(line) {
            Ok(tokens) => {
                let mut parser = Parser::new(tokens);

                match parser.parse_expression(0) {
                    Ok(expr) => match eval(&expr, &mut env) {
                        Ok(value) => println!("{}", value),
                        Err(e) => {
                            eprintln!("runtime error: {}", e);
                            break;
                        }
                    },

                    Err(e) => {
                        eprintln!("parse error: {}", e);
                        break;
                    }
                }
            }

            Err(e) => {
                eprintln!("lex error: {}", e);
                break;
            }
        }
    }
}
