use std::io::{stdin, stdout, Write,Read};
use std::process::{Command, Stdio,Child}; // new Libriaries for pipes
use std::env::{self, args};
use std::path::Path;
use std::collections::HashMap;

fn main() {
    println!("Well hello there and welcome to this shell interface I built, if you wanna leave just use exit");
    println!("Use ohh toodles to call for help.");
    
    let mut shell_env: HashMap<String, String> = HashMap::new();
    
    loop {
        let current_dir = env::current_dir().unwrap_or_default();
        let dir_name = current_dir.file_name()
            .unwrap_or_default()
            .to_string_lossy();
        
        print!("{} --> ", dir_name);
        let _ = stdout().flush();
        
        let mut ip = String::new();
        stdin().read_line(&mut ip).unwrap();
        let cmd = ip.trim();
        
        if cmd.is_empty() { continue; }
        if cmd.contains("|") {
            execute_pipeline(cmd,&shell_env);
            continue;
        }
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
                        
                        
                        shell_env.insert(key.to_string(), value.to_string());
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
            "ohh" => {
                if args.len() > 0 && args[0] == "toodles" {
                    println!("Available built-in commands:");
                    println!("  cd [dir]      - Change directory (to home if no dir specified)");
                    println!("  pwd           - Print working directory");
                    println!("  echo [args]   - Print arguments (supports $VAR expansion)");
                    println!("  export KEY=VAL- Set environment variable");
                    println!("  unset KEY     - Unset environment variable");
                    println!("  env/printenv  - List all environment variables");
                    println!("  exit          - Exit the shell");
                    println!("  ohh toodles          - Show this help message");
                } else {
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
            },
            _ => {
               
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
fn execute_pipeline(cmd:&str, shell_env: &HashMap<String,String>) {
    let commands: Vec<&str> = cmd.split('|').map(|s| s.trim()).collect();

    if commands.is_empty() {
        eprintln!("Oi mate that pipeline seems empty, eh...");
        return;
    }

    let mut processes: Vec<Child> = Vec::new();

    for (i, cmd_str) in commands.iter().enumerate() {
        let parts: Vec<&str> = cmd_str.split_whitespace().collect();
        if parts.is_empty() { continue; }
        let program = parts[0];
        let args = &parts[1..];
        if ["cd", "export", "unset"].contains(&program) {
            eprintln!("Built-in command '{}' cannot be used in a pipeline", program);
            break;
        }
        let mut command = Command::new(program);
        command.args(args);
        if i == 0 {
            command.stdin(Stdio::inherit());
        } else {
            command.stdin(Stdio::piped());
        }
        if i == commands.len() - 1 {
            command.stdout(Stdio::inherit());
        } else {
            command.stdout(Stdio::piped());
        }
        match command.spawn() {
            Ok(child) => {
                processes.push(child);
            }
            Err(e) => {
                eprintln!("Failed to execute '{}': {}", cmd_str, e);
                break;
            }
        }
    }
    if processes.len() > 1 {
        for i in 0..processes.len() - 1 {
            if let Some(mut stdout) = processes[i].stdout.take() {
                if let Some(mut stdin) = processes[i + 1].stdin.take() {
                    std::thread::spawn(move || {
                        let mut buffer = [0; 1024];
                        loop {
                            match stdout.read(&mut buffer) {
                                Ok(0) => break, 
                                Ok(n) => {
                                    if stdin.write_all(&buffer[0..n]).is_err() {
                                        break;
                                    }
                                }
                                Err(_) => break,
                            }
                        }
                    });
                }
            }
        }
    }
    for process in &mut processes {
        let _ = process.wait();
    }
    
}