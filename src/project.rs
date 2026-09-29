use std::fs;
use std::path::{Path, PathBuf};

#[derive(Clone, Debug)]
pub struct Dependency {
    pub alias: String,
    pub group_id: String,
    pub artifact_id: String,
    pub version: String,
}

impl Dependency {
    pub fn parse_coord(alias: &str, coord: &str) -> Result<Dependency, String> {
        let parts: Vec<&str> = coord.split(':').collect();
        if parts.len() != 3 {
            return Err(format!(
                "invalid dependency `{coord}` for `{alias}`: expected `groupId:artifactId:version`"
            ));
        }
        let group_id = parts[0].trim();
        let artifact_id = parts[1].trim();
        let version = parts[2].trim();
        if group_id.is_empty() || artifact_id.is_empty() || version.is_empty() {
            return Err(format!(
                "invalid dependency `{coord}` for `{alias}`: parts cannot be empty"
            ));
        }
        Ok(Dependency {
            alias: alias.to_string(),
            group_id: group_id.to_string(),
            artifact_id: artifact_id.to_string(),
            version: version.to_string(),
        })
    }

    pub fn jar_name(&self) -> String {
        format!("{}-{}.jar", self.artifact_id, self.version)
    }

    pub fn maven_url(&self) -> String {
        let group_path = self.group_id.replace('.', "/");
        format!(
            "https://repo1.maven.org/maven2/{}/{}/{}/{}",
            group_path,
            self.artifact_id,
            self.version,
            self.jar_name()
        )
    }
}

pub struct Project {
    pub root: PathBuf,
    pub package: Option<String>,
    pub main: String,
    pub dependencies: Vec<Dependency>,
}

impl Project {
    /// Walks up from the current directory looking for `javetas.toml` or `.javetas`.
    pub fn discover() -> Result<Project, String> {
        let cwd = std::env::current_dir()
            .map_err(|e| format!("cannot read the current directory: {e}"))?;
        let mut dir: Option<&Path> = Some(&cwd);
        while let Some(d) = dir {
            let toml_config = d.join("javetas.toml");
            if toml_config.is_file() {
                return parse_toml(&toml_config).map(|(package, main, dependencies)| Project {
                    root: d.to_path_buf(),
                    package,
                    main,
                    dependencies,
                });
            }
            let legacy_config = d.join(".javetas");
            if legacy_config.is_file() {
                return parse_legacy(&legacy_config).map(|(package, main)| Project {
                    root: d.to_path_buf(),
                    package,
                    main,
                    dependencies: Vec::new(),
                });
            }
            dir = d.parent();
        }
        Err("not inside a javetas project (no javetas.toml or .javetas file found)".into())
    }

    pub fn src_dir(&self) -> PathBuf {
        self.root.join("src")
    }

    pub fn out_dir(&self) -> PathBuf {
        self.root.join("out")
    }

    pub fn lib_dir(&self) -> PathBuf {
        self.root.join("lib")
    }

    pub fn has_dependencies(&self) -> bool {
        !self.dependencies.is_empty() || self.lib_dir().is_dir()
    }

    /// Full class name for `java -cp ...`.
    pub fn full_class(&self, name: &str) -> String {
        match &self.package {
            Some(p) if !name.contains('.') => format!("{p}.{name}"),
            _ => name.to_string(),
        }
    }

    pub fn manifest_path(&self) -> PathBuf {
        self.root.join("javetas.toml")
    }

    /// Removes a dependency from javetas.toml by alias.
    /// Returns the removed Dependency struct.
    pub fn remove_dependency(&self, alias: &str) -> Result<Dependency, String> {
        let manifest = self.manifest_path();
        if !manifest.is_file() {
            return Err("no javetas.toml found in this project".into());
        }

        let dep = self
            .dependencies
            .iter()
            .find(|d| d.alias == alias)
            .cloned()
            .ok_or_else(|| format!("dependency `{alias}` not found in javetas.toml"))?;

        let text = fs::read_to_string(&manifest)
            .map_err(|e| format!("cannot read {}: {e}", manifest.display()))?;

        let mut new_lines = Vec::new();
        let mut in_deps = false;
        let mut removed = false;

        for line in text.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with('[') && trimmed.ends_with(']') {
                in_deps = trimmed[1..trimmed.len() - 1].trim() == "dependencies";
                new_lines.push(line);
                continue;
            }

            if in_deps && !removed {
                if let Some((key, _)) = trimmed.split_once('=') {
                    if key.trim() == alias {
                        removed = true;
                        continue;
                    }
                }
            }

            new_lines.push(line);
        }

        if !removed {
            return Err(format!("dependency `{alias}` not found in javetas.toml"));
        }

        let mut output = new_lines.join("\n");
        if text.ends_with('\n') {
            output.push('\n');
        }

        fs::write(&manifest, output)
            .map_err(|e| format!("cannot write {}: {e}", manifest.display()))?;

        Ok(dep)
    }

    /// Returns classpath string combining out/ and lib/*.
    pub fn classpath(&self) -> String {
        let out = self.out_dir();
        let out_str = out.display().to_string();
        if !self.has_dependencies() {
            return out_str;
        }

        let lib = self.lib_dir();
        let sep = if cfg!(windows) { ';' } else { ':' };
        let lib_wildcard = if cfg!(windows) {
            format!("{}\\*", lib.display())
        } else {
            format!("{}/*", lib.display())
        };
        format!("{out_str}{sep}{lib_wildcard}")
    }
}

fn parse_toml(path: &Path) -> Result<(Option<String>, String, Vec<Dependency>), String> {
    let text =
        fs::read_to_string(path).map_err(|e| format!("cannot read {}: {e}", path.display()))?;
    let mut package = None;
    let mut main = "Main".to_string();
    let mut dependencies = Vec::new();
    let mut current_section = "";

    for (line_idx, line) in text.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if line.starts_with('[') && line.ends_with(']') {
            current_section = line[1..line.len() - 1].trim();
            continue;
        }

        let (key, value) = line.split_once('=').ok_or_else(|| {
            format!(
                "invalid syntax in {} line {}: {line}",
                path.display(),
                line_idx + 1
            )
        })?;
        let key = key.trim();
        let mut value = value.trim();
        if (value.starts_with('"') && value.ends_with('"'))
            || (value.starts_with('\'') && value.ends_with('\''))
        {
            if value.len() >= 2 {
                value = &value[1..value.len() - 1];
            }
        }

        match current_section {
            "project" => match key {
                "package" => {
                    package = if value.is_empty() {
                        None
                    } else {
                        Some(value.to_string())
                    };
                }
                "main" => {
                    if !value.is_empty() {
                        main = value.to_string();
                    }
                }
                _ => {}
            },
            "dependencies" => {
                let dep = Dependency::parse_coord(key, value)?;
                dependencies.push(dep);
            }
            _ => {}
        }
    }

    Ok((package, main, dependencies))
}

fn parse_legacy(path: &Path) -> Result<(Option<String>, String), String> {
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
