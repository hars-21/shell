use std::{env, path::PathBuf};

use rustyline::completion::{Completer, Pair, extract_word};
use rustyline::highlight::Highlighter;
use rustyline::hint::Hinter;
use rustyline::validate::Validator;
use rustyline::{Context, Helper, Result};

const BUILTINS: &[&str] = &[
    "cd",
    "echo",
    "exit",
    "pwd",
    "type",
    "history",
    "jobs",
    "savecheck",
    "runcheck",
    "listchecks",
    "delcheck",
];

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
        let (start, word) = extract_word(line, pos, None, |c| c == ' ');
        let first_word = line[..start].trim().is_empty();

        if first_word {
            let mut matches: Vec<String> = BUILTINS
                .iter()
                .copied()
                .chain(self.path_executables.iter().map(|s| s.as_str()))
                .filter(|cmd| cmd.starts_with(word))
                .map(|cmd| cmd.to_string())
                .collect();

            matches.sort();
            matches.dedup();

            let pairs = matches
                .into_iter()
                .map(|cmd| {
                    let replacement = format!("{cmd} ");
                    Pair {
                        display: replacement.clone(),
                        replacement,
                    }
                })
                .collect();

            return Ok((start, pairs));
        }

        let (base_dir, prefix) = match word.rfind('/') {
            Some(idx) => (&word[..=idx], &word[idx + 1..]),
            None => ("", word),
        };

        let dir: PathBuf = if base_dir.is_empty() {
            env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
        } else {
            env::current_dir()
                .map(|cwd| cwd.join(base_dir))
                .unwrap_or_else(|_| PathBuf::from(base_dir))
        };

        let mut pairs = Vec::new();

        if let Ok(entries) = dir.read_dir() {
            for entry in entries.flatten() {
                let file_name = entry.file_name();
                let name = file_name.to_string_lossy();

                if name.starts_with(prefix) {
                    let mut replacement = format!("{}{}", base_dir, name);

                    if entry.path().is_dir() {
                        if !replacement.ends_with('/') {
                            replacement.push('/');
                        }
                    } else {
                        if !replacement.ends_with(' ') {
                            replacement.push(' ');
                        }
                    }

                    pairs.push(Pair {
                        display: replacement.clone(),
                        replacement,
                    });
                }
            }
        }

        if pairs.is_empty() {
            return Ok((start, vec![]));
        }

        pairs.sort_by(|a, b| a.display.cmp(&b.display));

        Ok((start, pairs))
    }
}

fn list_path_executables() -> Vec<String> {
    let mut executables = Vec::new();

    if let Some(paths) = env::var_os("PATH") {
        for path in env::split_paths(&paths) {
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
    }

    executables.sort();
    executables.dedup();
    executables
}
