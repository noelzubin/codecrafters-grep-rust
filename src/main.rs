use std::env;
use std::io;
use std::process;

fn match_pattern(input_line: &str, pattern: &str) -> bool {
    let mut chars = pattern.chars();

    match chars.next() {
        Some('\\') => {
            match chars.next() {
                Some('d') => return input_line.chars().any(|c| c.is_ascii_digit()),
                Some('w') => return input_line.chars().any(|c| c.is_alphanumeric()),
                _ => panic!("unhandled pattern {}", pattern),
            }
        }
        Some(ch) => return input_line.contains(ch),
        _ => panic!("unhandled pattern {}", pattern),
    }
}

// Usage: echo <input_text> | your_grep.sh -E <pattern>
fn main() {
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    println!("Logs from your program will appear here!");

    if env::args().nth(1).unwrap() != "-E" {
        println!("Expected first argument to be '-E'");
        process::exit(1);
    }

    let pattern = env::args().nth(2).unwrap();
    let mut input_line = String::new();

    io::stdin().read_line(&mut input_line).unwrap();

    // Uncomment this block to pass the first stage
    if match_pattern(&input_line, &pattern) {
        process::exit(0)
    } else {
        process::exit(1)
    }
}
