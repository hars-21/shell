use pathsearch::find_executable_in_path;
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
        "exit" | "echo" | "type" | "pwd" | "cd" => Ok(format!("{} is a shell builtin", args[0])),
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
