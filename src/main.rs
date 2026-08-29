mod args;
mod commands;
mod project;
mod style;
mod templates;
mod update;

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
  javetas add <Name> [--package <pkg>] [--interface]
                                     Add a class (or an interface with --interface)
  javetas build                          Compile all sources into out/
  javetas run [ClassName]                Compile and run (default: Main)
  javetas update [--yes]                 Self-update to the latest release
  javetas help                           Show this help
  javetas version                        Show the version

Options:
  -p, --package <pkg>   Package for a new project or class (e.g. com.ejemplo)
  -i, --interface       With `add`: create an interface instead of a class
  -y, --yes             Skip the update confirmation prompt
  -h, --help            Show this help
  -v, --version         Show the version

Examples:
  javetas new demo
  javetas new demo --package com.ejemplo
  cd demo
  javetas add Persona
  javetas add Persona --package com.otro
  javetas add Contrato --interface
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
        Command::Add {
            class,
            package,
            interface,
        } => {
            with_project(|p| commands::add_cmd(p, class.as_deref(), package.as_deref(), interface))
        }
        Command::Build => with_project(commands::build_cmd),
        Command::Run { class } => with_project(|p| commands::run_cmd(p, class.as_deref())),
        Command::Update { yes } => update::update_cmd(yes),
    };

    ExitCode::from(code as u8)
}
