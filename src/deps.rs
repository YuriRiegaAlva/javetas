use crate::project::Project;
use crate::style;
use std::fs;
use std::path::Path;
use std::process::Command as Process;

fn download(url: &str, dest: &Path) -> Result<(), String> {
    let curl = Process::new("curl")
        .args(["-fsSL", url, "-o"])
        .arg(dest)
        .status();
    match curl {
        Ok(s) if s.success() => Ok(()),
        Ok(s) => Err(format!(
            "download failed (curl exit code {})",
            s.code().unwrap_or(1)
        )),
        Err(_) => {
            let wget = Process::new("wget")
                .args(["-q", url, "-O"])
                .arg(dest)
                .status();
            match wget {
                Ok(s) if s.success() => Ok(()),
                Ok(s) => Err(format!(
                    "download failed (wget exit code {})",
                    s.code().unwrap_or(1)
                )),
                Err(_) => Err("neither curl nor wget is available; install one of them".into()),
            }
        }
    }
}

pub fn ensure_dependencies(project: &Project) -> i32 {
    if project.dependencies.is_empty() {
        return 0;
    }

    let lib_dir = project.lib_dir();
    if !lib_dir.is_dir() {
        if let Err(e) = fs::create_dir_all(&lib_dir) {
            eprintln!("{} cannot create lib/ directory: {e}", style::red("error:"));
            return 1;
        }
    }

    for dep in &project.dependencies {
        let dest = lib_dir.join(dep.jar_name());
        if !dest.is_file() {
            println!(
                "{} downloading {} ({} v{}) ...",
                style::green("dep:"),
                dep.alias,
                dep.artifact_id,
                dep.version
            );
            if let Err(e) = download(&dep.maven_url(), &dest) {
                eprintln!(
                    "{} failed to download `{}` from Maven Central: {e}",
                    style::red("error:"),
                    dep.jar_name()
                );
                let _ = fs::remove_file(&dest);
                return 1;
            }
        }
    }

    0
}
