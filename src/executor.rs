use std::collections::HashMap;
use std::fs::File;
use std::process::{Child, Command, Stdio};

// Parses arguments for standard I/O redirection tokens (`<`, `>`).
//
// Extracts file paths and attempts to open or create the necessary files.
// Returns a tuple containing:
// - Filtered arguments (redirection tokens and paths removed)
// - Optional file for stdin redirection
// - Optional file for stdout redirection
pub fn parse_redirections<'a>(args: &[&'a str]) -> (Vec<&'a str>, Option<File>, Option<File>) {
    let mut filtered_args = Vec::new();
    let mut i = 0;
    let mut custom_stdin = None;
    let mut custom_stdout = None;

    while i < args.len() {
        if args[i].starts_with('<') && args[i] != "<" {
            let filename = &args[i][1..];
            if let Ok(file) = File::open(filename) {
                custom_stdin = Some(file);
            } else {
                eprintln!("Error opening file: {}", filename);
            }
            i += 1;
        } else if args[i] == "<" && i + 1 < args.len() {
            if let Ok(file) = File::open(args[i + 1]) {
                custom_stdin = Some(file);
            } else {
                eprintln!("Error opening file: {}", args[i + 1]);
            }
            i += 2;
        } else if args[i].starts_with('>') && args[i] != ">" {
            let filename = &args[i][1..];
            if let Ok(file) = File::create(filename) {
                custom_stdout = Some(file);
            } else {
                eprintln!("Error creating file: {}", filename);
            }
            i += 1;
        } else if args[i] == ">" && i + 1 < args.len() {
            if let Ok(file) = File::create(args[i + 1]) {
                custom_stdout = Some(file);
            } else {
                eprintln!("Error creating file: {}", args[i + 1]);
            }
            i += 2;
        } else {
            filtered_args.push(args[i]);
            i += 1;
        }
    }

    (filtered_args, custom_stdin, custom_stdout)
}

// Executes a pipeline sequence (`cmd1 | cmd2 | ...`).
//
// Iterates over commands split by the `|` token, connecting the `stdout` of
// one process directly to the `stdin` of the next natively via OS pipes.
pub fn execute_pipeline(cmd: &str, _shell_env: &HashMap<String, String>) {
    let commands: Vec<&str> = cmd.split('|').map(|s| s.trim()).collect();

    if commands.is_empty() {
        eprintln!("Oi mate that pipeline seems empty, eh...");
        return;
    }

    let mut processes: Vec<Child> = Vec::new();
    let mut previous_stdout = None;

    for (i, cmd_str) in commands.iter().enumerate() {
        let parts = match shlex::split(cmd_str) {
            Some(p) => p,
            None => {
                eprintln!("Error: mismatched quotes in pipeline segment");
                break;
            }
        };
        if parts.is_empty() {
            continue;
        }
        let program = parts[0].as_str();
        let args_vec: Vec<&str> = parts[1..].iter().map(|s| s.as_str()).collect();
        let args = &args_vec[..];

        if ["cd", "export", "unset"].contains(&program) {
            eprintln!("Oi mate sorry there can't use '{}' in a pipeline", program);
            break;
        }

        let (filtered_args, stdin_opt, stdout_opt) = parse_redirections(args);
        let mut command = Command::new(program);
        command.args(&filtered_args);

        if let Some(file) = stdin_opt {
            command.stdin(Stdio::from(file));
        } else if let Some(stdout) = previous_stdout.take() {
            command.stdin(Stdio::from(stdout));
        } else {
            command.stdin(Stdio::inherit());
        }

        if let Some(file) = stdout_opt {
            command.stdout(Stdio::from(file));
        } else if i == commands.len() - 1 {
            command.stdout(Stdio::inherit());
        } else {
            command.stdout(Stdio::piped());
        }

        match command.spawn() {
            Ok(mut child) => {
                if i < commands.len() - 1 {
                    previous_stdout = child.stdout.take();
                }
                processes.push(child);
            }
            Err(e) => {
                eprintln!("Failed to execute '{}': {}", program, e);
                break;
            }
        }
    }

    for process in &mut processes {
        let _ = process.wait();
    }
}

// Spawns an external application or command.
//
// Applies I/O redirection correctly before creating the OS-level subprocess wait lock.
pub fn execute_external(program: &str, args: &[&str]) {
    let (filtered_args, stdin_opt, stdout_opt) = parse_redirections(args);
    let mut command = Command::new(program);
    command.args(&filtered_args);

    if let Some(file) = stdin_opt {
        command.stdin(Stdio::from(file));
    }
    if let Some(file) = stdout_opt {
        command.stdout(Stdio::from(file));
    }

    let child = command.spawn();
    match child {
        Ok(mut child_proc) => {
            let _ = child_proc.wait();
        }
        Err(_e) => {
            eprintln!("Could not run that mate, sorry!");
        }
    }
}
