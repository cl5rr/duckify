use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

const SEED_GAMES: &[&str] = &[
    "robloxplayerbeta.exe",
    "ryujinx.exe",
    "pcsx2-qt.exe",
    "duckstation-qt-x64-releaseltcg.exe",
    "dolphin.exe",
    "cemu.exe",
    "rpcs3.exe",
    "ppsspp.exe",
];

const ALWAYS_ASK: &[&str] = &[
    "javaw.exe",
    "java.exe",
    "love.exe",
    "python.exe",
];

const DENYLIST: &[&str] = &[
    "chrome.exe",
    "msedge.exe",
    "firefox.exe",
    "brave.exe",
    "opera.exe",
    "zen.exe",
    "spotify.exe",
    "discord.exe",
    "obs64.exe",
    "obs32.exe",
    "vlc.exe",
    "mpc-hc64.exe",
    "mpv.exe",
    "explorer.exe",
    "steam.exe",
    "steamwebhelper.exe",
    "epicgameslauncher.exe",
    "code.exe",
    "devenv.exe",
    "windowsterminal.exe",
    "powershell.exe",
    "cmd.exe",
];

const STEAM_NON_GAMES: &[u32] = &[
    228980,
    250820,
    1070560,
    1391110,
    1628350,
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Classification {
    Game,
    NotGame,
    Unknown,
}

pub struct GameIndex {
    known: HashSet<String>,
    approved: HashSet<String>,
    rejected: HashSet<String>,
    always_ask: HashSet<String>,
    denied: HashSet<String>,
}

impl GameIndex {
    pub fn build(user_yes: &[String], user_no: &[String]) -> Self {
        let mut known: HashSet<String> =
            SEED_GAMES.iter().map(|s| s.to_string()).collect();

        for exe in scan_steam_libraries() {
            known.insert(exe);
        }
        for exe in user_yes {
            known.insert(exe.to_lowercase());
        }

        let always_ask: HashSet<String> =
            ALWAYS_ASK.iter().map(|s| s.to_string()).collect();

        let explicit: HashSet<String> =
            user_yes.iter().map(|s| s.to_lowercase()).collect();

        Self {
            known,
            approved: explicit.clone(),
            rejected: user_no.iter().map(|s| s.to_lowercase()).collect(),
            always_ask: always_ask.difference(&explicit).cloned().collect(),
            denied: DENYLIST.iter().map(|s| s.to_string()).collect(),
        }
    }

    pub fn classify(&self, exe: &str) -> Classification {
        let exe = exe.to_lowercase();

        if self.rejected.contains(&exe) {
            return Classification::NotGame;
        }
        if self.approved.contains(&exe) {
            return Classification::Game;
        }
        if self.denied.contains(&exe) {
            return Classification::NotGame;
        }
        if self.always_ask.contains(&exe) {
            return Classification::Unknown;
        }
        if self.known.contains(&exe) {
            return Classification::Game;
        }
        Classification::Unknown
    }

    pub fn known_count(&self) -> usize {
        self.known.len()
    }
}

fn scan_steam_libraries() -> Vec<String> {
    let mut exes = Vec::new();

    let Some(root) = steam_root() else {
        return exes;
    };

    let vdf = root.join("steamapps").join("libraryfolders.vdf");
    let libraries = match fs::read_to_string(&vdf) {
        Ok(text) => parse_library_paths(&text),
        Err(_) => vec![root.clone()],
    };

    for lib in libraries {
        let steamapps = lib.join("steamapps");
        let Ok(entries) = fs::read_dir(&steamapps) else {
            continue;
        };

        for entry in entries.flatten() {
            let path = entry.path();
            let is_manifest = path
                .file_name()
                .and_then(|n| n.to_str())
                .map(|n| n.starts_with("appmanifest_") && n.ends_with(".acf"))
                .unwrap_or(false);
            if !is_manifest {
                continue;
            }

            let Ok(text) = fs::read_to_string(&path) else {
                continue;
            };
            let Some(appid) = vdf_value(&text, "appid").and_then(|v| v.parse::<u32>().ok())
            else {
                continue;
            };
            if STEAM_NON_GAMES.contains(&appid) {
                continue;
            }
            let Some(installdir) = vdf_value(&text, "installdir") else {
                continue;
            };

            let game_dir = steamapps.join("common").join(&installdir);
            collect_executables(&game_dir, 0, &mut exes);
        }
    }

    exes
}

fn steam_root() -> Option<PathBuf> {
    for candidate in [
        r"C:\Program Files (x86)\Steam",
        r"C:\Program Files\Steam",
    ] {
        let p = PathBuf::from(candidate);
        if p.exists() {
            return Some(p);
        }
    }
    None
}

fn parse_library_paths(text: &str) -> Vec<PathBuf> {
    let mut out = Vec::new();
    for line in text.lines() {
        let line = line.trim();
        if !line.starts_with("\"path\"") {
            continue;
        }
        if let Some(value) = line.split('"').nth(3) {
            out.push(PathBuf::from(value.replace("\\\\", "\\")));
        }
    }
    out
}

fn vdf_value(text: &str, key: &str) -> Option<String> {
    let needle = format!("\"{key}\"");
    for line in text.lines() {
        let line = line.trim();
        if line.to_lowercase().starts_with(&needle.to_lowercase()) {
            if let Some(value) = line.split('"').nth(3) {
                return Some(value.replace("\\\\", "\\"));
            }
        }
    }
    None
}

fn collect_executables(dir: &Path, depth: usize, out: &mut Vec<String>) {
    const MAX_DEPTH: usize = 2;
    if depth > MAX_DEPTH {
        return;
    }
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_executables(&path, depth + 1, out);
        } else if path.extension().and_then(|e| e.to_str()) == Some("exe") {
            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                let lower = name.to_lowercase();
                if !is_noise_executable(&lower) {
                    out.push(lower);
                }
            }
        }
    }
}

fn is_noise_executable(name: &str) -> bool {
    const NOISE: &[&str] = &[
        "unitycrashhandler64.exe",
        "unitycrashhandler32.exe",
        "crashreportclient.exe",
        "crashpad_handler.exe",
        "vcredist_x64.exe",
        "vcredist_x86.exe",
        "dxsetup.exe",
        "uninstall.exe",
        "unins000.exe",
        "dotnetfx.exe",
        "oalinst.exe",
    ];
    NOISE.contains(&name)
        || name.starts_with("vcredist")
        || name.starts_with("directx")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_multiple_library_paths_from_vdf() {
        let vdf = r#"
"libraryfolders"
{
	"0"
	{
		"path"		"C:\\Program Files (x86)\\Steam"
	}
	"1"
	{
		"path"		"D:\\SteamLibrary"
	}
}
"#;
        let paths = parse_library_paths(vdf);
        assert_eq!(paths.len(), 2);
        assert_eq!(paths[0], PathBuf::from(r"C:\Program Files (x86)\Steam"));
        assert_eq!(paths[1], PathBuf::from(r"D:\SteamLibrary"));
    }

    #[test]
    fn extracts_appid_and_installdir_from_manifest() {
        let acf = r#"
"AppState"
{
	"appid"		"431960"
	"installdir"		"Wallpaper Engine"
}
"#;
        assert_eq!(vdf_value(acf, "appid").as_deref(), Some("431960"));
        assert_eq!(
            vdf_value(acf, "installdir").as_deref(),
            Some("Wallpaper Engine")
        );
    }

    #[test]
    fn user_rejection_overrides_seed_list() {
        let idx = GameIndex::build(&[], &["robloxplayerbeta.exe".into()]);
        assert_eq!(
            idx.classify("robloxplayerbeta.exe"),
            Classification::NotGame
        );
    }

    #[test]
    fn roblox_is_recognised_without_any_launcher_manifest() {
        let idx = GameIndex::build(&[], &[]);
        assert_eq!(idx.classify("RobloxPlayerBeta.exe"), Classification::Game);
    }

    #[test]
    fn ambiguous_names_always_ask_even_though_they_are_often_games() {
        let idx = GameIndex::build(&[], &[]);
        assert_eq!(idx.classify("javaw.exe"), Classification::Unknown);
    }

    #[test]
    fn explicit_yes_promotes_an_ambiguous_name() {
        let idx = GameIndex::build(&["javaw.exe".into()], &[]);
        assert_eq!(idx.classify("javaw.exe"), Classification::Game);
    }

    #[test]
    fn browsers_are_never_games() {
        let idx = GameIndex::build(&[], &[]);
        assert_eq!(idx.classify("chrome.exe"), Classification::NotGame);
        assert_eq!(idx.classify("spotify.exe"), Classification::NotGame);
    }

    #[test]
    fn explicit_yes_overrides_the_denylist() {
        let idx = GameIndex::build(&["obs64.exe".into()], &[]);
        assert_eq!(
            idx.classify("obs64.exe"),
            Classification::Game,
            "a denylisted app the user approved was still ignored"
        );
    }

    #[test]
    fn explicit_no_still_beats_an_explicit_yes() {
        let idx = GameIndex::build(&["x.exe".into()], &["x.exe".into()]);
        assert_eq!(idx.classify("x.exe"), Classification::NotGame);
    }

    #[test]
    fn bundled_redistributables_are_not_indexed_as_games() {
        assert!(is_noise_executable("unitycrashhandler64.exe"));
        assert!(is_noise_executable("vcredist_x64.exe"));
        assert!(!is_noise_executable("hollow_knight.exe"));
    }

    #[test]
    fn candidates_need_repeated_sightings_before_prompting() {
        let mut log = CandidateLog::default();
        log.note("mystery.exe", 0.5);
        assert!(log.pending(5).is_empty(), "prompted after a single sighting");
        for _ in 0..5 {
            log.note("mystery.exe", 0.5);
        }
        let pending = log.pending(5);
        assert_eq!(pending.len(), 1);
        assert_eq!(pending[0].process, "mystery.exe");
    }

    #[test]
    fn candidate_stays_listed_after_going_silent() {
        let mut log = CandidateLog::default();
        for _ in 0..6 {
            log.note("mystery.exe", 0.5);
        }
        log.mark_silent("mystery.exe");

        let pending = log.pending(5);
        assert_eq!(pending.len(), 1, "candidate vanished once it went quiet");
        assert!(!pending[0].active);
        assert_eq!(pending[0].peak, 0.0);
    }

    #[test]
    fn deciding_on_a_candidate_removes_it() {
        let mut log = CandidateLog::default();
        for _ in 0..6 {
            log.note("mystery.exe", 0.5);
        }
        log.forget("mystery.exe");
        assert!(log.pending(1).is_empty());
    }

    #[test]
    fn peak_tracks_the_latest_reading() {
        let mut log = CandidateLog::default();
        log.note("mystery.exe", 0.2);
        log.note("mystery.exe", 0.8);
        assert_eq!(log.pending(1)[0].peak, 0.8);
    }

    #[test]
    fn short_bursts_accumulate_towards_the_threshold() {
        // A notification chime is audible for only a few polls at a time. Three
        // separate bursts of two should still add up rather than resetting.
        let mut log = CandidateLog::default();
        for _ in 0..3 {
            log.note("chime.exe", 0.4);
            log.note("chime.exe", 0.4);
            log.mark_silent("chime.exe");
        }
        assert_eq!(
            log.pending(5).len(),
            1,
            "brief repeated sounds never reached the prompt threshold"
        );
    }

    #[test]
    fn going_silent_does_not_reset_progress() {
        let mut log = CandidateLog::default();
        for _ in 0..6 {
            log.note("x.exe", 0.4);
        }
        log.mark_silent("x.exe");
        log.mark_missing_silent(&[]);
        assert_eq!(log.pending(5).len(), 1, "silence cleared a listed candidate");
    }

    #[test]
    fn pending_is_stable_across_consecutive_polls() {
        // The panel blinked because the list was only built on some of the
        // frames that were sent. Whatever the frame, an established candidate
        // must always be present.
        let mut log = CandidateLog::default();
        for _ in 0..6 {
            log.note("x.exe", 0.4);
        }
        for frame in 0..40 {
            if frame % 3 == 0 {
                log.note("x.exe", 0.4);
            } else {
                log.mark_missing_silent(&[]);
            }
            assert_eq!(
                log.pending(5).len(),
                1,
                "candidate vanished on frame {frame}"
            );
        }
    }

    #[test]
    fn missing_from_the_poll_marks_inactive_but_keeps_listing() {
        let mut log = CandidateLog::default();
        for _ in 0..6 {
            log.note("x.exe", 0.4);
        }
        // Process exited entirely: absent from the session list.
        log.mark_missing_silent(&[]);
        let pending = log.pending(5);
        assert_eq!(pending.len(), 1);
        assert!(!pending[0].active);
        assert_eq!(pending[0].peak, 0.0);
    }
}

#[derive(Default)]
#[derive(Debug, Clone, serde::Serialize)]
pub struct Candidate {
    pub process: String,
    pub peak: f32,
    pub active: bool,
}

#[derive(Default)]
pub struct CandidateLog {
    seen: HashMap<String, u32>,
    peak: HashMap<String, f32>,
    active: HashMap<String, bool>,
}

impl CandidateLog {
    pub fn note(&mut self, exe: &str, peak: f32) {
        let key = exe.to_lowercase();
        let hits = self.seen.entry(key.clone()).or_insert(0);
        // Saturate rather than wrap: a long-running app would otherwise count
        // past u32 over a few days of uptime.
        *hits = hits.saturating_add(1);
        self.peak.insert(key.clone(), peak);
        self.active.insert(key, true);
    }

    pub fn mark_silent(&mut self, exe: &str) {
        let key = exe.to_lowercase();
        if let Some(a) = self.active.get_mut(&key) {
            *a = false;
        }
        self.peak.insert(key, 0.0);
    }

    pub fn mark_missing_silent(&mut self, seen: &[(String, f32)]) {
        let peak = &mut self.peak;
        for (name, active) in self.active.iter_mut() {
            if *active && !seen.iter().any(|(p, _)| p == name) {
                *active = false;
                if let Some(v) = peak.get_mut(name) {
                    *v = 0.0;
                }
            }
        }
    }

    pub fn pending(&self, min_hits: u32) -> Vec<Candidate> {
        let mut v: Vec<Candidate> = self
            .seen
            .iter()
            .filter(|(_, &n)| n >= min_hits)
            .map(|(k, _)| Candidate {
                process: k.clone(),
                peak: *self.peak.get(k).unwrap_or(&0.0),
                active: *self.active.get(k).unwrap_or(&false),
            })
            .collect();
        v.sort_by(|a, b| a.process.cmp(&b.process));
        v
    }

    pub fn forget(&mut self, exe: &str) {
        let key = exe.to_lowercase();
        self.seen.remove(&key);
        self.peak.remove(&key);
        self.active.remove(&key);
    }
}
