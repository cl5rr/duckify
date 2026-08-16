use std::fs;
use std::os::windows::process::CommandExt;
use std::path::PathBuf;
use std::process::Command;

pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const REPO: &str = "cl5rr/duckify";

const NO_WINDOW: u32 = 0x0800_0000;

pub const HELPER_EXE: &[u8] = include_bytes!(env!("DUCKIFY_HELPER"));
pub const EXTENSION_JS: &str = include_str!(env!("DUCKIFY_EXTENSION"));

pub fn install_dir() -> PathBuf {
    dirs_local().join("Duckify")
}

fn dirs_local() -> PathBuf {
    std::env::var("LOCALAPPDATA")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("."))
}

fn roaming() -> PathBuf {
    std::env::var("APPDATA")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("."))
}

pub fn is_installed() -> bool {
    install_dir().join("duckify-helper.exe").exists()
}

pub fn installed_version() -> Option<String> {
    fs::read_to_string(install_dir().join("version.txt"))
        .ok()
        .map(|s| s.trim().to_string())
}

fn run(cmd: &str, args: &[&str]) -> bool {
    Command::new(cmd)
        .args(args)
        .creation_flags(NO_WINDOW)
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

pub fn spicetify() -> Option<PathBuf> {
    let p = dirs_local().join("spicetify").join("spicetify.exe");
    if p.exists() {
        return Some(p);
    }
    // Some installs put it on PATH instead.
    let found = Command::new("where")
        .arg("spicetify")
        .creation_flags(NO_WINDOW)
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).lines().next().unwrap_or("").trim().to_string())
        .filter(|s| !s.is_empty())
        .map(PathBuf::from);
    found.filter(|p| p.exists())
}

pub fn spotify_installed() -> bool {
    roaming().join("Spotify").join("Spotify.exe").exists()
}

pub fn open_url(url: &str) {
    let _ = Command::new("cmd")
        .args(["/C", "start", "", url])
        .creation_flags(NO_WINDOW)
        .spawn();
}

pub fn stop_helper() {
    let _ = run("taskkill", &["/IM", "duckify-helper.exe", "/F"]);
    std::thread::sleep(std::time::Duration::from_millis(600));
}

/// Copy the helper, register autostart, and install the Spicetify extension.
pub fn install(progress: impl Fn(&str, i32)) -> Result<(), String> {
    progress("Stopping any running copy…", 5);
    stop_helper();

    let dir = install_dir();
    progress("Creating program folder…", 15);
    fs::create_dir_all(&dir).map_err(|e| format!("could not create {dir:?}: {e}"))?;

    progress("Writing files…", 30);
    let exe = dir.join("duckify-helper.exe");
    fs::write(&exe, HELPER_EXE).map_err(|e| format!("could not write helper: {e}"))?;
    let _ = fs::write(dir.join("version.txt"), VERSION);

    progress("Registering startup entry…", 50);
    if !run(&exe.to_string_lossy(), &["--install"]) {
        return Err("could not register the startup entry".into());
    }

    progress("Installing the Spotify extension…", 65);
    let ext_dir = roaming().join("spicetify").join("Extensions");
    if ext_dir.exists() || fs::create_dir_all(&ext_dir).is_ok() {
        fs::write(ext_dir.join("duckify.js"), EXTENSION_JS)
            .map_err(|e| format!("could not write the extension: {e}"))?;

        if let Some(spice) = spicetify() {
            let s = spice.to_string_lossy().to_string();
            progress("Enabling the extension…", 78);
            // Preserve any other extensions the user already has enabled.
            let listed = Command::new(&s)
                .args(["config", "extensions"])
                .creation_flags(NO_WINDOW)
                .output()
                .ok()
                .map(|o| String::from_utf8_lossy(&o.stdout).to_string())
                .unwrap_or_default();

            if !listed.contains("duckify.js") {
                run(&s, &["config", "extensions", "duckify.js"]);
            }

            progress("Applying to Spotify…", 88);
            run(&s, &["apply"]);
        }
    }

    progress("Starting Duckify…", 95);
    let _ = Command::new(&exe).creation_flags(NO_WINDOW).spawn();

    progress("Done.", 100);
    Ok(())
}

/// Remove everything this installer created.
pub fn uninstall(progress: impl Fn(&str, i32)) -> Result<(), String> {
    progress("Stopping Duckify…", 10);
    let dir = install_dir();
    let exe = dir.join("duckify-helper.exe");
    if exe.exists() {
        run(&exe.to_string_lossy(), &["--uninstall"]);
    }
    stop_helper();

    progress("Removing the Spotify extension…", 35);
    let ext = roaming().join("spicetify").join("Extensions").join("duckify.js");
    let _ = fs::remove_file(&ext);

    if let Some(spice) = spicetify() {
        let s = spice.to_string_lossy().to_string();
        // Clearing the whole list would disable the user's other extensions, so
        // only strip ours out of it.
        let listed = Command::new(&s)
            .args(["config", "extensions"])
            .creation_flags(NO_WINDOW)
            .output()
            .ok()
            .map(|o| String::from_utf8_lossy(&o.stdout).to_string())
            .unwrap_or_default();

        if listed.contains("duckify.js") {
            run(&s, &["config", "extensions", "duckify.js-"]);
        }
        progress("Applying to Spotify…", 60);
        run(&s, &["apply"]);
    }

    progress("Deleting files…", 80);
    let _ = fs::remove_dir_all(&dir);

    progress("Removed.", 100);
    Ok(())
}

#[derive(Debug, Clone)]
pub struct Release {
    pub tag: String,
    pub url: Option<String>,
}

/// Ask GitHub for the newest release. Returns None when offline or rate
/// limited, which must never block installing.
pub fn latest_release() -> Option<Release> {
    let url = format!("https://api.github.com/repos/{REPO}/releases/latest");
    let body = ureq::get(&url)
        .set("User-Agent", "duckify-setup")
        .timeout(std::time::Duration::from_secs(6))
        .call()
        .ok()?
        .into_string()
        .ok()?;

    let v: serde_json::Value = serde_json::from_str(&body).ok()?;
    let tag = v.get("tag_name")?.as_str()?.to_string();
    let asset = v
        .get("assets")?
        .as_array()?
        .iter()
        .find(|a| {
            a.get("name")
                .and_then(|n| n.as_str())
                .map(|n| n.ends_with(".exe"))
                .unwrap_or(false)
        })
        .and_then(|a| a.get("browser_download_url"))
        .and_then(|u| u.as_str())
        .map(|s| s.to_string());

    Some(Release { tag, url: asset })
}

/// Compare dotted numeric versions; leading v is ignored.
pub fn is_newer(remote: &str, local: &str) -> bool {
    let parse = |s: &str| -> Vec<u32> {
        s.trim_start_matches(['v', 'V'])
            .split('.')
            .map(|p| p.chars().take_while(|c| c.is_ascii_digit()).collect::<String>())
            .map(|p| p.parse().unwrap_or(0))
            .collect()
    };
    let r = parse(remote);
    let l = parse(local);
    for i in 0..r.len().max(l.len()) {
        let a = r.get(i).copied().unwrap_or(0);
        let b = l.get(i).copied().unwrap_or(0);
        if a != b {
            return a > b;
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::is_newer;

    #[test]
    fn compares_versions_numerically() {
        assert!(is_newer("v1.1.0", "1.0.0"));
        assert!(is_newer("1.0.1", "1.0.0"));
        assert!(is_newer("v2.0.0", "v1.9.9"));
        assert!(!is_newer("1.0.0", "1.0.0"));
        assert!(!is_newer("v1.0.0", "1.0.1"));
    }

    #[test]
    fn treats_ten_as_greater_than_nine() {
        assert!(is_newer("1.10.0", "1.9.0"), "compared as text, not numbers");
    }

    #[test]
    fn tolerates_missing_and_extra_parts() {
        assert!(is_newer("1.1", "1.0.9"));
        assert!(!is_newer("1.0", "1.0.0"));
    }
}
