use crate::project::Project;
use crate::style;
use crate::templates;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::Command as Process;

pub(crate) fn error(msg: &str) -> i32 {
    eprintln!("{} {msg}", style::red("error:"));
    1
}

pub(crate) fn ok(msg: &str) {
    println!("{} {msg}", style::green("ok:"));
}

pub(crate) fn warn(msg: &str) {
    eprintln!("{} {msg}", style::yellow("warn:"));
}

pub(crate) fn prompt(label: &str) -> Option<String> {
    print!("{} ", style::yellow(&format!("? {label}")));
    let _ = io::stdout().flush();
    let mut line = String::new();
    match io::stdin().read_line(&mut line) {
        Ok(_) => Some(line.trim().to_string()),
        Err(e) => {
            eprintln!("{} failed to read input: {e}", style::red("error:"));
            None
        }
    }
}

fn valid_identifier(s: &str) -> bool {
    let mut chars = s.chars();
    match chars.next() {
        Some(c) if c.is_ascii_alphabetic() || c == '_' || c == '$' => {}
        _ => return false,
    }
    chars.all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '$')
}

fn valid_package(p: &str) -> bool {
    !p.is_empty() && p.split('.').all(valid_identifier)
}

pub fn new_cmd(name: Option<&str>, package: Option<&str>) -> i32 {
    let name = match name {
        Some(n) if !n.is_empty() => n.to_string(),
        _ => match prompt("Project name:") {
            Some(n) if !n.is_empty() => n,
            _ => return error("a project name is required"),
        },
    };
    if name.contains('/') || name.contains('\\') || name.contains(' ') {
        return error(&format!("`{name}` is not a valid project name"));
    }

    let package = match package {
        Some(p) if !p.is_empty() => Some(p.to_string()),
        _ => match prompt("Package (optional, e.g. com.ejemplo):") {
            Some(p) if !p.is_empty() => Some(p),
            _ => None,
        },
    };
    if let Some(p) = &package
        && !valid_package(p)
    {
        return error(&format!("`{p}` is not a valid package name"));
    }

    let root = PathBuf::from(&name);
    if root.exists() {
        return error(&format!("directory already exists: {name}"));
    }

    let class_dir = match &package {
        Some(p) => root.join("src").join(p.replace('.', "/")),
        None => root.join("src"),
    };
    if let Err(e) = fs::create_dir_all(&class_dir) {
        return error(&format!("cannot create directories: {e}"));
    }

    let main_class = package
        .as_deref()
        .map_or_else(|| "Main".to_string(), |p| format!("{p}.Main"));

    let files: Vec<(PathBuf, String)> = vec![
        (root.join(".gitignore"), templates::gitignore()),
        (root.join(".javetas"), templates::config(package.as_deref())),
        (root.join("Makefile"), templates::makefile(&main_class)),
        (
            root.join("README.md"),
            templates::readme(&name, &main_class),
        ),
        (
            class_dir.join("Main.java"),
            templates::main_java(package.as_deref()),
        ),
    ];
    for (path, content) in files {
        if let Err(e) = fs::write(&path, content) {
            return error(&format!("cannot write {}: {e}", path.display()));
        }
    }

    ok(&format!("created project {}", style::bold(&name)));
    println!("{}", style::dim(&format!("  next: cd {name} && make run")));
    0
}

pub fn add_cmd(project: &Project, class: Option<&str>) -> i32 {
    let class = match class {
        Some(c) if !c.is_empty() => c.to_string(),
        _ => match prompt("Class name:") {
            Some(c) if !c.is_empty() => c,
            _ => return error("a class name is required"),
        },
    };
    if !valid_identifier(&class) {
        return error(&format!("`{class}` is not a valid class name"));
    }
    if class.starts_with(|c: char| c.is_ascii_lowercase()) {
        warn(&format!(
            "class names usually start with an uppercase letter (`{class}`)"
        ));
    }

    let dir = project.class_dir();
    if let Err(e) = fs::create_dir_all(&dir) {
        return error(&format!("cannot create directories: {e}"));
    }
    let file = dir.join(format!("{class}.java"));
    if file.exists() {
        return error(&format!("file already exists: {}", file.display()));
    }
    let content = templates::class_java(project.package.as_deref(), &class);
    if let Err(e) = fs::write(&file, content) {
        return error(&format!("cannot write {}: {e}", file.display()));
    }

    ok(&format!("created {}", file.display()));
    0
}

fn collect_java_files(dir: &Path, files: &mut Vec<PathBuf>) -> io::Result<()> {
    for entry in fs::read_dir(dir)? {
        let path = entry?.path();
        if path.is_dir() {
            collect_java_files(&path, files)?;
        } else if path.extension().and_then(|e| e.to_str()) == Some("java") {
            files.push(path);
        }
    }
    Ok(())
}

fn run_javac(project: &Project) -> i32 {
    let src = project.src_dir();
    if !src.is_dir() {
        return error("no src/ directory found (are you in a javetas project?)");
    }
    let mut files = Vec::new();
    if let Err(e) = collect_java_files(&src, &mut files) {
        return error(&format!("cannot scan src/: {e}"));
    }
    if files.is_empty() {
        return error("no .java files found in src/");
    }

    let out = project.out_dir();
    if let Err(e) = fs::create_dir_all(&out) {
        return error(&format!("cannot create {}: {e}", out.display()));
    }

    let status = Process::new("javac")
        .arg("-d")
        .arg(&out)
        .args(&files)
        .status();
    match status {
        Ok(s) if s.success() => {
            ok(&format!(
                "compiled {} file(s) -> {}/",
                files.len(),
                out.display()
            ));
            0
        }
        Ok(s) => {
            eprintln!("{} compilation failed", style::red("error:"));
            s.code().unwrap_or(1)
        }
        Err(e) => error(&format!("cannot run javac: {e}")),
    }
}

pub fn build_cmd(project: &Project) -> i32 {
    run_javac(project)
}

pub fn run_cmd(project: &Project, class: Option<&str>) -> i32 {
    let code = run_javac(project);
    if code != 0 {
        return code;
    }
    let name = match class {
        Some(c) if !c.is_empty() => c.to_string(),
        _ => project.main.clone(),
    };
    let full = project.full_class(&name);
    let status = Process::new("java")
        .arg("-cp")
        .arg(project.out_dir())
        .arg(&full)
        .status();
    match status {
        Ok(s) => s.code().unwrap_or(1),
        Err(e) => error(&format!("cannot run java: {e}")),
    }
}
