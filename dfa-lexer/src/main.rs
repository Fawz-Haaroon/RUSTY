use std::collections::HashMap;
use std::io::{self, Read, Write};

#[cfg(unix)]
fn stdin_is_tty() -> bool {
    unsafe { libc::isatty(libc::STDIN_FILENO) == 1 }
}

#[cfg(not(unix))]
fn stdin_is_tty() -> bool {
    false
}

#[derive(Debug)]
enum TokenKind {
    Ident(String),
    Number(i64),
    Operator(String),
    LParen,
    RParen,
    Comma,
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
            ' ' | '\n' | '\t' | '\r' => i += 1,

            ',' => {
                tokens.push(Token { kind: TokenKind::Comma });
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
                tokens.push(Token { kind: TokenKind::Ident(text) });
            }

            '0'..='9' => {
                let start = i;
                while i < chars.len() && matches!(chars[i], '0'..='9') {
                    i += 1;
                }
                let text: String = chars[start..i].iter().collect();
                let value = text.parse::<i64>().map_err(|_| "invalid number")?;
                tokens.push(Token { kind: TokenKind::Number(value) });
            }

            '(' => { tokens.push(Token { kind: TokenKind::LParen }); i += 1; }
            ')' => { tokens.push(Token { kind: TokenKind::RParen }); i += 1; }

            '+' | '-' | '*' | '/' => {
                tokens.push(Token { kind: TokenKind::Operator(chars[i].to_string()) });
                i += 1;
            }

            _ => return Err(format!("invalid character '{}'", chars[i])),
        }
    }

    Ok(tokens)
}

#[derive(Debug)]
enum Expr {
    Number(i64),
    Ident(String),

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
        if self.pos >= self.tokens.len() { return None; }
        let token = std::mem::replace(&mut self.tokens[self.pos].kind, TokenKind::Comma);
        self.pos += 1;
        Some(token)
    }

    fn parse_expression(&mut self) -> Result<Expr, String> {
        let left = match self.next() {
            Some(TokenKind::Number(n)) => Expr::Number(n),
            Some(TokenKind::Ident(s)) => Expr::Ident(s),
            _ => return Err("bad expr".into()),
        };

        if let Some(TokenKind::Operator(op)) = self.peek() {
            let op = op.clone();
            self.next();

            let right = match self.next() {
                Some(TokenKind::Number(n)) => Expr::Number(n),
                Some(TokenKind::Ident(s)) => Expr::Ident(s),
                _ => return Err("bad rhs".into()),
            };

            return Ok(Expr::Binary {
                op,
                left: Box::new(left),
                right: Box::new(right),
            });
        }

        Ok(left)
    }
}

fn eval(expr: &Expr, env: &mut HashMap<String, i64>) -> Result<i64, String> {
    match expr {
        Expr::Number(n) => Ok(*n),

        Expr::Ident(name) => env.get(name).copied()
            .ok_or_else(|| format!("undefined '{}'", name)),

        Expr::Binary { op, left, right } => {
            let l = eval(left, env)?;
            let r = eval(right, env)?;

            match op.as_str() {
                "+" => Ok(l + r),
                "-" => Ok(l - r),
                "*" => Ok(l * r),
                "/" => {
                    if r == 0 { return Err("div by zero".into()); }
                    Ok(l / r)
                }
                _ => Err("bad op".into()),
            }
        }
    }
}

fn repl(env: &mut HashMap<String, i64>) {
    let stdin = io::stdin();
    let mut line = String::new();

    loop {
        print!("> ");
        io::stdout().flush().unwrap();

        line.clear();
        if stdin.read_line(&mut line).unwrap() == 0 {
            break;
        }

        let line = line.trim();
        if line == "quit" { break; }

        let tokens = match tokenize(line) {
            Ok(t) => t,
            Err(e) => { eprintln!("{}", e); continue; }
        };

        let mut parser = Parser::new(tokens);

        let expr = match parser.parse_expression() {
            Ok(e) => e,
            Err(e) => { eprintln!("{}", e); continue; }
        };

        match eval(&expr, env) {
            Ok(v) => println!("{}", v),
            Err(e) => eprintln!("{}", e),
        }
    }
}

fn main() {
    let mut env = HashMap::new();

    if stdin_is_tty() {
        repl(&mut env);
        return;
    }

    let mut input = String::new();
    io::stdin().read_to_string(&mut input).unwrap();

    for line in input.lines() {
        if line.trim().is_empty() { continue; }

        let tokens = match tokenize(line) {
            Ok(t) => t,
            Err(e) => { eprintln!("{}", e); break; }
        };

        let mut parser = Parser::new(tokens);

        let expr = match parser.parse_expression() {
            Ok(e) => e,
            Err(e) => { eprintln!("{}", e); break; }
        };

        match eval(&expr, &mut env) {
            Ok(v) => println!("{}", v),
            Err(e) => { eprintln!("{}", e); break; }
        }
    }
}
