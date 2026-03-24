use std::env;

use rustyline::completion::{Completer, Pair};
use rustyline::highlight::Highlighter;
use rustyline::hint::Hinter;
use rustyline::validate::Validator;
use rustyline::{Context, Helper, Result};

const BUILTINS: &[&str] = &["cd", "echo", "exit", "pwd", "type", "history"];

pub struct ShellHelper {
    path_executables: Vec<String>,
}

impl ShellHelper {
    pub fn new() -> Self {
        Self {
            path_executables: list_path_executables(),
        }
    }
}

impl Helper for ShellHelper {}

impl Hinter for ShellHelper {
    type Hint = String;
}

impl Highlighter for ShellHelper {}

impl Validator for ShellHelper {}

impl Completer for ShellHelper {
    type Candidate = Pair;

    fn complete(&self, line: &str, pos: usize, _ctx: &Context<'_>) -> Result<(usize, Vec<Pair>)> {
        let input = &line[..pos];
        let start = input.rfind(char::is_whitespace).map(|i| i + 1).unwrap_or(0);

        let word = &input[start..];

        let mut matches: Vec<String> = BUILTINS
            .iter()
            .copied()
            .chain(self.path_executables.iter().map(|p| p.as_str()))
            .filter(|cmd| cmd.starts_with(word))
            .map(|cmd| cmd.to_string())
            .collect();

        matches.sort();
        matches.dedup();

        let pairs = matches
            .into_iter()
            .map(|cmd: String| {
                let replacement = format!("{cmd} ");
                Pair {
                    display: replacement.clone(),
                    replacement: replacement,
                }
            })
            .collect();

        Ok((start, pairs))
    }
}

fn list_path_executables() -> Vec<String> {
    let mut executables = Vec::new();

    for path in env::split_paths(&env::var_os("PATH").unwrap_or_default()) {
        if let Ok(entries) = path.read_dir() {
            for entry in entries.flatten() {
                let path = entry.path();

                if path.is_file() {
                    if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                        executables.push(name.to_string());
                    }
                }
            }
        }
    }

    executables.sort();
    executables.dedup();
    executables
}
