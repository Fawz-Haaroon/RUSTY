use colored::*;
use rand::{distributions::Alphanumeric, Rng};
use regex::Regex;
use std::env;

fn evaluate_strength(password: &str) -> (usize, &'static str) {
    let length = password.len();

    let has_upper = Regex::new(r"[A-Z]").unwrap().is_match(password);
    let has_lower = Regex::new(r"[a-z]").unwrap().is_match(password);
    let has_digit = Regex::new(r"\d").unwrap().is_match(password);
    let has_symbol = Regex::new(r"[^A-Za-z0-9]").unwrap().is_match(password);

    // Scoring system: mix of length and character variety
    let mut score = 0;
    score += match length {
        0..=4 => 0,
        5..=8 => 1,
        9..=12 => 2,
        13..=16 => 3,
        _ => 4,
    };
    if has_lower {
        score += 1;
    }
    if has_upper {
        score += 1;
    }
    if has_digit {
        score += 1;
    }
    if has_symbol {
        score += 1;
    }

    // Clamp 1–7
    let level = score.min(7).max(1);

    let description = match level {
        1 => "🌑 Voidling",
        2 => "🪨 Stone Age",
        3 => "⚡ Spark Carrier",
        4 => "🔮 Mystic Adept",
        5 => "🦅 Skybreaker",
        6 => "🐉 Dragonforged",
        7 => "🌌 Cosmic Overlord",
        _ => "❓ Unknown",
    };

    (level, description)
}

fn render_bar(level: usize) -> String {
    let total = 7;
    let filled = "█".repeat(level);
    let empty = "░".repeat(total - level);

    let colored_bar = match level {
        1..=2 => filled.red().bold().to_string(),
        3..=4 => filled.yellow().bold().to_string(),
        5..=6 => filled.green().bold().to_string(),
        7 => filled.bright_magenta().bold().to_string(),
        _ => filled.white().to_string(),
    };

    format!("[{}{}] {}%", colored_bar, empty, (level * 100) / total)
}

fn generate_password(length: usize) -> String {
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .map(char::from)
        .take(length)
        .collect()
}

fn dramatic_intro() {
    println!("{}", "🧙 Welcome, traveler...".bright_blue().bold());
    println!(
        "{}",
        "Your password shall now be judged by the Ancient Order..."
            .italic()
            .dimmed()
    );
    println!();
}

fn check_password(pw: &str) {
    dramatic_intro();
    let (level, desc) = evaluate_strength(pw);

    // Display results
    println!("🔍 Analyzing password: {}", pw.bright_white());
    println!("🏆 Level {}/7: {}", level.to_string().bold(), desc.bold());
    println!("{}", render_bar(level));
    println!();

    match level {
        1 | 2 => println!("{}", "💀 This won’t survive a sneeze...".red().bold()),
        3 | 4 => println!(
            "{}",
            "⚔️ Decent, but still breakable by mere mortals.".yellow()
        ),
        5 | 6 => println!("{}", "🛡 Impressive! Few can pierce this defense.".green()),
        7 => println!(
            "{}",
            "🌌 Legendary! Even cosmic forces bow to this password."
                .magenta()
                .bold()
        ),
        _ => {}
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage:");
        eprintln!("  check <password>");
        eprintln!("  generate <length>");
        return;
    }

    match args[1].as_str() {
        "check" => {
            if args.len() < 3 {
                eprintln!("Error: Provide a password to check.");
            } else {
                check_password(&args[2]);
            }
        }
        "generate" => {
            if args.len() < 3 {
                eprintln!("Error: Provide length for password.");
            } else if let Ok(len) = args[2].parse::<usize>() {
                let password = generate_password(len);
                println!("✨ Generated password: {}", password.cyan().bold());
                check_password(&password);
            } else {
                eprintln!("Error: Invalid length.");
            }
        }
        _ => {
            eprintln!("Unknown command.");
        }
    }
}
