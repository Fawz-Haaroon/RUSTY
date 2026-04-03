use std::collections::HashMap;
use std::io::{self, Read};

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
                tokens.push(Token { kind: TokenKind::LParen });
                i += 1;
            }

            ')' => {
                tokens.push(Token { kind: TokenKind::RParen });
                i += 1;
            }

            '+' | '-' | '*' | '/' => {
                tokens.push(Token {
                    kind: TokenKind::Operator(chars[i].to_string()),
                });
                i += 1;
            }

            '!' => {
                if i + 1 < chars.len() && chars[i + 1] == '=' {
                    tokens.push(Token {
                        kind: TokenKind::Operator("!=".into()),
                    });
                    i += 2;
                } else {
                    tokens.push(Token {
                        kind: TokenKind::Operator("!".into()),
                    });
                    i += 1;
                }
            }

            '&' => {
                if i + 1 < chars.len() && chars[i + 1] == '&' {
                    tokens.push(Token {
                        kind: TokenKind::Operator("&&".into()),
                    });
                    i += 2;
                } else {
                    return Err("unexpected '&'".into());
                }
            }

            '|' => {
                if i + 1 < chars.len() && chars[i + 1] == '|' {
                    tokens.push(Token {
                        kind: TokenKind::Operator("||".into()),
                    });
                    i += 2;
                } else {
                    return Err("unexpected '|'".into());
                }
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

            _ => return Err(format!("invalid character '{}'", chars[i])),
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

    Call {
        name: String,
        args: Vec<Expr>,
    },
}

fn fold_binary(op: &str, l: i64, r: i64) -> Option<i64> {
    match op {
        "+" => Some(l + r),
        "-" => Some(l - r),
        "*" => Some(l * r),
        "/" => if r != 0 { Some(l / r) } else { None },
        "==" => Some((l == r) as i64),
        "!=" => Some((l != r) as i64),
        "<" => Some((l < r) as i64),
        ">" => Some((l > r) as i64),
        "<=" => Some((l <= r) as i64),
        ">=" => Some((l >= r) as i64),
        _ => None,
    }
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

        let token = std::mem::replace(&mut self.tokens[self.pos].kind, TokenKind::Comma);
        self.pos += 1;
        Some(token)
    }

    fn parse_call(&mut self, name: String) -> Result<Expr, String> {
        let mut args = Vec::new();

        loop {
            if matches!(self.peek(), Some(TokenKind::RParen)) {
                self.next();
                break;
            }

            args.push(self.parse_expression(0)?);

            match self.peek() {
                Some(TokenKind::Comma) => { self.next(); }
                Some(TokenKind::RParen) => { self.next(); break; }
                _ => return Err("expected ',' or ')'".into()),
            }
        }

        Ok(Expr::Call { name, args })
    }

    fn parse_expression(&mut self, min_bp: u8) -> Result<Expr, String> {
        let mut left = match self.next() {
            Some(TokenKind::Number(n)) => Expr::Number(n),

            Some(TokenKind::Ident(name)) => {
                if matches!(self.peek(), Some(TokenKind::LParen)) {
                    self.next();
                    self.parse_call(name)?
                } else {
                    Expr::Ident(name)
                }
            }

            Some(TokenKind::Operator(op)) if op == "-" || op == "+" || op == "!" => {
                let expr = self.parse_expression(30)?;
                Expr::Unary { op, expr: Box::new(expr) }
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
                "||" => (2, 3),
                "&&" => (4, 5),
                "==" | "!=" => (6, 7),
                "<" | ">" | "<=" | ">=" => (8, 9),
                "+" | "-" => (10, 11),
                "*" | "/" => (12, 13),
                _ => break,
            };

            if l_bp < min_bp {
                break;
            }

            self.next();
            let right = self.parse_expression(r_bp)?;

            // constant folding
            if let (Expr::Number(lv), Expr::Number(rv)) = (&left, &right) {
                if let Some(v) = fold_binary(&op, *lv, *rv) {
                    left = Expr::Number(v);
                    continue;
                }
            }

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

        Expr::Ident(name) => env.get(name).copied()
            .ok_or_else(|| format!("undefined '{}'", name)),

        Expr::Unary { op, expr } => {
            let v = eval(expr, env)?;
            match op.as_str() {
                "-" => Ok(-v),
                "+" => Ok(v),
                "!" => Ok((v == 0) as i64),
                _ => Err("bad unary".into()),
            }
        }

        Expr::Binary { op, left, right } => {
            if op == "=" {
                if let Expr::Ident(name) = &**left {
                    let v = eval(right, env)?;
                    env.insert(name.clone(), v);
                    return Ok(v);
                }
                return Err("bad assignment".into());
            }

            if op == "&&" {
                let l = eval(left, env)?;
                if l == 0 { return Ok(0); }
                return Ok((eval(right, env)? != 0) as i64);
            }

            if op == "||" {
                let l = eval(left, env)?;
                if l != 0 { return Ok(1); }
                return Ok((eval(right, env)? != 0) as i64);
            }

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

        Expr::Call { name, args } => {
            if name == "clear" {
                env.clear();
                return Ok(0);
            }

            if name == "env" {
                for (k,v) in env.iter() {
                    println!("{} = {}", k, v);
                }
                return Ok(env.len() as i64);
            }

            let mut vals = Vec::new();
            for a in args {
                vals.push(eval(a, env)?);
            }

            match name.as_str() {
                "print" => {
                    for v in &vals { println!("{}", v); }
                    Ok(*vals.last().unwrap_or(&0))
                }
                "abs" => Ok(vals[0].abs()),
                "pow" => Ok(vals[0].pow(vals[1] as u32)),
                _ => Err("unknown fn".into()),
            }
        }
    }
}

fn main() {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input).unwrap();

    let mut env = HashMap::new();

    for line in input.lines() {
        if line.trim().is_empty() { continue; }

        let tokens = match tokenize(line) {
            Ok(t) => t,
            Err(e) => { eprintln!("{}", e); break; }
        };

        let mut parser = Parser::new(tokens);

        let expr = match parser.parse_expression(0) {
            Ok(e) => e,
            Err(e) => { eprintln!("{}", e); break; }
        };

        match eval(&expr, &mut env) {
            Ok(v) => println!("{}", v),
            Err(e) => { eprintln!("{}", e); break; }
        }
    }
}
