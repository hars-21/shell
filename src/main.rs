mod helper;
mod parser;

use std::fs::OpenOptions;
use std::io::{self, Write};
use std::process;

use reqsh::{
    cd, command_type, delcheck, echo, exec, history, jobs, listchecks, pwd, runcheck, savecheck,
};
use rustyline::config::BellStyle;
use rustyline::error::ReadlineError;
use rustyline::history::FileHistory;
use rustyline::{CompletionType, Config, EditMode, Editor};

use crate::helper::ShellHelper;
use crate::parser::ShellCommand;

fn run(cmd: ShellCommand, rl: &mut Editor<ShellHelper, FileHistory>) {
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

        "history" => {
            history(rl.history_mut(), &args);
        }

        "jobs" => jobs(),

        "savecheck" => match savecheck(args) {
            Ok(msg) => writeln!(stdout, "{}", msg).unwrap(),
            Err(e) => writeln!(stderr, "{}", e).unwrap(),
        },

        "listchecks" => match listchecks() {
            Ok(items) => {
                if items.is_empty() {
                    writeln!(stdout, "no checks saved").unwrap();
                } else {
                    for item in items {
                        writeln!(stdout, "{}", item).unwrap();
                    }
                }
            }
            Err(e) => writeln!(stderr, "{}", e).unwrap(),
        },

        "delcheck" => match delcheck(args) {
            Ok(msg) => writeln!(stdout, "{}", msg).unwrap(),
            Err(e) => writeln!(stderr, "{}", e).unwrap(),
        },

        "runcheck" => match runcheck(args) {
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
    let config = Config::builder()
        .history_ignore_space(true)
        .completion_type(CompletionType::List)
        .edit_mode(EditMode::Vi)
        .bell_style(BellStyle::Audible)
        .build();

    let mut rl = Editor::with_config(config).unwrap();
    rl.set_helper(Some(ShellHelper::new()));
    rl.load_history("history.txt").unwrap_or_default();

    loop {
        let readline = rl.readline("$ ");
        match readline {
            Ok(line) => {
                rl.add_history_entry(&line).unwrap();
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

                run(cmd, &mut rl);
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

    rl.append_history("history.txt").unwrap_or_default();
}
