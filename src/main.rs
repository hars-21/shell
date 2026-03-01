use pathsearch::find_executable_in_path;
#[allow(unused_imports)]
use std::io::{self, Write};

fn main() {
    loop {
        print!("$ ");
        io::stdout().flush().unwrap();
        let mut command = String::new();
        io::stdin().read_line(&mut command).unwrap();
        command = command.trim().to_string();

        if command == "exit" {
            break;
        } else if command.starts_with("echo ") {
            println!("{}", &command[5..]);
        } else if command.starts_with("type ") {
            match &command[5..] {
                "exit" | "echo" | "type" => println!("{} is a shell builtin", &command[5..]),
                _ => {
                    if let Some(path) = find_executable_in_path(&command[5..]) {
                        println!("{} is {}", &command[5..], path.display());
                    } else {
                        println!("{}: not found", &command[5..]);
                    }
                }
            }
        } else {
            println!("{}: command not found", command);
        }
    }
}
