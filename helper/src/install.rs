use std::os::windows::process::CommandExt;
use std::path::PathBuf;
use std::process::Command;
use std::sync::atomic::{AtomicBool, Ordering};

const NO_WINDOW: u32 = 0x0800_0000;

const RUN_KEY: &str = r"HKCU\Software\Microsoft\Windows\CurrentVersion\Run";
const PROTO_KEY: &str = r"HKCU\Software\Classes\duckify";

fn exe_path() -> Option<PathBuf> {
    std::env::current_exe().ok()
}

fn reg(args: &[&str]) -> bool {
    Command::new("reg")
        .args(args)
        .creation_flags(NO_WINDOW)
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

pub fn install() {
    let Some(exe) = exe_path() else {
        return;
    };
    let path = exe.to_string_lossy().to_string();

    reg(&[
        "add",
        RUN_KEY,
        "/v",
        "Duckify",
        "/t",
        "REG_SZ",
        "/d",
        &format!("\"{path}\""),
        "/f",
    ]);

    register_protocol(&path);
}

#[allow(dead_code)]
pub fn uninstall() {
    reg(&["delete", RUN_KEY, "/v", "Duckify", "/f"]);
    reg(&["delete", PROTO_KEY, "/f"]);
}

fn register_protocol(path: &str) {
    reg(&[
        "add",
        PROTO_KEY,
        "/ve",
        "/t",
        "REG_SZ",
        "/d",
        "URL:Duckify Protocol",
        "/f",
    ]);
    reg(&[
        "add",
        PROTO_KEY,
        "/v",
        "URL Protocol",
        "/t",
        "REG_SZ",
        "/d",
        "",
        "/f",
    ]);
    reg(&[
        "add",
        &format!(r"{PROTO_KEY}\shell\open\command"),
        "/ve",
        "/t",
        "REG_SZ",
        "/d",
        &format!("\"{path}\""),
        "/f",
    ]);
}

pub fn ensure_registered() {
    let Some(exe) = exe_path() else {
        return;
    };
    let path = exe.to_string_lossy().to_string();

    let current = Command::new("reg")
        .args(["query", &format!(r"{PROTO_KEY}\shell\open\command"), "/ve"])
        .creation_flags(NO_WINDOW)
        .output()
        .ok()
        .map(|o| String::from_utf8_lossy(&o.stdout).to_string())
        .unwrap_or_default();

    if !current.contains(path.as_str()) {
        register_protocol(&path);
    }
}

// Broadcast happens several times a second; spawning reg.exe that often would
// cost far more than the state is worth, so the answer is cached and only
// refreshed when we change it ourselves.
static AUTOSTART: AtomicBool = AtomicBool::new(false);
static AUTOSTART_READ: AtomicBool = AtomicBool::new(false);

fn query_autostart() -> bool {
    Command::new("reg")
        .args(["query", RUN_KEY, "/v", "Duckify"])
        .creation_flags(NO_WINDOW)
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

pub fn autostart_enabled() -> bool {
    if !AUTOSTART_READ.swap(true, Ordering::Relaxed) {
        AUTOSTART.store(query_autostart(), Ordering::Relaxed);
    }
    AUTOSTART.load(Ordering::Relaxed)
}

pub fn set_autostart(on: bool) {
    if on {
        install();
    } else {
        reg(&["delete", RUN_KEY, "/v", "Duckify", "/f"]);
    }
    AUTOSTART.store(on, Ordering::Relaxed);
    AUTOSTART_READ.store(true, Ordering::Relaxed);
}
