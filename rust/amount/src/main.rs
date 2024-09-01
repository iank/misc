use amount::parser;
use std::fs::read_to_string;

fn main() {
    for line in read_to_string("/proc/mounts").unwrap().lines() {
        match parser::parse_line(line) {
            Ok((_, m)) => {
                println!("{m}");
            }
            Err(_) => {
                println!("Failed to parse line {line}");
            }
        }
    }
}
