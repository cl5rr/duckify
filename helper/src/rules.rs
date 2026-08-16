use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};

use crate::audio::SessionPeak;
use crate::games::{Classification, GameIndex};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub audible_threshold: f32,
    pub game_silent_volume: f32,
    pub voice_duck_volume: f32,
    pub game_attack_ms: u64,
    pub game_release_ms: u64,
    pub voice_attack_ms: u64,
    pub voice_release_ms: u64,
    pub fade_ms: u64,
    pub voice_processes: Vec<String>,
    #[serde(default = "default_browsers")]
    pub browser_processes: Vec<String>,
    #[serde(default = "default_true")]
    pub browsers_as_games: bool,
    pub user_games: Vec<String>,
    pub user_not_games: Vec<String>,
    pub enabled: bool,
}

fn default_true() -> bool {
    true
}

fn default_browsers() -> Vec<String> {
    vec![
        "chrome.exe".into(),
        "msedge.exe".into(),
        "firefox.exe".into(),
        "brave.exe".into(),
        "opera.exe".into(),
        "zen.exe".into(),
        "vivaldi.exe".into(),
        "librewolf.exe".into(),
    ]
}

impl Default for Config {
    fn default() -> Self {
        Self {
            audible_threshold: 0.01,
            game_silent_volume: 0.10,
            voice_duck_volume: 0.10,
            game_attack_ms: 300,
            game_release_ms: 2500,
            voice_attack_ms: 50,
            voice_release_ms: 800,
            fade_ms: 200,
            voice_processes: vec![
                "discord.exe".into(),
                "discordptb.exe".into(),
                "discordcanary.exe".into(),
                "vesktop.exe".into(),
                "teams.exe".into(),
                "ms-teams.exe".into(),
            ],
            browser_processes: default_browsers(),
            browsers_as_games: true,
            user_games: Vec::new(),
            user_not_games: Vec::new(),
            enabled: true,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Decision {
    pub pause: bool,
    pub volume: f32,
    pub reason: &'static str,
    pub fade_ms: u64,
    pub detail: String,
}

struct Hysteresis {
    state: bool,
    since: Instant,
    attack: Duration,
    release: Duration,
}

impl Hysteresis {
    fn new(attack_ms: u64, release_ms: u64) -> Self {
        Self {
            state: false,
            since: Instant::now(),
            attack: Duration::from_millis(attack_ms),
            release: Duration::from_millis(release_ms),
        }
    }

    fn update(&mut self, raw: bool, now: Instant) -> bool {
        if raw == self.state {
            self.since = now;
            return self.state;
        }
        let needed = if raw { self.attack } else { self.release };
        if now.duration_since(self.since) >= needed {
            self.state = raw;
            self.since = now;
        }
        self.state
    }

    fn retime(&mut self, attack_ms: u64, release_ms: u64) {
        self.attack = Duration::from_millis(attack_ms);
        self.release = Duration::from_millis(release_ms);
    }
}

pub struct Engine {
    pub config: Config,
    index: GameIndex,
    game_audible: Hysteresis,
    voice_active: Hysteresis,
    game_present: Hysteresis,
}

impl Engine {
    pub fn new(config: Config) -> Self {
        let index = GameIndex::build(&config.user_games, &config.user_not_games);
        let game_audible = Hysteresis::new(config.game_attack_ms, config.game_release_ms);
        let voice_active = Hysteresis::new(config.voice_attack_ms, config.voice_release_ms);
        let game_present = Hysteresis::new(0, 1500);
        Self {
            config,
            index,
            game_audible,
            voice_active,
            game_present,
        }
    }

    pub fn apply_config(&mut self, config: Config) {
        self.index = GameIndex::build(&config.user_games, &config.user_not_games);
        self.game_audible
            .retime(config.game_attack_ms, config.game_release_ms);
        self.voice_active
            .retime(config.voice_attack_ms, config.voice_release_ms);
        self.config = config;
    }

    pub fn known_game_count(&self) -> usize {
        self.index.known_count()
    }

    pub fn wants_foreground(&self) -> bool {
        self.config.enabled
    }

    pub fn evaluate(
        &mut self,
        sessions: &[SessionPeak],
        foreground: Option<&str>,
        now: Instant,
    ) -> (Decision, Vec<(String, f32)>) {
        let mut candidates = Vec::new();

        if !self.config.enabled {
            return (
                Decision {
                    pause: false,
                    volume: 1.0,
                    reason: "disabled",
                    fade_ms: self.config.fade_ms,
                    detail: "Duckify is turned off".into(),
                },
                candidates,
            );
        }

        let threshold = self.config.audible_threshold;

        let mut game_running = false;
        let mut game_loud = false;
        let mut loud_game_name = String::new();
        let mut voice_loud = false;
        let mut voice_name = String::new();

        for s in sessions {
            if self.is_voice(&s.process) {
                if s.peak > threshold {
                    voice_loud = true;
                    voice_name = s.process.clone();
                }
                continue;
            }

            if self.is_browser(&s.process) {
                if self.config.browsers_as_games && s.peak > threshold {
                    game_running = true;
                    game_loud = true;
                    loud_game_name = s.process.clone();
                }
                continue;
            }

            match self.index.classify(&s.process) {
                Classification::Game => {
                    game_running = true;
                    if s.peak > threshold {
                        game_loud = true;
                        loud_game_name = s.process.clone();
                    }
                }
                Classification::Unknown => {
                    candidates.push((s.process.clone(), s.peak));
                }
                Classification::NotGame => {}
            }
        }

        if let Some(fg) = foreground {
            match self.index.classify(fg) {
                Classification::Game => game_running = true,
                Classification::Unknown => {
                    if !candidates.iter().any(|(p, _)| p == fg) {
                        candidates.push((fg.to_string(), 0.0));
                    }
                }
                Classification::NotGame => {}
            }
        }

        let present = self.game_present.update(game_running, now);
        let audible = self.game_audible.update(game_loud, now);
        let talking = self.voice_active.update(voice_loud, now);

        let decision = if present && audible {
            Decision {
                pause: true,
                volume: 0.0,
                reason: "game-audible",
                fade_ms: self.config.fade_ms,
                detail: if loud_game_name.is_empty() {
                    "Game is making sound".into()
                } else {
                    format!("{loud_game_name} is playing audio")
                },
            }
        } else if present {
            Decision {
                pause: false,
                volume: self.config.game_silent_volume,
                reason: "game-silent",
                fade_ms: self.config.fade_ms,
                detail: "Game running but quiet".into(),
            }
        } else if talking {
            Decision {
                pause: false,
                volume: self.config.voice_duck_volume,
                reason: "voice-active",
                fade_ms: self.config.fade_ms,
                detail: if voice_name.is_empty() {
                    "Someone is talking".into()
                } else {
                    format!("Voice activity in {voice_name}")
                },
            }
        } else {
            Decision {
                pause: false,
                volume: 1.0,
                reason: "idle",
                fade_ms: self.config.fade_ms,
                detail: "Nothing competing for audio".into(),
            }
        };

        (decision, candidates)
    }

    fn is_voice(&self, process: &str) -> bool {
        self.config
            .voice_processes
            .iter()
            .any(|v| v.eq_ignore_ascii_case(process))
    }

    fn is_browser(&self, process: &str) -> bool {
        self.config
            .browser_processes
            .iter()
            .any(|b| b.eq_ignore_ascii_case(process))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn peak(process: &str, peak: f32) -> SessionPeak {
        SessionPeak {
            process: process.into(),
            pid: 1,
            peak,
        }
    }

    fn engine() -> Engine {
        let mut cfg = Config::default();
        cfg.user_games = vec!["mygame.exe".into()];
        Engine::new(cfg)
    }

    #[test]
    fn idle_when_nothing_is_playing() {
        let mut e = engine();
        let (d, _) = e.evaluate(&[], None, Instant::now());
        assert_eq!(d.reason, "idle");
        assert_eq!(d.volume, 1.0);
        assert!(!d.pause);
    }

    #[test]
    fn silent_game_plays_quietly_rather_than_pausing() {
        let mut e = engine();
        let (d, _) = e.evaluate(&[peak("mygame.exe", 0.0)], None, Instant::now());
        assert_eq!(d.reason, "game-silent");
        assert!(!d.pause);
        assert_eq!(d.volume, e.config.game_silent_volume);
    }

    #[test]
    fn audible_game_pauses_only_after_attack_time() {
        let mut e = engine();
        let t0 = Instant::now();
        let (d, _) = e.evaluate(&[peak("mygame.exe", 0.5)], None, t0);
        assert!(!d.pause, "paused too eagerly: {:?}", d);

        let t1 = t0 + Duration::from_millis(400);
        let (d, _) = e.evaluate(&[peak("mygame.exe", 0.5)], None, t1);
        assert!(d.pause);
        assert_eq!(d.reason, "game-audible");
    }

    #[test]
    fn brief_silence_does_not_immediately_resume() {
        let mut e = engine();
        let t0 = Instant::now();
        e.evaluate(&[peak("mygame.exe", 0.5)], None, t0);
        let t1 = t0 + Duration::from_millis(400);
        assert!(e.evaluate(&[peak("mygame.exe", 0.5)], None, t1).0.pause);

        let t2 = t1 + Duration::from_millis(500);
        let (d, _) = e.evaluate(&[peak("mygame.exe", 0.0)], None, t2);
        assert!(d.pause, "resumed during a brief in-game silence");

        let t3 = t1 + Duration::from_millis(3000);
        let (d, _) = e.evaluate(&[peak("mygame.exe", 0.0)], None, t3);
        assert!(!d.pause);
        assert_eq!(d.reason, "game-silent");
    }

    #[test]
    fn voice_ducks_but_does_not_pause() {
        let mut e = engine();
        let t0 = Instant::now();
        e.evaluate(&[peak("discord.exe", 0.4)], None, t0);
        let t1 = t0 + Duration::from_millis(100);
        let (d, _) = e.evaluate(&[peak("discord.exe", 0.4)], None, t1);
        assert_eq!(d.reason, "voice-active");
        assert!(!d.pause);
        assert_eq!(d.volume, e.config.voice_duck_volume);
    }

    #[test]
    fn game_outranks_voice() {
        let mut e = engine();
        let t0 = Instant::now();
        let sessions = [peak("mygame.exe", 0.5), peak("discord.exe", 0.5)];
        e.evaluate(&sessions, None, t0);
        let (d, _) = e.evaluate(&sessions, None, t0 + Duration::from_millis(400));
        assert_eq!(d.reason, "game-audible");
    }

    #[test]
    fn unknown_process_is_never_acted_on_only_logged() {
        let mut e = engine();
        let (d, candidates) =
            e.evaluate(&[peak("mystery.exe", 0.6)], None, Instant::now());
        assert_eq!(d.reason, "idle", "acted on an unclassified process");
        assert_eq!(d.volume, 1.0);
        assert!(candidates.iter().any(|(p, _)| p == "mystery.exe"));
    }

    #[test]
    fn unknown_reports_live_peak_as_it_changes() {
        let mut e = engine();
        let (_, c1) = e.evaluate(&[peak("mystery.exe", 0.75)], None, Instant::now());
        assert_eq!(c1.iter().find(|(p, _)| p == "mystery.exe").unwrap().1, 0.75);

        let (_, c2) = e.evaluate(&[peak("mystery.exe", 0.10)], None, Instant::now());
        assert_eq!(c2.iter().find(|(p, _)| p == "mystery.exe").unwrap().1, 0.10);
    }

    #[test]
    fn quiet_unknown_is_still_reported_with_its_level() {
        let mut e = engine();
        let (_, candidates) =
            e.evaluate(&[peak("mystery.exe", 0.0)], None, Instant::now());
        let found = candidates.iter().find(|(p, _)| p == "mystery.exe");
        assert!(found.is_some(), "silent unknown dropped from the list");
        assert_eq!(found.unwrap().1, 0.0);
    }

    #[test]
    fn silent_browser_does_not_trigger_anything() {
        let mut e = engine();
        let (d, candidates) =
            e.evaluate(&[peak("chrome.exe", 0.0)], None, Instant::now());
        assert_eq!(d.reason, "idle");
        assert!(candidates.is_empty(), "browser offered as a candidate");
    }

    #[test]
    fn audible_browser_pauses_like_a_game() {
        let mut e = engine();
        let t0 = Instant::now();
        e.evaluate(&[peak("brave.exe", 0.6)], None, t0);
        let (d, _) = e.evaluate(&[peak("brave.exe", 0.6)], None, t0 + Duration::from_millis(400));
        assert_eq!(d.reason, "game-audible");
        assert!(d.pause);
    }

    #[test]
    fn browser_audio_can_be_turned_off() {
        let mut e = engine();
        e.config.browsers_as_games = false;
        let t0 = Instant::now();
        e.evaluate(&[peak("brave.exe", 0.9)], None, t0);
        let (d, _) = e.evaluate(&[peak("brave.exe", 0.9)], None, t0 + Duration::from_millis(400));
        assert_eq!(d.reason, "idle");
        assert!(!d.pause);
    }

    #[test]
    fn browser_is_never_offered_as_an_unknown_candidate() {
        let mut e = engine();
        e.config.browsers_as_games = false;
        let (_, candidates) =
            e.evaluate(&[peak("firefox.exe", 0.9)], None, Instant::now());
        assert!(candidates.is_empty(), "browser leaked into candidates");
    }

    #[test]
    fn config_with_utf8_bom_still_parses() {
        let json = "\u{feff}{\"audible_threshold\":0.05,\"game_silent_volume\":0.2,\
            \"voice_duck_volume\":0.1,\"game_attack_ms\":300,\"game_release_ms\":2500,\
            \"voice_attack_ms\":50,\"voice_release_ms\":800,\"fade_ms\":200,\
            \"voice_processes\":[],\"user_games\":[\"x.exe\"],\"user_not_games\":[],\
            \"enabled\":true}";
        let stripped = json.strip_prefix('\u{feff}').unwrap_or(json);
        let cfg: Config = serde_json::from_str(stripped).expect("BOM-stripped config parses");
        assert_eq!(cfg.user_games, vec!["x.exe".to_string()]);
        assert_eq!(cfg.audible_threshold, 0.05);
    }

    #[test]
    fn disabling_returns_full_volume() {
        let mut e = engine();
        e.config.enabled = false;
        let (d, _) = e.evaluate(&[peak("mygame.exe", 0.9)], None, Instant::now());
        assert_eq!(d.volume, 1.0);
        assert!(!d.pause);
    }
}
