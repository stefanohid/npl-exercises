mod hello_world;

use std::fs::File;
use std::io::BufRead;
use std::io::{BufWriter, Write};

//main behaves like a statement, no need to return a value, so you end up with a semicolon
fn main() {
    hello_world::hello_world(String::new());
    let input = std::fs::File::open("/home/stefa/npl-projects/baby_emulator/input/sum.bby")
        .expect("Could not open file");
    let output = File::create("output.txt").expect("Could not create file");
    let mut writer = BufWriter::new(output);

    let reader = std::io::BufReader::new(input);

    let mut index = 0;
    writeln!(writer, "{}: 00000000000000000000000000000000", index)
        .expect("Could not write to file");

    for line in reader.lines() {
        match line {
            Ok(line) => {
                let mut result = String::new();
                if consume_line(&line, &mut result) == false {
                    println!("Syntax error at line: \n{}", line);
                    return;
                }
                index += 1;
                writeln!(
                    writer,
                    "{}: {}",
                    index,
                    result.chars().rev().collect::<String>()
                )
                .expect("Could not write to file");
            }
            Err(e) => eprintln!("Error reading line: {}", e),
        }
    }
}

fn op_code(val: &str) -> (Option<&'static str>, bool) {
    match val {
        "JMP" => (Some("000"), true),
        "JRP" => (Some("001"), true),
        "LDN" => (Some("010"), true),
        "STO" => (Some("011"), true),
        "SUB" => (Some("100"), true),
        "CMP" => (Some("110"), false),
        "STP" => (Some("111"), false),
        _ => (None, false),
    }
}

fn consume_line(line: &String, result: &mut String) -> bool {
    let has_addr;
    let mut iter = line.split(" ");
    iter.next(); // Skip first word

    *result = String::from("0000000000000000");

    let word = iter.next();
    match word {
        Some(word) => {
            let op_code = op_code(word);

            match op_code.0 {
                Some(string) => {
                    *result = format!("{}{}", result, string);
                    has_addr = op_code.1;
                }
                None => return false,
            }
        }
        None => return false,
    };

    if has_addr {
        let word = iter.next();
        match word {
            Some(addr) => {
                let int_addr = addr.parse::<u32>();
                match int_addr {
                    Ok(integer) => {
                        if integer > 31 {
                            return false;
                        }

                        *result = format!("{}00000000{:05b}", result, integer);
                    }
                    Err(_) => return false,
                }
            }
            None => return false,
        }
    } else {
        *result = format!("{}0000000000000", result);
    };

    return true;
}

// Case of BufWriter:
// use std::fs::File;
// use std::io::{self, BufWriter, Write};
// fn main() {
//     let file = File::create("output.txt").expect("Could not create file");
//     let mut writer = BufWriter::new(file);
//     writeln!(writer, "Hello, world!").expect("Could not write to file");
