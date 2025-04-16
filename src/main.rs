use std::io::{stdin, stdout, Write};
use std::process::Command;
use std::env; // Add this for changing directories
use std::path::Path; // Add this for path operations

fn main() {
    println!("Well hello there and welcome to this shell interface I built, if you wanna leave just use exit");
    
    loop {
        print!("--> ");
        let _ = stdout().flush();
        
        let mut ip = String::new();
        stdin().read_line(&mut ip).unwrap();
        let cmd = ip.trim();
        
        if cmd.is_empty() { continue; }
        if cmd == "exit" {
            println!("Okee Byee");
            break;
        }
        
        let parts: Vec<&str> = cmd.split_whitespace().collect();
        let program = parts[0];
        let args = &parts[1..];
        
        // Handle built-in commands
        match program {
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