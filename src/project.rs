use std::fs;
use std::path::{Path, PathBuf};

pub struct Project {
    pub root: PathBuf,
    pub package: Option<String>,
    pub main: String,
}

impl Project {
    /// Walks up from the current directory looking for a `.javetas` file.
    pub fn discover() -> Result<Project, String> {
        let cwd = std::env::current_dir()
            .map_err(|e| format!("cannot read the current directory: {e}"))?;
        let mut dir: Option<&Path> = Some(&cwd);
        while let Some(d) = dir {
            let config = d.join(".javetas");
            if config.is_file() {
                return parse(&config).map(|(package, main)| Project {
                    root: d.to_path_buf(),
                    package,
                    main,
                });
            }
            dir = d.parent();
        }
        Err("not inside a javetas project (no .javetas file found)".into())
    }

    pub fn src_dir(&self) -> PathBuf {
        self.root.join("src")
    }

    pub fn out_dir(&self) -> PathBuf {
        self.root.join("out")
    }

    /// Full class name for `java -cp out ...`.
    pub fn full_class(&self, name: &str) -> String {
        match &self.package {
            Some(p) if !name.contains('.') => format!("{p}.{name}"),
            _ => name.to_string(),
        }
    }
}

fn parse(path: &Path) -> Result<(Option<String>, String), String> {
    let text =
        fs::read_to_string(path).map_err(|e| format!("cannot read {}: {e}", path.display()))?;
    let mut package = None;
    let mut main = "Main".to_string();
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let (key, value) = line
            .split_once('=')
            .ok_or_else(|| format!("invalid line in {}: {line}", path.display()))?;
        match key.trim() {
            "package" => {
                package = if value.trim().is_empty() {
                    None
                } else {
                    Some(value.trim().to_string())
                };
            }
            "main" => main = value.trim().to_string(),
            _ => {}
        }
    }
    Ok((package, main))
}
