use std::io::{self, Write};
use std::process;

use codecrafters_shell::{cd, command_type, echo, exec, file_write, pwd};

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

    fn run(&self) -> Result<String, String> {
        let command = &self.name;
        let args = &self.args;

        match command.as_str() {
            "cd" => cd(args),
            "pwd" => pwd(),
            "echo" => echo(&args),
            "type" => command_type(&command),
            _ => exec(&command, &args),
        }
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

        let output = cmd.run().unwrap_or_else(|err| {
            eprintln!("{}", err);
            process::exit(1);
        });

        if cmd.append {
            file_write(&cmd.filename, &output);
        }
    }
}
