use std::io::stdin;
use std::io::stdout;
use std::io::Write;
use std::process::Command;
fn main() {
    println!("Well hello there and welcome to this shell interface I built, if you wanna leave just use exit");
    loop {
        print!("--> ");
        let _ = stdout().flush();

        let mut ip = String::new();
        stdin().read_line(&mut ip).unwrap();

        let cmd = ip.trim();
        if cmd.is_empty() {continue;}
        if cmd == "exit" {
            println!("Okee Byee");
            break;
        }

        let parts: Vec<&str> = cmd.split_whitespace().collect();
        let program = parts[0];
        let args = &parts[1..];

        let child = Command::new(program).args(args).spawn();
        
        match child {
            Ok(mut child_proc) => {
                child_proc.wait().unwrap();
            }
            Err(_e) => {
                eprintln!("Could run that mate, sorry!");
            }
        } 
    }
}
