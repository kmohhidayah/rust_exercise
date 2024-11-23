use rand::Rng;
use std::cmp::Ordering;
use std::io;
use std::process::Command;

fn clear_screen() {
    if cfg!(target_os = "windows") {
        Command::new("cmd").args(["/c", "cls"]).status().unwrap();
    } else {
        Command::new("clear").status().unwrap();
    }
}

fn main() {
    let secret_number = rand::thread_rng().gen_range(0..100);
    loop {
        println!("Guess the number!");

        let mut guess = String::new();

        println!("Please input your guess.");
        io::stdin()
            .read_line(&mut guess)
            .expect("failed to read line.");

        let guess: u32 = match guess.trim().parse() {
            Ok(num) => num,
            Err(_) => continue,
        };

        println!("your guess {guess}");

        match guess.cmp(&secret_number) {
            Ordering::Less => {
                println!("To small!");
                std::thread::sleep(std::time::Duration::from_secs(1));
                clear_screen();
            }
            Ordering::Greater => {
                println!("To big!");
                std::thread::sleep(std::time::Duration::from_secs(1));
                clear_screen();
            }
            Ordering::Equal => {
                println!("You Win!");
                break;
            }
        }
    }
    println!("the secret number is {secret_number}");
}
