use pathsearch::find_executable_in_path;
use std::env;
use std::fs::{File, write};
#[allow(unused_imports)]
use std::io::{self, Write};
use std::process::Command;

struct CommandLine {
    command: String,
    args: Vec<String>,
    redirect: bool,
    filename: Option<String>,
}

fn main() {
    loop {
        print!("$ ");
        io::stdout().flush().unwrap();
        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        input = input.trim().to_string();

        if input.is_empty() {
            continue;
        }

        if input == "exit" {
            break;
        }

        run_commands(&input);
    }
}

fn run_commands(input: &str) {
    let shell_command = parse_args(input);
    let command = &shell_command.command;
    let args = &shell_command.args;

    match command.as_str() {
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
        "echo" => {
            println!("{}", args[1..].join(" "));
            if shell_command.redirect {
                file_write(&shell_command.filename, &args[1..].join(" "))
            }
        }
        "type" => match args[1].as_str() {
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
                let status = Command::new(command).args(&args[1..]).output();
                if let Ok(s) = status {
                    let content = String::from_utf8(s.stdout);
                    if let Ok(c) = content {
                        file_write(&shell_command.filename, &c);
                    }
                }
            } else {
                println!("{}: command not found", &command);
            }
        }
    }
}

fn parse_args(input: &str) -> CommandLine {
    let mut args = Vec::new();
    let mut current = String::new();
    let mut chars = input.chars().peekable();
    let mut redirect = false;
    let mut filename: Option<String> = None;

    while let Some(c) = chars.next() {
        match c {
            '\'' => {
                while let Some(c) = chars.next() {
                    if c == '\'' {
                        break;
                    } else {
                        current.push(c);
                    }
                }
            }

            '"' => {
                while let Some(c) = chars.next() {
                    if c == '"' {
                        break;
                    }
                    if c == '\\' {
                        if let Some(next) = chars.next() {
                            match next {
                                '"' | '\\' | '$' | '\n' => current.push(next),
                                _ => {
                                    current.push('\\');
                                    current.push(next);
                                }
                            }
                        }
                    } else {
                        current.push(c);
                    }
                }
            }

            '\\' => {
                if let Some(next) = chars.next() {
                    current.push(next);
                }
            }

            ' ' | '\t' => {
                if !current.is_empty() {
                    if redirect {
                        filename = Some(current.clone());
                        redirect = false;
                    } else {
                        args.push(current.clone());
                    }
                    current.clear();
                }
            }

            '>' => {
                redirect = true;
            }

            _ => current.push(c),
        }
    }

    if !current.is_empty() {
        if redirect {
            filename = Some(current.clone());
        } else {
            args.push(current.clone());
        }
        current.clear();
    }

    redirect = filename.is_some();

    let shell_command = CommandLine {
        command: args[0].clone(),
        args: args,
        redirect,
        filename,
    };

    return shell_command;
}

fn file_write(filename: &Option<String>, content: &String) {
    if let Some(file) = filename {
        File::create(&file).unwrap();
        write(&file, content).unwrap();
    }
}
