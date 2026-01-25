use colored::*;
use num_bigint::BigInt;
use num_rational::BigRational;
use num_traits::{Num, ToPrimitive, Zero};
use std::io::{self, Write};

fn main() {
    Ui::banner();
    Ui::rules();

    let input = RawInput::read();
    let inspector = Inspector::new(input);

    let interpretations = inspector.inspect();

    if interpretations.is_empty() {
        Ui::error("no valid interpretations");
        return;
    }

    let renderer = Renderer::default();
    renderer.render_all(&interpretations);
}

//UI (START SCREEN)
struct Ui;

impl Ui {
    fn banner() {
        println!("{}", "\nNUMBER SYSTEM (RADIX BASE) CONVERTOR");
    }

    fn rules() {
        println!(
            "{}",
            "
            INSTRUCTIONS::
            - 0bxxxx / 0bxx.yy → binary
            - 0oxxxx / 0oxx.yy → octal
            - 0xxxxx / 0xxx.yy → hexadecimal
            - noprefix (def.)  → enumerate integer interpretations
            - decimal fractions allowed without prefix
        "
            .bright_yellow()
        )
    }

    fn error(msg: &str) {
        eprintln!("{}", msg.red());
    }
}

// INPUT
struct RawInput(String);

impl RawInput {
    fn read() -> Self {
        print!("{}", "enter number > ");
        io::stdout().flush().unwrap();

        let mut s = String::new();
        io::stdin().read_line(&mut s).unwrap();

        Self(s.trim().to_owned())
    }

    fn as_str(&self) -> &str {
        &self.0
    }
}

// RADIX BASES
#[derive(Clone, Copy, Debug)]
enum Radix {
    Bin,
    Oct,
    Dec,
    Hex,
}

impl Radix {
    fn base(self) -> u32 {
        match self {
            Radix::Bin => 2,
            Radix::Oct => 8,
            Radix::Dec => 10,
            Radix::Hex => 16,
        }
    }

    fn all_integer_candidates() -> [Radix; 4] {
        [Radix::Dec, Radix::Bin, Radix::Oct, Radix::Hex]
    }

    fn name(self) -> &'static str {
        match self {
            Radix::Bin => "binary",
            Radix::Oct => "octal",
            Radix::Dec => "decimal",
            Radix::Hex => "hex",
        }
    }
}

// EXACT NUMBER
#[derive(Clone)]
struct ExactNumber(BigRational);

impl ExactNumber {
    fn new(v: BigRational) -> Self {
        Self(v)
    }

    fn rational(&self) -> &BigRational {
        &self.0
    }
}

// INTERPRETATION
struct Interpretation {
    radix: Radix,
    value: ExactNumber,
}

impl Interpretation {
    fn new(radix: Radix, value: BigRational) -> Self {
        Self {
            radix,
            value: ExactNumber::new(value),
        }
    }
}

// INSPECTOR
struct Inspector {
    input: RawInput,
}

impl Inspector {
    fn new(input: RawInput) -> Self {
        Self { input }
    }

    fn inspect(&self) -> Vec<Interpretation> {
        let s = self.input.as_str();

        if let Some((radix, rest)) = Self::explicit_prefix(s) {
            return Self::parse_single(radix, rest);
        }

        if s.contains('.') {
            return Self::parse_decimal_fraction(s);
        }

        Self::enumerate_integer(s)
    }

    fn explicit_prefix(s: &str) -> Option<(Radix, &str)> {
        s.strip_prefix("0b")
            .map(|r| (Radix::Bin, r))
            .or_else(|| s.strip_prefix("0o").map(|r| (Radix::Oct, r)))
            .or_else(|| s.strip_prefix("0x").map(|r| (Radix::Hex, r)))
    }

    fn parse_single(radix: Radix, s: &str) -> Vec<Interpretation> {
        Self::parse(s, radix).into_iter().collect()
    }

    fn enumerate_integer(s: &str) -> Vec<Interpretation> {
        Radix::all_integer_candidates()
            .into_iter()
            .filter(|&r| Self::valid_for_base(s, r))
            .filter_map(|r| Self::parse(s, r))
            .collect()
    }

    fn parse_decimal_fraction(s: &str) -> Vec<Interpretation> {
        parse_decimal_fraction(s)
            .ok()
            .map(|v| Interpretation::new(Radix::Dec, v))
            .into_iter()
            .collect()
    }

    fn parse(s: &str, radix: Radix) -> Option<Interpretation> {
        // validate input constraints
        if s.is_empty() || s.starts_with('-') {
            return None;
        }

        if s.contains('.') {
            parse_base_fraction(s, radix.base())
                .ok()
                .map(|v| Interpretation::new(radix, v))
        } else {
            BigInt::from_str_radix(s, radix.base())
                .ok()
                .map(BigRational::from_integer)
                .map(|v| Interpretation::new(radix, v))
        }
    }

    fn valid_for_base(s: &str, radix: Radix) -> bool {
        !s.is_empty() && s.chars().all(|c| c.to_digit(radix.base()).is_some())
    }
}

// RENDERER
struct Renderer {
    frac_limit: usize,
}

impl Default for Renderer {
    fn default() -> Self {
        Self { frac_limit: 64 }
    }
}

impl Renderer {
    fn render_all(&self, items: &[Interpretation]) {
        for i in items {
            self.render(i);
            println!();
        }
    }

    fn render(&self, i: &Interpretation) {
        println!("{} {}", "interpreted as".bright_blue(), i.radix.name());

        self.render_decimal(i.value.rational());
        self.render_radix("binary", 2, i.value.rational());
        self.render_radix("octal", 8, i.value.rational());
        self.render_radix("hex", 16, i.value.rational());

        println!(
            "{} {}/{}",
            "rational :".cyan(),
            i.value.rational().numer(),
            i.value.rational().denom()
        );
    }

    fn render_decimal(&self, v: &BigRational) {
        let exact = format_decimal(v);

        if let Some(approx) = v.to_f64() {
            println!(
                "{} {}  {} {:.10}",
                "decimal  :".cyan(),
                exact,
                "≈".dimmed(),
                approx
            );
        } else {
            println!("{} {}", "decimal  :".cyan(), exact);
        }
    }

    fn render_radix(&self, label: &str, base: u32, v: &BigRational) {
        println!(
            "{} {}",
            format!("{label:<7}  :").cyan(),
            to_base(v, base, self.frac_limit)
        );
    }
}

// BASE CONVERSION
fn to_base(v: &BigRational, base: u32, limit: usize) -> String {
    let base_big = BigInt::from(base);

    let int = v.to_integer();
    let mut frac = v - BigRational::from_integer(int.clone());

    let mut out = int.to_str_radix(base);

    if frac.is_zero() {
        return out;
    }

    out.push('.');

    for _ in 0..limit {
        frac *= &base_big;
        let d = frac.to_integer();
        let digit = d.to_u32().unwrap();

        out.push(if digit < 10 {
            (b'0' + digit as u8) as char
        } else {
            (b'a' + (digit - 10) as u8) as char
        });

        frac -= BigRational::from_integer(d);

        if frac.is_zero() {
            break;
        }
    }

    out
}

// DECIMAL FORMAT
fn format_decimal(v: &BigRational) -> String {
    let num = v.numer();
    let den = v.denom();

    let mut d = den.clone();
    let mut k = 0usize;

    while (&d % 10u32) == BigInt::zero() {
        d /= 10u32;
        k += 1;
    }

    if d != BigInt::from(1u32) {
        return format!("{}/{}", num, den);
    }

    if k == 0 {
        return num.to_str_radix(10);
    }

    let neg = num.sign() == num_bigint::Sign::Minus;
    let mut s = if neg {
        (-num).to_str_radix(10)
    } else {
        num.to_str_radix(10)
    };

    if k >= s.len() {
        s = format!("0.{}{}", "0".repeat(k - s.len()), s);
    } else {
        s.insert(s.len() - k, '.');
    }

    if neg {
        format!("-{s}")
    } else {
        s
    }
}

/// PARSING HELPERS
fn parse_decimal_fraction(s: &str) -> Result<BigRational, ()> {
    if s.matches('.').count() != 1 {
        return Err(());
    }

    let neg = s.starts_with('-');
    let s = s.trim_start_matches('-');

    let (i, f) = s.split_once('.').ok_or(())?;

    // handle edge cases
    let i = if i.is_empty() { "0" } else { i };
    let f = if f.is_empty() { return Err(()) } else { f };

    let mut num = BigInt::from_str_radix(&(i.to_string() + f), 10).map_err(|_| ())?;
    let den = BigInt::from(10u32).pow(f.len() as u32);

    if neg {
        num = -num;
    }

    Ok(BigRational::new(num, den))
}

fn parse_base_fraction(s: &str, base: u32) -> Result<BigRational, ()> {
    if s.matches('.').count() != 1 {
        return Err(());
    }

    let (i, f) = s.split_once('.').ok_or(())?;

    // handle edge cases
    let i = if i.is_empty() { "0" } else { i };
    if f.is_empty() {
        return Err(());
    }

    let int = BigInt::from_str_radix(i, base).map_err(|_| ())?;
    let mut val = BigRational::from_integer(int);


    let base_big = BigInt::from(base);
    let mut denom = base_big.clone();

    for c in f.chars() {
        let d = c.to_digit(base).ok_or(())?;
        val += BigRational::new(BigInt::from(d), denom.clone());
        denom *= &base_big;
    }

    Ok(val)
}
