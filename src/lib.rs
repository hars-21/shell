use pathsearch::find_executable_in_path;
use rustyline::history::{FileHistory, History};
use std::fs::OpenOptions;
use std::io::{BufWriter, Write};
use std::{env, process::Command};

pub fn cd(args: &Vec<String>) -> Result<(), String> {
    let target = if args[0] == "~" {
        env::var("HOME").unwrap()
    } else {
        args[0].clone()
    };

    env::set_current_dir(&target).map_err(|_| format!("cd: {}: No such file or directory", target))
}

pub fn pwd() -> Result<String, String> {
    env::current_dir()
        .map(|p| p.display().to_string())
        .map_err(|e| format!("{}", e))
}

pub fn echo(args: &Vec<String>) -> String {
    args.join(" ")
}

pub fn command_type(args: &Vec<String>) -> Result<String, String> {
    match args[0].as_str() {
        "exit" | "echo" | "type" | "pwd" | "cd" | "history" | "jobs" => {
            Ok(format!("{} is a shell builtin", args[0]))
        }
        _ => match find_executable_in_path(&args[0]) {
            Some(path) => Ok(format!("{} is {}", args[0], path.display())),
            None => Err(format!("{}: not found", args[0])),
        },
    }
}

pub fn exec(command: &String, args: &Vec<String>) -> Result<(String, String), String> {
    if find_executable_in_path(&command).is_none() {
        return Err(format!("{}: command not found", &command));
    }

    let output = Command::new(&command)
        .args(args)
        .output()
        .map_err(|err| err.to_string())?;

    let stdout = String::from_utf8(output.stdout).unwrap().trim().to_string();
    let stderr = String::from_utf8(output.stderr).unwrap().trim().to_string();

    Ok((stdout, stderr))
}

pub fn history(history: &mut FileHistory, args: &[String]) {
    if args.first().is_some_and(|arg| arg == "-r") {
        history.load(std::path::Path::new(&args[1])).unwrap();
        return;
    }

    if args.first().is_some_and(|arg| arg == "-w") {
        let mut output = BufWriter::new(std::fs::File::create(&args[1]).unwrap());
        for record in history.iter() {
            writeln!(&mut output, "{record}").unwrap();
        }
        return;
    }

    if args.first().is_some_and(|arg| arg == "-a") {
        let mut output = BufWriter::new(
            OpenOptions::new()
                .create(true)
                .append(true)
                .open(&args[1])
                .unwrap(),
        );
        let since_idx = history
            .search(
                &format!("history -a {}", args[1]),
                history.len().saturating_sub(2),
                rustyline::history::SearchDirection::Reverse,
            )
            .unwrap()
            .map_or(0, |search_result| search_result.idx + 1);
        for record in history.iter().skip(since_idx) {
            writeln!(&mut output, "{record}").unwrap();
        }
        return;
    }

    let history_length = history.len();
    let limit: usize = args
        .first()
        .map_or(history_length, |n| n.parse().unwrap())
        .min(history_length);
    for (i, record) in history.iter().enumerate().skip(history_length - limit) {
        println!("  {} {}", i + 1, record)
    }
}

pub fn jobs() {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cd_invalid_dir() {
        let args = vec!["/nonexistent".to_string()];
        assert!(cd(&args).is_err());
    }

    #[test]
    fn test_pwd() {
        let path = pwd().unwrap();
        assert!(!path.is_empty());
    }

    #[test]
    fn test_echo() {
        let args: Vec<String> = vec!["Hello".to_string(), "World".to_string()];
        assert_eq!("Hello World".to_string(), echo(&args));
    }

    #[test]
    fn test_command_type() {
        let command = String::from("echo");
        let args = vec![command];

        assert_eq!(
            String::from("echo is a shell builtin"),
            command_type(&args).unwrap()
        );
    }

    #[test]
    fn test_exec() {
        let command = String::from("args");
        let args = vec!["".to_string()];
        assert!(exec(&command, &args).is_err());
    }
}
