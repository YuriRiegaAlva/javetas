use crate::commands::{error, ok, prompt};
use crate::style;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command as Process;

const REPO: &str = "YuriRiegaAlva/javetas";
const VERSION: &str = env!("CARGO_PKG_VERSION");

fn parse_version(s: &str) -> Option<(u64, u64, u64)> {
    let s = s.strip_prefix('v').unwrap_or(s);
    let mut parts = s
        .split(|c: char| !c.is_ascii_digit())
        .filter(|p| !p.is_empty());
    let major = parts.next()?.parse().ok()?;
    let minor = parts.next()?.parse().ok()?;
    let patch = parts.next()?.parse().ok()?;
    Some((major, minor, patch))
}

fn extract_tag(json: &str) -> Option<&str> {
    let rest = json.split("\"tag_name\"").nth(1)?;
    let rest = rest.split_once(':')?.1.trim_start();
    rest.strip_prefix('"')?.split('"').next()
}

fn platform_asset() -> Result<String, String> {
    let asset = match (env::consts::OS, env::consts::ARCH) {
        ("linux", "x86_64") => "javetas-linux-x86_64.tar.gz",
        ("macos", "x86_64") => "javetas-macos-x86_64.tar.gz",
        ("macos", "aarch64") => "javetas-macos-arm64.tar.gz",
        (os, arch) => return Err(format!("no release available for {os}/{arch}")),
    };
    Ok(asset.to_string())
}

fn urls(asset: &str) -> (String, String) {
    match env::var("JAVETAS_UPDATE_BASE_URL") {
        Ok(base) => {
            let base = base.trim_end_matches('/');
            (
                format!("{base}/releases/latest"),
                format!("{base}/releases/latest/download/{asset}"),
            )
        }
        Err(_) => (
            format!("https://api.github.com/repos/{REPO}/releases/latest"),
            format!("https://github.com/{REPO}/releases/latest/download/{asset}"),
        ),
    }
}

fn fetch_text(url: &str) -> Result<String, String> {
    let curl = Process::new("curl").args(["-sS", url]).output();
    match curl {
        Ok(out) if out.status.success() => Ok(String::from_utf8_lossy(&out.stdout).into_owned()),
        Ok(out) => Err(format!(
            "version check failed (curl exit code {})",
            out.status.code().unwrap_or(1)
        )),
        Err(_) => {
            let wget = Process::new("wget").args(["-qO-", url]).output();
            match wget {
                Ok(out) if out.status.success() => {
                    Ok(String::from_utf8_lossy(&out.stdout).into_owned())
                }
                Ok(out) => Err(format!(
                    "version check failed (wget exit code {})",
                    out.status.code().unwrap_or(1)
                )),
                Err(_) => Err("neither curl nor wget is available; install one of them".into()),
            }
        }
    }
}

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

fn temp_dir() -> Result<PathBuf, String> {
    let dir = env::temp_dir().join(format!("javetas-update-{}", std::process::id()));
    fs::create_dir_all(&dir).map_err(|e| format!("cannot create temp dir: {e}"))?;
    Ok(dir)
}

#[cfg(unix)]
fn make_executable(path: &Path) -> Result<(), String> {
    use std::os::unix::fs::PermissionsExt;
    fs::set_permissions(path, fs::Permissions::from_mode(0o755))
        .map_err(|e| format!("cannot chmod {}: {e}", path.display()))
}

#[cfg(not(unix))]
fn make_executable(_path: &Path) -> Result<(), String> {
    Ok(())
}

fn perform_update(dir: &Path, asset_url: &str) -> Result<PathBuf, String> {
    let archive = dir.join("javetas.tar.gz");
    download(asset_url, &archive)?;

    let status = Process::new("tar")
        .arg("-xzf")
        .arg(&archive)
        .arg("-C")
        .arg(dir)
        .status();
    match status {
        Ok(s) if s.success() => {}
        Ok(s) => {
            return Err(format!(
                "cannot extract archive (tar exit code {})",
                s.code().unwrap_or(1)
            ));
        }
        Err(e) => return Err(format!("cannot run tar: {e}")),
    }

    let new_bin = dir.join("javetas");
    if !new_bin.is_file() {
        return Err("the archive does not contain the javetas binary".into());
    }
    make_executable(&new_bin)?;

    let exe = env::current_exe().map_err(|e| format!("cannot locate the installed binary: {e}"))?;
    fs::rename(&new_bin, &exe).map_err(|e| {
        format!(
            "cannot replace {} (do you have write permission?): {e}",
            exe.display()
        )
    })?;
    Ok(exe)
}

pub fn update_cmd(yes: bool) -> i32 {
    if cfg!(windows) {
        eprintln!(
            "{} self-update is not supported on Windows",
            style::red("error:")
        );
        eprintln!("{} re-run the install script instead:", style::dim("hint:"));
        println!("  irm https://raw.githubusercontent.com/{REPO}/main/install.ps1 | iex");
        return 1;
    }

    let asset = match platform_asset() {
        Ok(a) => a,
        Err(e) => return error(&format!("cannot self-update: {e}")),
    };
    let (version_url, asset_url) = urls(&asset);

    let current = parse_version(VERSION).unwrap_or((0, 0, 0));
    let json = match fetch_text(&version_url) {
        Ok(t) => t,
        Err(e) => return error(&e),
    };
    let tag = match extract_tag(&json) {
        Some(t) => t.to_string(),
        None => return error("cannot find the latest version in the release info"),
    };
    let latest = match parse_version(&tag) {
        Some(v) => v,
        None => return error(&format!("cannot parse latest version tag `{tag}`")),
    };

    if latest <= current {
        ok(&format!("already up to date (v{VERSION})"));
        return 0;
    }

    println!("current: v{VERSION}, latest: {tag}");
    if !yes {
        match prompt("Update? [y/N]") {
            Some(a) if a.eq_ignore_ascii_case("y") || a.eq_ignore_ascii_case("yes") => {}
            _ => {
                println!("{}", style::dim("cancelled"));
                return 0;
            }
        }
    }

    let dir = match temp_dir() {
        Ok(d) => d,
        Err(e) => return error(&e),
    };
    let result = perform_update(&dir, &asset_url);
    let _ = fs::remove_dir_all(&dir);
    match result {
        Ok(exe) => {
            ok(&format!("updated to {tag}"));
            println!("{}", style::dim(&format!("  -> {}", exe.display())));
            0
        }
        Err(e) => error(&e),
    }
}
