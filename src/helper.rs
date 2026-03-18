use rustyline::completion::{Completer, Pair};
use rustyline::highlight::Highlighter;
use rustyline::hint::Hinter;
use rustyline::validate::Validator;
use rustyline::{Context, Helper, Result};

const BUILTINS: &[&str] = &["cd", "echo", "exit", "pwd", "type"];

pub struct ShellHelper;

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
            .filter(|cmd| cmd.starts_with(input))
            .map(|cmd| {
                let replacement = format!("{cmd}");
                Pair {
                    display: replacement.clone(),
                    replacement,
                }
            })
            .collect();

        Ok((0, matches))
    }
}
