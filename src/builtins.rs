use crate::executor::parse_redirections;
use std::collections::HashMap;
use std::env;
use std::io::Write;
use std::path::Path;

// Represents the status of a command after checking against shell built-ins.
pub enum BuiltinResult {
    // The command was a recognized built-in and executed successfully.
    Handled,
    // The command is not a shell built-in and should be executed as an external program.
    NotBuiltin,
    // The shell was instructed to terminate (e.g., via `exit`).
    Exit,
}

// Intercepts and executes shell built-in commands (`cd`, `pwd`, `export`, etc.).

// Updates the `shell_env` map or the process's native environment variables
// when state changes occur. Returns a `BuiltinResult` indicating resolution status.
pub fn handle_builtin(
    program: &str,
    args: &[&str],
    shell_env: &mut HashMap<String, String>,
) -> BuiltinResult {
    let (filtered_args, _stdin_opt, mut stdout_opt) = parse_redirections(args);
    let args = &filtered_args;

    match program {
        "exit" => {
            println!("Okee Byee");
            BuiltinResult::Exit
        }
        "cd" => {
            let new_dir = if args.is_empty() {
                match env::var("HOME") {
                    Ok(home) => home,
                    Err(_) => {
                        eprintln!("Could not find home directory");
                        return BuiltinResult::Handled;
                    }
                }
            } else {
                args[0].to_string()
            };

            let path = Path::new(&new_dir);
            if let Err(e) = env::set_current_dir(&path) {
                eprintln!("Failed to change directory: {}", e);
            }
            BuiltinResult::Handled
        }
        "pwd" => {
            match env::current_dir() {
                Ok(path) => {
                    let output = format!("{}", path.display());
                    if let Some(ref mut file) = stdout_opt {
                        let _ = writeln!(file, "{}", output);
                    } else {
                        println!("{}", output);
                    }
                }
                Err(e) => eprintln!("Failed to get current directory: {}", e),
            }
            BuiltinResult::Handled
        }
        "echo" => {
            let mut output = Vec::new();
            for arg in args {
                if arg.starts_with("$") {
                    let var_name = &arg[1..];
                    if var_name == "$" {
                        output.push(std::process::id().to_string());
                    } else if let Ok(value) = env::var(var_name) {
                        output.push(value);
                    } else if let Some(value) = shell_env.get(var_name) {
                        output.push(value.clone());
                    } else {
                        output.push(String::new());
                    }
                } else {
                    output.push(arg.to_string());
                }
            }
            let final_output = output.join(" ");
            if let Some(ref mut file) = stdout_opt {
                let _ = writeln!(file, "{}", final_output);
            } else {
                println!("{}", final_output);
            }
            BuiltinResult::Handled
        }
        "export" => {
            for arg in args {
                if let Some(pos) = arg.find('=') {
                    let key = &arg[..pos];
                    let value = &arg[pos + 1..];

                    shell_env.insert(key.to_string(), value.to_string());
                    unsafe {
                        env::set_var(key, value);
                    }

                    println!("{}={}", key, value);
                } else {
                    eprintln!("Invalid export syntax: {}", arg);
                }
            }
            BuiltinResult::Handled
        }
        "unset" => {
            for arg in args {
                let key = arg.to_string();
                shell_env.remove(&key);

                unsafe {
                    env::remove_var(arg);
                }
            }
            BuiltinResult::Handled
        }
        "env" | "printenv" => {
            for (key, value) in env::vars() {
                if let Some(ref mut file) = stdout_opt {
                    let _ = writeln!(file, "{}={}", key, value);
                } else {
                    println!("{}={}", key, value);
                }
            }
            for (key, value) in shell_env.iter() {
                if env::var(key).is_err() {
                    if let Some(ref mut file) = stdout_opt {
                        let _ = writeln!(file, "{}={}", key, value);
                    } else {
                        println!("{}={}", key, value);
                    }
                }
            }
            BuiltinResult::Handled
        }
        "ohh" => {
            if !args.is_empty() && args[0] == "toodles" {
                println!("Available built-in commands:");
                println!("  cd [dir]      - Change directory (to home if no dir specified)");
                println!("  pwd           - Print working directory");
                println!("  echo [args]   - Print arguments (supports $VAR expansion)");
                println!("  export KEY=VAL- Set environment variable");
                println!("  unset KEY     - Unset environment variable");
                println!("  env/printenv  - List all environment variables");
                println!("  exit          - Exit the shell");
                println!("  ohh toodles   - Show this help message");
                BuiltinResult::Handled
            } else {
                BuiltinResult::NotBuiltin
            }
        }
        _ => BuiltinResult::NotBuiltin,
    }
}
