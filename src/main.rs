mod args;
mod commands;
mod project;
mod style;
mod templates;

use args::{Command, parse};
use project::Project;
use std::process::ExitCode;

const VERSION: &str = env!("CARGO_PKG_VERSION");

fn print_help() {
    println!(
        "javetas {VERSION}
Create and manage simple Java projects for learning.

Usage:
  javetas new <name> [--package <pkg>]   Create a new project
  javetas add <ClassName>                Add a class to the current project
  javetas build                          Compile all sources into out/
  javetas run [ClassName]                Compile and run (default: Main)
  javetas help                           Show this help
  javetas version                        Show the version

Options:
  -p, --package <pkg>   Package for the new project (e.g. com.ejemplo)
  -h, --help            Show this help
  -v, --version         Show the version

Examples:
  javetas new demo
  javetas new demo --package com.ejemplo
  cd demo
  javetas add Persona
  javetas run"
    );
}

fn with_project(f: impl FnOnce(&Project) -> i32) -> i32 {
    match Project::discover() {
        Ok(project) => f(&project),
        Err(e) => {
            eprintln!("{} {e}", style::red("error:"));
            eprintln!(
                "{} run `javetas new <name>` first, or cd into a project",
                style::dim("hint:")
            );
            1
        }
    }
}

fn main() -> ExitCode {
    let raw: Vec<String> = std::env::args().skip(1).collect();
    let command = match parse(&raw) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("{} {e}", style::red("error:"));
            eprintln!("{} run `javetas help` for usage", style::dim("hint:"));
            return ExitCode::from(1);
        }
    };

    let code = match command {
        Command::Help => {
            print_help();
            0
        }
        Command::Version => {
            println!("javetas {VERSION}");
            0
        }
        Command::New { name, package } => commands::new_cmd(name.as_deref(), package.as_deref()),
        Command::Add { class } => with_project(|p| commands::add_cmd(p, class.as_deref())),
        Command::Build => with_project(commands::build_cmd),
        Command::Run { class } => with_project(|p| commands::run_cmd(p, class.as_deref())),
    };

    ExitCode::from(code as u8)
}
