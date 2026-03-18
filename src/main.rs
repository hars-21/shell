mod helper;

use std::fs::OpenOptions;
use std::io::{self, Write};
use std::process;

use codecrafters_shell::{cd, command_type, echo, exec, pwd};
use rustyline::Editor;
use rustyline::error::ReadlineError;

use crate::helper::ShellHelper;

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
    stdout: Option<String>,
    stderr: Option<String>,
    append: bool,
}

enum RedirectType {
    Stdout,
    Stderr,
}

impl ShellCommand {
    fn build(command_line: &str) -> Result<ShellCommand, &str> {
        let mut tokens = Vec::new();
        let mut current = String::new();
        let mut chars = command_line.chars().peekable();
        let mut pending_redirect: Option<RedirectType> = None;
        let mut stdout: Option<String> = None;
        let mut stderr: Option<String> = None;
        let mut append = false;

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
                        if let Some(rtype) = pending_redirect.take() {
                            match rtype {
                                RedirectType::Stdout => stdout = Some(current.clone()),
                                RedirectType::Stderr => stderr = Some(current.clone()),
                            }
                        } else {
                            tokens.push(current.clone());
                        }
                        current.clear();
                    }
                }

                '1' => {
                    if let Some('>') = chars.peek() {
                        chars.next();
                        pending_redirect = Some(RedirectType::Stdout);
                        if let Some('>') = chars.peek() {
                            chars.next();
                            append = true;
                        }
                    } else {
                        current.push('1');
                    }
                }

                '2' => {
                    if let Some('>') = chars.peek() {
                        chars.next();
                        pending_redirect = Some(RedirectType::Stderr);
                        if let Some('>') = chars.peek() {
                            chars.next();
                            append = true;
                        }
                    } else {
                        current.push('2');
                    }
                }

                '>' => {
                    if let Some('>') = chars.peek() {
                        chars.next();
                        pending_redirect = Some(RedirectType::Stdout);
                        append = true;
                    } else {
                        pending_redirect = Some(RedirectType::Stdout);
                    }
                }

                _ => current.push(c),
            }
        }

        if !current.is_empty() {
            if let Some(rtype) = pending_redirect.take() {
                match rtype {
                    RedirectType::Stdout => stdout = Some(current.clone()),
                    RedirectType::Stderr => stderr = Some(current.clone()),
                }
            } else {
                tokens.push(current.clone());
            }
            current.clear();
        }

        let (name, args) = tokens.split_first().unwrap();

        Ok(ShellCommand {
            name: name.clone(),
            args: args.to_vec(),
            stdout,
            stderr,
            append,
        })
    }
}

fn run(cmd: ShellCommand) {
    let mut stdout: Box<dyn Write> = match cmd.stdout {
        Some(file) => Box::new(
            OpenOptions::new()
                .create(true)
                .write(true)
                .append(cmd.append)
                .open(file)
                .unwrap(),
        ),
        None => Box::new(io::stdout()),
    };

    let mut stderr: Box<dyn Write> = match cmd.stderr {
        Some(file) => Box::new(
            OpenOptions::new()
                .create(true)
                .write(true)
                .append(cmd.append)
                .open(file)
                .unwrap(),
        ),
        None => Box::new(io::stderr()),
    };

    let command = &cmd.name;
    let args = &cmd.args;

    match command.as_str() {
        "cd" => cd(args).unwrap_or_else(|err| {
            writeln!(stderr, "{}", err).unwrap();
        }),

        "pwd" => match pwd() {
            Ok(c) => writeln!(stdout, "{}", c).unwrap(),
            Err(e) => writeln!(stderr, "{}", e).unwrap(),
        },

        "echo" => {
            writeln!(stdout, "{}", &echo(&args)).unwrap();
        }

        "type" => match command_type(&args) {
            Ok(c) => writeln!(stdout, "{}", c).unwrap(),
            Err(e) => writeln!(stderr, "{}", e).unwrap(),
        },

        _ => match &exec(&command, &args) {
            Ok((out, err)) => {
                if !out.is_empty() {
                    writeln!(stdout, "{}", out).unwrap();
                }
                if !err.is_empty() {
                    writeln!(stderr, "{}", err).unwrap();
                }
            }
            Err(e) => writeln!(stderr, "{}", e).unwrap(),
        },
    }
}

fn main() {
    let mut rl = Editor::new().unwrap();
    rl.set_helper(Some(ShellHelper::new()));

    loop {
        let readline = rl.readline("$ ");
        match readline {
            Ok(line) => {
                if line.is_empty() {
                    continue;
                }

                if line == "exit" {
                    break;
                }

                let cmd = ShellCommand::build(&line).unwrap_or_else(|err| {
                    eprintln!("Error parsing arguments: {err}");
                    process::exit(1);
                });

                run(cmd);
            }

            Err(ReadlineError::Interrupted) => {
                break;
            }

            Err(ReadlineError::Eof) => {
                break;
            }

            Err(err) => {
                println!("Error: {:?}", err);
                break;
            }
        }
    }
}
