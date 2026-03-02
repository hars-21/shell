use pathsearch::find_executable_in_path;
use std::env;
#[allow(unused_imports)]
use std::io::{self, Write};
use std::process::Command;

fn main() {
    loop {
        print!("$ ");
        io::stdout().flush().unwrap();
        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        input = input.trim().to_string();

        if input == "exit" {
            break;
        }

        run_commands(&input);
    }
}

fn run_commands(input: &str) {
    let parts: Vec<&str> = input.split_whitespace().collect();
    let command = parts[0];

    match command {
        "pwd" => println!("{}", env::current_dir().unwrap().display()),
        "echo" => println!("{}", parts[1..].join(" ")),
        "type" => match parts[1] {
            "exit" | "echo" | "type" | "pwd" => println!("{} is a shell builtin", parts[1]),
            _ => {
                if let Some(path) = find_executable_in_path(&parts[1]) {
                    println!("{} is {}", parts[1], path.display());
                } else {
                    println!("{}: not found", parts[1]);
                }
            }
        },
        _ => {
            if let Some(_path) = find_executable_in_path(&command) {
                let _status = Command::new(command).args(&parts[1..]).status();
            } else {
                println!("{}: command not found", command);
            }
        }
    }
}
