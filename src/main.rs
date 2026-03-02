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
    let args: Vec<&str> = input.split_whitespace().collect();
    let command = args[0];

    match command {
        "cd" => {
            if args[1] == "~" {
                env::set_current_dir(env::var("HOME").unwrap())
                    .unwrap_or_else(|_| println!("cd: {}: No such file or directory", &args[1]))
            } else {
                env::set_current_dir(&args[1])
                    .unwrap_or_else(|_| println!("cd: {}: No such file or directory", &args[1]))
            }
        }
        "pwd" => println!("{}", env::current_dir().unwrap().display()),
        "echo" => println!("{}", args[1..].join(" ")),
        "type" => match args[1] {
            "exit" | "echo" | "type" | "pwd" | "cd" => println!("{} is a shell builtin", args[1]),
            _ => {
                if let Some(path) = find_executable_in_path(&args[1]) {
                    println!("{} is {}", args[1], path.display());
                } else {
                    println!("{}: not found", args[1]);
                }
            }
        },
        _ => {
            if let Some(_path) = find_executable_in_path(&command) {
                let _status = Command::new(command).args(&args[1..]).status();
            } else {
                println!("{}: command not found", command);
            }
        }
    }
}
