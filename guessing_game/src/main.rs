use std::io;
use std::cmp::Ordering;
use rand::Rng;
use colored::*;
use std::time::Instant;

fn main() {
    println!("{}", "Guess the Number Game!".cyan().bold());

    let secret_number = rand::thread_rng().gen_range(1..=100);
    let start_time = Instant::now();
    let mut attempts = 0;

    loop {
        println!("{}", "Please input your guess:".yellow());

        let mut guess = String::new();
        io::stdin().read_line(&mut guess).expect("Failed to read line");

        let guess: u32 = match guess.trim().parse() {Ok(num) => num, Err(_) => {println!("{}", "Please enter a valid number".red()); continue;}};

        attempts += 1;
        println!(" You guessed: {}", guess.to_string().bold());

        match guess.cmp(&secret_number) {
            Ordering::Less => println!("{}", "Your guess is smaller!".red()),
            Ordering::Greater => println!("{}", "Your guess is larger!".red()),
            Ordering::Equal => {
                println!("{}", "You win!".green().bold());
                println!("Total attempts: {}", attempts);
                println!("Time taken: {:.2?}", start_time.elapsed());
                break;
            }
        }
    }
}
