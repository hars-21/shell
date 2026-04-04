use pathsearch::find_executable_in_path;
use rustyline::history::{FileHistory, History};
use std::fs::{self, OpenOptions};
use std::io::{BufWriter, Write};
use std::{env, process::Command};

const CHECKS_FILE: &str = "checks.txt";

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
        "exit" | "echo" | "type" | "pwd" | "cd" | "history" | "jobs" | "savecheck" | "runcheck"
        | "listchecks" | "delcheck" => Ok(format!("{} is a shell builtin", args[0])),
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

pub fn savecheck(args: &Vec<String>) -> Result<String, String> {
    if args.len() < 2 {
        return Err("usage: savecheck <name> <command>".to_string());
    }

    let name = args[0].clone();
    let command = args[1..].join(" ");

    if name.contains('|') {
        return Err("check name cannot contain |".to_string());
    }

    let content = fs::read_to_string(CHECKS_FILE).unwrap_or_default();
    let mut lines: Vec<String> = content
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| line.to_string())
        .collect();

    lines.retain(|line| {
        if let Some((existing_name, _)) = line.split_once('|') {
            existing_name != name
        } else {
            true
        }
    });

    lines.push(format!("{}|{}", name, command));

    fs::write(CHECKS_FILE, lines.join("\n") + "\n").map_err(|e| e.to_string())?;

    Ok(format!("saved check {}", name))
}

pub fn listchecks() -> Result<Vec<String>, String> {
    let content = fs::read_to_string(CHECKS_FILE).unwrap_or_default();

    let mut result = Vec::new();
    for line in content.lines() {
        if let Some((name, command)) = line.split_once('|') {
            result.push(format!("{} -> {}", name, command));
        }
    }

    Ok(result)
}

pub fn delcheck(args: &Vec<String>) -> Result<String, String> {
    if args.is_empty() {
        return Err("usage: delcheck <name>".to_string());
    }

    let name = args[0].clone();
    let content = fs::read_to_string(CHECKS_FILE).unwrap_or_default();
    let mut found = false;

    let lines: Vec<String> = content
        .lines()
        .filter(|line| {
            if let Some((existing_name, _)) = line.split_once('|') {
                if existing_name == name {
                    found = true;
                    return false;
                }
            }
            true
        })
        .map(|line| line.to_string())
        .collect();

    if !found {
        return Err(format!("check {} not found", name));
    }

    fs::write(CHECKS_FILE, lines.join("\n") + "\n").map_err(|e| e.to_string())?;
    Ok(format!("deleted check {}", name))
}

pub fn runcheck(args: &Vec<String>) -> Result<(String, String), String> {
    if args.is_empty() {
        return Err("usage: runcheck <name>".to_string());
    }

    let name = args[0].clone();
    let content = fs::read_to_string(CHECKS_FILE).unwrap_or_default();

    let mut command_to_run = String::new();
    for line in content.lines() {
        if let Some((existing_name, command)) = line.split_once('|') {
            if existing_name == name {
                command_to_run = command.to_string();
                break;
            }
        }
    }

    if command_to_run.is_empty() {
        return Err(format!("check {} not found", name));
    }

    let output = Command::new("sh")
        .arg("-c")
        .arg(command_to_run)
        .output()
        .map_err(|err| err.to_string())?;

    let stdout = String::from_utf8(output.stdout).unwrap().trim().to_string();
    let stderr = String::from_utf8(output.stderr).unwrap().trim().to_string();

    Ok((stdout, stderr))
}

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
            String::from("echo is a reqsh builtin"),
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
