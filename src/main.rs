use std::io::{self, Write};
use std::process;

use codecrafters_shell::{cd, command_type, echo, exec, file_write, pwd};

// enum Builtin {
//     Cd,
//     Echo,
//     Pwd,
//     Type,
// }

// impl Builtin {
//     fn from_str(s: &str) -> Option<Builtin> {
//         match s {
//             "cd" => Some(Builtin::Cd),
//             "echo" => Some(Builtin::Echo),
//             "pwd" => Some(Builtin::Pwd),
//             "type" => Some(Builtin::Type),
//             _ => None,
//         }
//     }
// }

struct ShellCommand {
    name: String,
    args: Vec<String>,
    append: bool,
    filename: String,
}

impl ShellCommand {
    fn build(command_line: &str) -> Result<ShellCommand, &str> {
        let mut tokens = Vec::new();
        let mut current = String::new();
        let mut chars = command_line.chars().peekable();
        let mut append = false;
        let mut filename = String::new();

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
                        if append {
                            filename = current.clone();
                            append = false;
                        } else {
                            tokens.push(current.clone());
                        }
                        current.clear();
                    }
                }

                '1' => {
                    if let Some('>') = chars.peek() {
                        chars.next();
                        append = true;
                    } else {
                        current.push('1');
                    }
                }

                '>' => {
                    append = true;
                }

                _ => current.push(c),
            }
        }

        if !current.is_empty() {
            if append {
                filename = current.clone();
            } else {
                tokens.push(current.clone());
            }
            current.clear();
        }

        append = !filename.is_empty();
        let (name, args) = tokens.split_first().unwrap();

        Ok(ShellCommand {
            name: name.clone(),
            args: args.to_vec(),
            append,
            filename,
        })
    }
}

fn run(cmd: ShellCommand) {
    let command = &cmd.name;
    let args = &cmd.args;

    match command.as_str() {
        "cd" => cd(args).unwrap_or_else(|err| {
            println!("{}", err);
        }),
        "pwd" => match pwd() {
            Ok(c) => println!("{}", c),
            Err(e) => println!("{}", e),
        },
        "echo" => {
            if cmd.append {
                file_write(&cmd.filename, &echo(&args));
            } else {
                println!("{}", &echo(&args));
            }
        }
        "type" => match command_type(&args) {
            Ok(c) => println!("{}", c),
            Err(e) => println!("{}", e),
        },
        _ => match &exec(&command, &args) {
            Ok(c) => {
                if cmd.append {
                    file_write(&cmd.filename, c);
                } else {
                    println!("{}", c);
                }
            }
            Err(e) => println!("{}", e),
        },
    }
}

fn main() {
    loop {
        print!("$ ");
        io::stdout().flush().unwrap();
        let mut command_line = String::new();
        io::stdin().read_line(&mut command_line).unwrap();
        command_line = command_line.trim().to_string();

        if command_line.is_empty() {
            continue;
        }

        if command_line == "exit" {
            break;
        }

        let cmd = ShellCommand::build(&command_line).unwrap_or_else(|err| {
            eprintln!("Error parsing arguments: {err}");
            process::exit(1);
        });

        run(cmd);
    }
}
