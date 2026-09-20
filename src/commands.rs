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
        (
            root.join("javetas.toml"),
            templates::javetas_toml(&name, package.as_deref()),
        ),
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

pub fn add_cmd(
    project: &Project,
    class: Option<&str>,
    package: Option<&str>,
    interface: bool,
) -> i32 {
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

    // `--package` wins over the project's package; otherwise inherit it.
    let package = match package {
        Some(p) if !p.is_empty() => Some(p.to_string()),
        _ => project.package.clone(),
    };
    if let Some(p) = &package
        && !valid_package(p)
    {
        return error(&format!("`{p}` is not a valid package name"));
    }

    // The folder follows the package: dots become directory separators.
    let dir = match &package {
        Some(p) => project.src_dir().join(p.replace('.', "/")),
        None => project.src_dir(),
    };
    if let Err(e) = fs::create_dir_all(&dir) {
        return error(&format!("cannot create directories: {e}"));
    }
    let file = dir.join(format!("{class}.java"));
    if file.exists() {
        return error(&format!("file already exists: {}", file.display()));
    }
    let content = if interface {
        templates::interface_java(package.as_deref(), &class)
    } else {
        templates::class_java(package.as_deref(), &class)
    };
    if let Err(e) = fs::write(&file, content) {
        return error(&format!("cannot write {}: {e}", file.display()));
    }

    ok(&format!("created {}", file.display()));
    0
}

fn is_apple_double(path: &Path) -> bool {
    path.file_name()
        .and_then(|n| n.to_str())
        .is_some_and(|n| n.starts_with("._"))
}

fn collect_java_files(dir: &Path, files: &mut Vec<PathBuf>) -> io::Result<()> {
    for entry in fs::read_dir(dir)? {
        let path = entry?.path();
        if path.is_dir() {
            collect_java_files(&path, files)?;
        } else if path.extension().and_then(|e| e.to_str()) == Some("java")
            && !is_apple_double(&path)
        {
            files.push(path);
        }
    }
    Ok(())
}

fn run_javac(project: &Project) -> i32 {
    let dep_code = crate::deps::ensure_dependencies(project);
    if dep_code != 0 {
        return dep_code;
    }

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

    let cp = project.classpath();
    let status = Process::new("javac")
        .arg("-d")
        .arg(&out)
        .arg("-cp")
        .arg(&cp)
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
    let cp = project.classpath();
    let status = Process::new("java")
        .arg("-cp")
        .arg(&cp)
        .arg(&full)
        .status();
    match status {
        Ok(s) => s.code().unwrap_or(1),
        Err(e) => error(&format!("cannot run java: {e}")),
    }
}

fn java_file_role(
    path: &Path,
    class_name: &str,
    full_class: &str,
    project_main: &str,
) -> &'static str {
    if full_class == project_main
        || class_name == project_main
        || project_main.ends_with(&format!(".{class_name}"))
    {
        return "[main]";
    }
    if let Ok(content) = fs::read_to_string(path) {
        if content.contains("public static void main") {
            return "[main]";
        }
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("//") || trimmed.starts_with('*') || trimmed.starts_with("/*") {
                continue;
            }
            let words: Vec<&str> = trimmed.split_whitespace().collect();
            if words.contains(&"interface") {
                return "[interface]";
            }
            if words.contains(&"class") {
                return "[class]";
            }
        }
    }
    "[class]"
}

struct TreeNode {
    name: String,
    is_dir: bool,
    annotation: Option<String>,
    children: Vec<TreeNode>,
}

fn should_ignore_entry(name: &str) -> bool {
    name == ".git"
        || name == ".codegraph"
        || name == "target"
        || name == "out"
        || name == ".DS_Store"
        || name.starts_with("._")
}

fn build_src_tree(dir: &Path, pkg_parts: &[String], project: &Project) -> Vec<TreeNode> {
    let mut nodes = Vec::new();
    let entries = match fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return nodes,
    };

    let mut items: Vec<PathBuf> = entries
        .filter_map(|e| e.ok().map(|ent| ent.path()))
        .filter(|p| {
            p.file_name()
                .and_then(|n| n.to_str())
                .is_some_and(|name| !should_ignore_entry(name))
        })
        .collect();

    items.sort_by(|a, b| {
        let a_is_dir = a.is_dir();
        let b_is_dir = b.is_dir();
        if a_is_dir != b_is_dir {
            b_is_dir.cmp(&a_is_dir)
        } else {
            a.file_name().cmp(&b.file_name())
        }
    });

    for path in items {
        let name = match path.file_name().and_then(|n| n.to_str()) {
            Some(n) => n.to_string(),
            None => continue,
        };

        if path.is_dir() {
            let mut next_pkg = pkg_parts.to_vec();
            next_pkg.push(name.clone());
            let pkg_str = next_pkg.join(".");
            let annotation = if valid_package(&pkg_str) {
                Some(style::dim(&format!("(package {pkg_str})")))
            } else {
                None
            };
            let children = build_src_tree(&path, &next_pkg, project);
            nodes.push(TreeNode {
                name,
                is_dir: true,
                annotation,
                children,
            });
        } else if path.extension().and_then(|e| e.to_str()) == Some("java") {
            let class_name = path.file_stem().and_then(|s| s.to_str()).unwrap_or(&name);
            let full_class = if pkg_parts.is_empty() {
                class_name.to_string()
            } else {
                format!("{}.{}", pkg_parts.join("."), class_name)
            };
            let role = java_file_role(&path, class_name, &full_class, &project.main);
            let role_style = match role {
                "[main]" => style::green(role),
                "[interface]" => style::yellow(role),
                _ => style::dim(role),
            };
            nodes.push(TreeNode {
                name,
                is_dir: false,
                annotation: Some(role_style),
                children: Vec::new(),
            });
        } else {
            nodes.push(TreeNode {
                name,
                is_dir: false,
                annotation: None,
                children: Vec::new(),
            });
        }
    }

    nodes
}

fn print_tree_nodes(
    nodes: &[TreeNode],
    prefix: &str,
    dir_count: &mut usize,
    java_count: &mut usize,
) {
    for (i, node) in nodes.iter().enumerate() {
        let is_last = i + 1 == nodes.len();
        let connector = if is_last { "└── " } else { "├── " };
        let new_prefix = if is_last {
            format!("{prefix}    ")
        } else {
            format!("{prefix}│   ")
        };

        let annot_str = match &node.annotation {
            Some(a) => format!(" {a}"),
            None => String::new(),
        };

        if node.is_dir {
            *dir_count += 1;
            println!("{prefix}{connector}{}/{annot_str}", node.name);
            print_tree_nodes(&node.children, &new_prefix, dir_count, java_count);
        } else {
            if node.name.ends_with(".java") {
                *java_count += 1;
            }
            println!("{prefix}{connector}{}{annot_str}", node.name);
        }
    }
}

fn build_lib_tree(dir: &Path) -> Vec<TreeNode> {
    let mut nodes = Vec::new();
    let entries = match fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return nodes,
    };
    let mut items: Vec<PathBuf> = entries
        .filter_map(|e| e.ok().map(|ent| ent.path()))
        .filter(|p| {
            p.file_name()
                .and_then(|n| n.to_str())
                .is_some_and(|name| !should_ignore_entry(name))
        })
        .collect();
    items.sort_by(|a, b| a.file_name().cmp(&b.file_name()));

    for path in items {
        let name = match path.file_name().and_then(|n| n.to_str()) {
            Some(n) => n.to_string(),
            None => continue,
        };
        let is_jar = path.extension().and_then(|e| e.to_str()) == Some("jar");
        let annotation = if is_jar {
            Some(style::dim("[dependency]"))
        } else {
            None
        };
        nodes.push(TreeNode {
            name,
            is_dir: path.is_dir(),
            annotation,
            children: Vec::new(),
        });
    }
    nodes
}

pub fn tree_cmd(project: &Project) -> i32 {
    let root_name = project
        .root
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("project");

    let pkg_info = match &project.package {
        Some(p) => format!(" {}", style::dim(&format!("(package {p})"))),
        None => String::new(),
    };
    println!("{}{pkg_info}", style::bold(&format!("{root_name}/")));

    let entries = match fs::read_dir(&project.root) {
        Ok(e) => e,
        Err(e) => return error(&format!("cannot read project directory: {e}")),
    };

    let mut items: Vec<PathBuf> = entries
        .filter_map(|e| e.ok().map(|ent| ent.path()))
        .filter(|p| {
            p.file_name()
                .and_then(|n| n.to_str())
                .is_some_and(|name| !should_ignore_entry(name))
        })
        .collect();

    items.sort_by(|a, b| {
        let a_name = a.file_name().and_then(|n| n.to_str()).unwrap_or("");
        let b_name = b.file_name().and_then(|n| n.to_str()).unwrap_or("");
        let a_order = if a_name == "src" {
            2
        } else if a_name == "lib" {
            1
        } else {
            0
        };
        let b_order = if b_name == "src" {
            2
        } else if b_name == "lib" {
            1
        } else {
            0
        };
        if a_order != b_order {
            a_order.cmp(&b_order)
        } else {
            let a_is_dir = a.is_dir();
            let b_is_dir = b.is_dir();
            if a_is_dir != b_is_dir {
                a_is_dir.cmp(&b_is_dir)
            } else {
                a.file_name().cmp(&b.file_name())
            }
        }
    });

    let mut root_nodes = Vec::new();
    for path in items {
        let name = match path.file_name().and_then(|n| n.to_str()) {
            Some(n) => n.to_string(),
            None => continue,
        };

        if name == "src" && path.is_dir() {
            let children = build_src_tree(&path, &[], project);
            root_nodes.push(TreeNode {
                name,
                is_dir: true,
                annotation: None,
                children,
            });
        } else if name == "lib" && path.is_dir() {
            let children = build_lib_tree(&path);
            root_nodes.push(TreeNode {
                name,
                is_dir: true,
                annotation: None,
                children,
            });
        } else if path.is_dir() {
            root_nodes.push(TreeNode {
                name,
                is_dir: true,
                annotation: None,
                children: Vec::new(),
            });
        } else {
            root_nodes.push(TreeNode {
                name,
                is_dir: false,
                annotation: None,
                children: Vec::new(),
            });
        }
    }

    let mut dir_count = 0;
    let mut java_count = 0;
    print_tree_nodes(&root_nodes, "", &mut dir_count, &mut java_count);

    let d_s = if dir_count == 1 { "directory" } else { "directories" };
    let f_s = if java_count == 1 { "Java file" } else { "Java files" };
    println!("\n{dir_count} {d_s}, {java_count} {f_s}");
    0
}
