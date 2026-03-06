//! Headless shell parser/evaluator compatible with browser-hosted environments.
//!
//! This crate intentionally implements only the small subset needed by the system terminal:
//! line tokenization, quoting/escaping, argument-vector construction, and basic session state.

#![warn(missing_docs, rustdoc::broken_intra_doc_links)]

use serde::{Deserialize, Serialize};

/// Mutable shell session state tracked by the headless evaluator.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct HeadlessShellState {
    /// Most recent argv parsed by the evaluator.
    pub last_argv: Vec<String>,
}

/// Input payload for a shell evaluation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HeadlessEvalInput {
    /// Raw line to parse.
    pub line: String,
}

/// Parsed shell evaluation result.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HeadlessEvalOutput {
    /// Parsed argv tokens.
    pub argv: Vec<String>,
    /// Whether the command line was empty after trimming.
    pub is_empty: bool,
    /// Whether the parsed argv requests help.
    pub wants_help: bool,
}

/// Parse/evaluation error from the headless shell.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HeadlessEvalError {
    /// Human-readable message.
    pub message: String,
}

impl HeadlessEvalError {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

/// Stateless entrypoint for headless shell parsing.
#[derive(Debug, Default, Clone, Copy)]
pub struct HeadlessEvaluator;

impl HeadlessEvaluator {
    /// Parses `input` and updates `state`.
    pub fn eval_line(
        &self,
        state: &mut HeadlessShellState,
        input: HeadlessEvalInput,
    ) -> Result<HeadlessEvalOutput, HeadlessEvalError> {
        let argv = tokenize(&input.line)?;
        state.last_argv = argv.clone();
        Ok(HeadlessEvalOutput {
            wants_help: argv.iter().any(|arg| arg == "--help" || arg == "-h")
                || argv.first().map(|arg| arg == "help").unwrap_or(false),
            is_empty: argv.is_empty(),
            argv,
        })
    }
}

/// Convenience wrapper around [`HeadlessEvaluator::eval_line`].
pub fn eval_line(
    state: &mut HeadlessShellState,
    input: HeadlessEvalInput,
) -> Result<HeadlessEvalOutput, HeadlessEvalError> {
    HeadlessEvaluator.eval_line(state, input)
}

fn tokenize(line: &str) -> Result<Vec<String>, HeadlessEvalError> {
    let mut tokens = Vec::new();
    let mut current = String::new();
    let mut chars = line.chars().peekable();
    let mut quote = None::<char>;

    while let Some(ch) = chars.next() {
        match quote {
            Some(active) if ch == active => quote = None,
            Some(_) if ch == '\\' => {
                let Some(next) = chars.next() else {
                    return Err(HeadlessEvalError::new("dangling escape sequence"));
                };
                current.push(next);
            }
            Some(_) => current.push(ch),
            None if ch == '"' || ch == '\'' => quote = Some(ch),
            None if ch.is_whitespace() => {
                if !current.is_empty() {
                    tokens.push(std::mem::take(&mut current));
                }
            }
            None if ch == '\\' => {
                let Some(next) = chars.next() else {
                    return Err(HeadlessEvalError::new("dangling escape sequence"));
                };
                current.push(next);
            }
            None => current.push(ch),
        }
    }

    if quote.is_some() {
        return Err(HeadlessEvalError::new("unterminated quoted string"));
    }

    if !current.is_empty() {
        tokens.push(current);
    }

    Ok(tokens)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tokenizes_quoted_segments() {
        let mut state = HeadlessShellState::default();
        let output = eval_line(
            &mut state,
            HeadlessEvalInput {
                line: "open \"hello world\"".to_string(),
            },
        )
        .expect("parse");
        assert_eq!(output.argv, vec!["open", "hello world"]);
    }

    #[test]
    fn tokenizes_escaped_whitespace() {
        let mut state = HeadlessShellState::default();
        let output = eval_line(
            &mut state,
            HeadlessEvalInput {
                line: "open hello\\ world".to_string(),
            },
        )
        .expect("parse");
        assert_eq!(output.argv, vec!["open", "hello world"]);
    }

    #[test]
    fn empty_command_reports_is_empty() {
        let mut state = HeadlessShellState::default();
        let output = eval_line(
            &mut state,
            HeadlessEvalInput {
                line: "   ".to_string(),
            },
        )
        .expect("parse");
        assert!(output.is_empty);
    }

    #[test]
    fn help_passthrough_is_detected() {
        let mut state = HeadlessShellState::default();
        let output = eval_line(
            &mut state,
            HeadlessEvalInput {
                line: "apps.list --help".to_string(),
            },
        )
        .expect("parse");
        assert!(output.wants_help);
    }

    #[test]
    fn invalid_quote_returns_error() {
        let mut state = HeadlessShellState::default();
        let error = eval_line(
            &mut state,
            HeadlessEvalInput {
                line: "open \"unterminated".to_string(),
            },
        )
        .expect_err("should fail");
        assert!(error.message.contains("unterminated"));
    }
}
