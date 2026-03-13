# shellMgX

In this project, I aim to make a simple shell using Rust. Before I start this entry, I would like to describe exactly what I mean by the shell here, because a shell natively is something much more complex than what we're trying to create. A shell is something that allows one to control the computer, not always in obvious means. Now the most common way to interact with a shell is through some sort of terminal like (CUI) interface, and that is sort of what I am going to try to build.

Editorial --> https://www.notion.so/shellMgX-A-Rust-Based-Shell-Interface-Editorial-1dffe2354e4480f093affeb6c0e476bf?pvs=4

## Features
- **Built-in commands**: `cd`, `pwd`, `echo`, `export`, `unset`, `env`/`printenv`, `exit`, and `ohh toodles` (help).
- **Environment variables**: Support for setting and expanding `$VAR` and customizable `$PS1` prompt.
- **Robust UI**: Powered by `rustyline` for arrow-key command history and Emacs/Vi bindings.
- **Accurate Quoting**: Uses `shlex` to correctly parse strings like `echo "hello world"`.
- **Command Pipelines**: Support for piping commands (`|`) using native OS-level pipes.
- **I/O Redirection**: Supports output (`>`) and input (`<`) redirection natively.

## To Run
- Clone the repository
- Navigate to the project directory
- Use `cargo run`
- Use the command `ohh toodles` for help

## What's New With Version 2.0
- Implemented rustyline so shellMgX supports arrow keys, vim line-editing shortcuts and persistent command logging
- Implemented parsing of quote endings
- Refactored main.rs, it is now split into 3 files for easier upkeeping
- Custom string thread-buffered pipelines have been eliminated, meaning pipelines operate asynchronously as close to the target C-syscall speeds as natively possible.
- Added support to redirect files into applications
