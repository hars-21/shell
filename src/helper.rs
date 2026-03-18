use std::env;

use rustyline::completion::{Completer, Pair};
use rustyline::highlight::Highlighter;
use rustyline::hint::Hinter;
use rustyline::validate::Validator;
use rustyline::{Context, Helper, Result};

const BUILTINS: &[&str] = &["cd", "echo", "exit", "pwd", "type"];

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

        let matches = BUILTINS
            .iter()
            .copied()
            .chain(self.path_executables.iter().map(|p| p.as_str()))
            .filter(|cmd| cmd.starts_with(input))
            .map(|cmd| {
                let replacement = format!("{cmd} ");
                Pair {
                    display: replacement.clone(),
                    replacement,
                }
            })
            .collect();

        Ok((0, matches))
    }
}

fn list_path_executables() -> Vec<String> {
    let mut executables = Vec::new();
    for path in env::split_paths(&env::var_os("PATH").unwrap_or_default()) {
        if let Ok(entries) = path.read_dir() {
            for entry in entries.flatten() {
                executables.push(format!("{}", entry.file_name().display()));
            }
        }
    }

    executables
}
