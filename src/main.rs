mod builtins;
mod executor;

use builtins::{handle_builtin, BuiltinResult};
use executor::{execute_external, execute_pipeline};
use rustyline::error::ReadlineError;
use rustyline::DefaultEditor;
use std::collections::HashMap;
use std::env::{self};

// shellMgX: A simple shell interface designed for learning Rust.
// It supports built-in commands, environment variables, pipelines, and external programs.

// The main entry point of the shell.
// Initializes the Rustyline editor, loads command history, and starts the REPL loop.
fn main() {
    println!("Well hello there and welcome to this shell interface I built, if you wanna leave just use exit");
    println!("Use ohh toodles to call for help.");

    // Map for shell variables
    let mut shell_env: HashMap<String, String> = HashMap::new();
    // Setup Rustyline Editor
    let mut rl = DefaultEditor::new().expect("Failed to create rustyline editor");
    let history_file = match env::var("HOME") {
        Ok(home) => format!("{}/.shellMgX_history", home),
        Err(_) => ".shellMgX_history".to_string(),
    };
    let _ = rl.load_history(&history_file);

    // Main Program Loop
    loop {
        // Construct the prompt based on PS1 or fallback
        let prompt = if let Ok(ps1) = env::var("PS1") {
            ps1
        } else if let Some(ps1) = shell_env.get("PS1") {
            ps1.clone()
        } else {
            let current_dir = env::current_dir().unwrap_or_default();
            let dir_name = current_dir
                .file_name()
                .unwrap_or_default()
                .to_string_lossy();
            format!("{} --> ", dir_name)
        };

        // Taking User input via Rustyline
        let readline = rl.readline(&prompt);
        let ip = match readline {
            Ok(line) => {
                let _ = rl.add_history_entry(line.as_str());
                line
            }
            Err(ReadlineError::Interrupted) => {
                // Ctrl-C
                continue;
            }
            Err(ReadlineError::Eof) => {
                // Ctrl-D
                println!("Okee Byee");
                break;
            }
            Err(err) => {
                eprintln!("Error reading line: {:?}", err);
                break;
            }
        };
        let cmd = ip.trim();

        // Handling Empty commands and pipelines
        if cmd.is_empty() {
            continue;
        }
        if cmd.contains("|") {
            execute_pipeline(cmd, &shell_env);
            continue;
        }

        // Tokenizing parsing commands correctly using shlex
        let parts = match shlex::split(cmd) {
            Some(p) => p,
            None => {
                eprintln!("Error: mismatched quotes in command");
                continue;
            }
        };
        if parts.is_empty() {
            continue;
        }
        let program = parts[0].as_str();
        let args_vec: Vec<&str> = parts[1..].iter().map(|s| s.as_str()).collect();
        let args = &args_vec[..];

        // Attempt to execute as a built-in command first; fallback to external command
        match handle_builtin(program, args, &mut shell_env) {
            BuiltinResult::Exit => {
                break;
            }
            BuiltinResult::Handled => {
                continue;
            }
            BuiltinResult::NotBuiltin => {
                execute_external(program, args);
            }
        }
    }
    let _ = rl.save_history(&history_file);
}
