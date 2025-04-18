use std::io::{stdin, stdout, Write};
use std::process::Command;
use std::env;
use std::path::Path;
use std::collections::HashMap;

fn main() {
    println!("Well hello there and welcome to this shell interface I built, if you wanna leave just use exit");
    
    // Create a map to store environment variables
    let mut shell_env: HashMap<String, String> = HashMap::new();
    
    loop {
        // Display current directory in prompt
        let current_dir = env::current_dir().unwrap_or_default();
        let dir_name = current_dir.file_name()
            .unwrap_or_default()
            .to_string_lossy();
        
        print!("[{}] --> ", dir_name);
        let _ = stdout().flush();
        
        let mut ip = String::new();
        stdin().read_line(&mut ip).unwrap();
        let cmd = ip.trim();
        
        if cmd.is_empty() { continue; }
        
        let parts: Vec<&str> = cmd.split_whitespace().collect();
        let program = parts[0];
        let args = &parts[1..];
        
        // Handle built-in commands
        match program {
            "exit" => {
                println!("Okee Byee");
                break;
            },
            "cd" => {
                let new_dir = if args.is_empty() {
                    // If no argument is provided, change to the home directory
                    match env::var("HOME") {
                        Ok(home) => home,
                        Err(_) => {
                            eprintln!("Could not find home directory");
                            continue;
                        }
                    }
                } else {
                    args[0].to_string()
                };
                
                let path = Path::new(&new_dir);
                if let Err(e) = env::set_current_dir(&path) {
                    eprintln!("Failed to change directory: {}", e);
                }
            },
            "pwd" => {
                match env::current_dir() {
                    Ok(path) => println!("{}", path.display()),
                    Err(e) => eprintln!("Failed to get current directory: {}", e),
                }
            },
            "echo" => {
                let mut output = Vec::new();
                for arg in args {
                    if arg.starts_with("$") {
                        let var_name = &arg[1..];
                        if var_name == "$" {
                            // Handle $$ (process id)
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
                println!("{}", output.join(" "));
            },
            "export" => {
                for arg in args {
                    if let Some(pos) = arg.find('=') {
                        let key = &arg[..pos];
                        let value = &arg[pos+1..];
                        
                        // Update both our shell's environment and the process environment
                        shell_env.insert(key.to_string(), value.to_string());
                        
                        // Wrap unsafe call in an unsafe block
                        unsafe {
                            env::set_var(key, value);
                        }
                        
                        println!("{}={}", key, value);
                    } else {
                        eprintln!("Invalid export syntax: {}", arg);
                    }
                }
            },
            "unset" => {
                for arg in args {
                    let key = arg.to_string();
                    shell_env.remove(&key);
                    
                    unsafe {
                        env::remove_var(arg);
                    }
                }
            },
            "env" | "printenv" => {
                for (key, value) in env::vars() {
                    println!("{}={}", key, value);
                }
                for (key, value) in &shell_env {
                    if env::var(key).is_err() {
                        println!("{}={}", key, value);
                    }
                }
            },
            "help" => {
                println!("Available built-in commands:");
                println!("  cd [dir]      - Change directory (to home if no dir specified)");
                println!("  pwd           - Print working directory");
                println!("  echo [args]   - Print arguments (supports $VAR expansion)");
                println!("  export KEY=VAL- Set environment variable");
                println!("  unset KEY     - Unset environment variable");
                println!("  env/printenv  - List all environment variables");
                println!("  exit          - Exit the shell");
                println!("  help          - Show this help message");
            },
            _ => {
                // Handle external commands
                let child = Command::new(program).args(args).spawn();
                match child {
                    Ok(mut child_proc) => {
                        child_proc.wait().unwrap();
                    }
                    Err(_e) => {
                        eprintln!("Could not run that mate, sorry!");
                    }
                }
            }
        }
    }
}