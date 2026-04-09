use std::collections::HashMap;
use std::env;
use std::fs;
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
struct Error {
    msg: String,
    col: usize,
}

impl Error {
    fn new(msg: &str, col: usize) -> Self {
        Self { msg: msg.into(), col }
    }
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
    col: usize,
}

fn tokenize(input: &str) -> Result<Vec<Token>, Error> {
    let chars: Vec<char> = input.chars().collect();
    let mut i = 0;
    let mut tokens = Vec::new();

    while i < chars.len() {
        match chars[i] {
            ' ' | '\t' | '\r' => i += 1,

            ',' => {
                tokens.push(Token { kind: TokenKind::Comma, col: i });
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
                tokens.push(Token { kind: TokenKind::Ident(text), col: start });
            }

            '0'..='9' => {
                let start = i;
                while i < chars.len() && matches!(chars[i], '0'..='9') {
                    i += 1;
                }
                let text: String = chars[start..i].iter().collect();
                let value = text.parse::<i64>().map_err(|_| Error::new("invalid number", start))?;
                tokens.push(Token { kind: TokenKind::Number(value), col: start });
            }

            '(' => { tokens.push(Token { kind: TokenKind::LParen, col: i }); i += 1; }
            ')' => { tokens.push(Token { kind: TokenKind::RParen, col: i }); i += 1; }

            '+' | '-' | '*' | '/' => {
                tokens.push(Token {
                    kind: TokenKind::Operator(chars[i].to_string()),
                    col: i,
                });
                i += 1;
            }

            _ => return Err(Error::new(&format!("invalid character '{}'", chars[i]), i)),
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

    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.pos)
    }

    fn next(&mut self) -> Option<Token> {
        if self.pos >= self.tokens.len() { return None; }
        let t = self.tokens[self.pos].clone();
        self.pos += 1;
        Some(t)
    }

    fn parse_expression(&mut self) -> Result<Expr, Error> {
        let left_tok = self.next().ok_or(Error::new("unexpected end", 0))?;

        let left = match left_tok.kind {
            TokenKind::Number(n) => Expr::Number(n),
            TokenKind::Ident(s) => Expr::Ident(s),
            _ => return Err(Error::new("expected value", left_tok.col)),
        };

        if let Some(op_tok) = self.peek() {
            if let TokenKind::Operator(op) = &op_tok.kind {
                let op = op.clone();
                let col = op_tok.col;
                self.next();

                let right_tok = self.next().ok_or(Error::new("missing rhs", col))?;

                let right = match right_tok.kind {
                    TokenKind::Number(n) => Expr::Number(n),
                    TokenKind::Ident(s) => Expr::Ident(s),
                    _ => return Err(Error::new("invalid rhs", right_tok.col)),
                };

                return Ok(Expr::Binary {
                    op,
                    left: Box::new(left),
                    right: Box::new(right),
                });
            }
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
                    if r == 0 { return Err("division by zero".into()); }
                    Ok(l / r)
                }
                _ => Err("unknown operator".into()),
            }
        }
    }
}

fn render_error(line: &str, line_no: usize, err: Error) {
    eprintln!("error at line {}, col {}: {}", line_no, err.col + 1, err.msg);
    eprintln!("{}", line);
    eprintln!("{}^", " ".repeat(err.col));
}

fn execute_line(line: &str, line_no: usize, env: &mut HashMap<String, i64>) -> Result<i64, ()> {
    let tokens = match tokenize(line) {
        Ok(t) => t,
        Err(e) => {
            render_error(line, line_no, e);
            return Err(());
        }
    };

    let mut parser = Parser::new(tokens);

    let expr = match parser.parse_expression() {
        Ok(e) => e,
        Err(e) => {
            render_error(line, line_no, e);
            return Err(());
        }
    };

    match eval(&expr, env) {
        Ok(v) => Ok(v),
        Err(e) => {
            eprintln!("runtime error at line {}: {}", line_no, e);
            Err(())
        }
    }
}

fn run_script(input: &str, env: &mut HashMap<String, i64>) {
    for (i, line) in input.lines().enumerate() {
        if line.trim().is_empty() { continue; }

        match execute_line(line, i + 1, env) {
            Ok(v) => println!("{}", v),
            Err(_) => break,
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
        if line.is_empty() { continue; }

        match execute_line(line, 1, env) {
            Ok(v) => println!("{}", v),
            Err(_) => {}
        }
    }
}

fn main() {
    let mut env = HashMap::new();
    let args: Vec<String> = env::args().collect();

    if args.len() > 1 {
        let content = match fs::read_to_string(&args[1]) {
            Ok(c) => c,
            Err(e) => {
                eprintln!("file error: {}", e);
                return;
            }
        };

        run_script(&content, &mut env);
        return;
    }

    if stdin_is_tty() {
        repl(&mut env);
        return;
    }

    let mut input = String::new();
    io::stdin().read_to_string(&mut input).unwrap();
    run_script(&input, &mut env);
}
