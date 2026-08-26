pub enum Command {
    Help,
    Version,
    New {
        name: Option<String>,
        package: Option<String>,
    },
    Add {
        class: Option<String>,
    },
    Build,
    Run {
        class: Option<String>,
    },
}

pub fn parse(args: &[String]) -> Result<Command, String> {
    if args.is_empty() {
        return Ok(Command::Help);
    }

    match args[0].as_str() {
        "-h" | "--help" | "help" => Ok(Command::Help),
        "-v" | "--version" | "version" => Ok(Command::Version),
        "new" => {
            let mut name = None;
            let mut package = None;
            let mut i = 1;
            while i < args.len() {
                let arg = &args[i];
                if arg == "--package" || arg == "-p" {
                    i += 1;
                    if i >= args.len() {
                        return Err(
                            "`--package` needs a value (e.g. `--package com.ejemplo`)".into()
                        );
                    }
                    package = Some(args[i].clone());
                } else if let Some(value) = arg.strip_prefix("--package=") {
                    package = Some(value.to_string());
                } else if arg.starts_with('-') {
                    return Err(format!("unknown flag: {arg}"));
                } else if name.is_none() {
                    name = Some(arg.clone());
                } else {
                    return Err(format!("unexpected argument: {arg}"));
                }
                i += 1;
            }
            Ok(Command::New { name, package })
        }
        "add" => {
            let mut class = None;
            for arg in &args[1..] {
                if arg.starts_with('-') {
                    return Err(format!("unknown flag: {arg}"));
                }
                if class.is_some() {
                    return Err(format!("unexpected argument: {arg}"));
                }
                class = Some(arg.clone());
            }
            Ok(Command::Add { class })
        }
        "build" => {
            if args.len() > 1 {
                return Err(format!("unexpected argument: {}", args[1]));
            }
            Ok(Command::Build)
        }
        "run" => {
            let mut class = None;
            for arg in &args[1..] {
                if arg.starts_with('-') {
                    return Err(format!("unknown flag: {arg}"));
                }
                if class.is_some() {
                    return Err(format!("unexpected argument: {arg}"));
                }
                class = Some(arg.clone());
            }
            Ok(Command::Run { class })
        }
        other => Err(format!("unknown command: {other}")),
    }
}
