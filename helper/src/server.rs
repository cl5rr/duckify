use std::fs;
use std::net::{TcpListener, TcpStream};
use std::path::PathBuf;
use std::sync::mpsc::{self, Sender};
use std::sync::{Arc, Mutex};

use serde::{Deserialize, Serialize};
use tungstenite::{Message, WebSocket};

use crate::games::Candidate;
use crate::rules::{Config, Decision};
use crate::Shared;

pub const PORT: u16 = 8787;

#[derive(Serialize)]
struct StateMsg<'a> {
    #[serde(rename = "type")]
    kind: &'static str,
    pause: bool,
    volume: f32,
    reason: &'a str,
    detail: &'a str,
    fade_ms: u64,
    candidates: &'a [Candidate],
    known_games: usize,
    config: &'a Config,
    autostart: bool,
}

#[derive(Deserialize)]
#[serde(tag = "type")]
enum ClientMsg {
    #[serde(rename = "hello")]
    Hello,
    #[serde(rename = "config")]
    SetConfig { config: Config },
    #[serde(rename = "classify")]
    Classify { process: String, is_game: bool },
    #[serde(rename = "reset")]
    Reset,
    #[serde(rename = "autostart")]
    Autostart { enabled: bool },
}

#[derive(Clone)]
pub struct Hub {
    clients: Arc<Mutex<Vec<Sender<String>>>>,
    shared: Arc<Mutex<Shared>>,
}

impl Hub {
    pub fn has_clients(&self) -> bool {
        !self.clients.lock().unwrap().is_empty()
    }

    pub fn broadcast_decision(&self, decision: &Decision, candidates: &[Candidate]) {
        if self.clients.lock().unwrap().is_empty() {
            return;
        }

        let (known_games, config) = {
            let s = self.shared.lock().unwrap();
            (s.engine.known_game_count(), s.engine.config.clone())
        };

        let msg = StateMsg {
            kind: "state",
            pause: decision.pause,
            volume: decision.volume,
            reason: decision.reason,
            detail: &decision.detail,
            fade_ms: decision.fade_ms,
            candidates,
            known_games,
            config: &config,
            autostart: crate::install::autostart_enabled(),
        };
        let Ok(text) = serde_json::to_string(&msg) else {
            return;
        };

        let mut clients = self.clients.lock().unwrap();
        clients.retain(|tx| tx.send(text.clone()).is_ok());
    }
}

pub fn start(shared: Arc<Mutex<Shared>>) -> Option<Hub> {
    let hub = Hub {
        clients: Arc::new(Mutex::new(Vec::new())),
        shared,
    };

    let listener = match TcpListener::bind(("127.0.0.1", PORT)) {
        Ok(l) => l,
        Err(_) => return None,
    };

    let accept_hub = hub.clone();
    std::thread::spawn(move || {
        for stream in listener.incoming().flatten() {
            let hub = accept_hub.clone();
            std::thread::spawn(move || handle_client(stream, hub));
        }
    });

    Some(hub)
}

fn handle_client(stream: TcpStream, hub: Hub) {
    let Ok(write_half) = stream.try_clone() else {
        return;
    };
    let Ok(peer) = stream.try_clone() else {
        return;
    };
    let Ok(mut ws) = tungstenite::accept(stream) else {
        return;
    };

    let (tx, rx) = mpsc::channel::<String>();
    let reply_tx = tx.clone();
    hub.clients.lock().unwrap().push(tx);
    std::thread::spawn(move || {
        let mut out = WebSocket::from_raw_socket(
            write_half,
            tungstenite::protocol::Role::Server,
            None,
        );
        while let Ok(text) = rx.recv() {
            if out.send(Message::Text(text)).is_err() {
                break;
            }
        }
    });

    let _ = peer.set_read_timeout(None);

    reply(&hub, &reply_tx);

    loop {
        match ws.read() {
            Ok(Message::Text(text)) => {
                if let Ok(msg) = serde_json::from_str::<ClientMsg>(&text) {
                    handle_message(&hub, msg);
                    reply(&hub, &reply_tx);
                }
            }
            Ok(Message::Close(_)) => break,
            Ok(_) => {}
            Err(_) => break,
        }
    }
}

fn reply(hub: &Hub, tx: &Sender<String>) {
    let (decision, candidates, known_games, config) = {
        let s = hub.shared.lock().unwrap();
        (
            s.last.clone(),
            s.candidates.pending(crate::CANDIDATE_MIN_HITS),
            s.engine.known_game_count(),
            s.engine.config.clone(),
        )
    };
    let Some(decision) = decision else {
        return;
    };

    let msg = StateMsg {
        kind: "state",
        pause: decision.pause,
        volume: decision.volume,
        reason: decision.reason,
        detail: &decision.detail,
        fade_ms: decision.fade_ms,
        candidates: &candidates,
        known_games,
        config: &config,
        autostart: crate::install::autostart_enabled(),
    };
    if let Ok(text) = serde_json::to_string(&msg) {
        let _ = tx.send(text);
    }
}

fn handle_message(hub: &Hub, msg: ClientMsg) {
    match msg {
        ClientMsg::Hello => {}
        ClientMsg::SetConfig { config } => {
            save_config(&config);
            let mut s = hub.shared.lock().unwrap();
            s.engine.apply_config(config);
        }
        ClientMsg::Classify { process, is_game } => {
            let mut s = hub.shared.lock().unwrap();
            let mut config = s.engine.config.clone();
            let lower = process.to_lowercase();

            config.user_games.retain(|g| g.to_lowercase() != lower);
            config.user_not_games.retain(|g| g.to_lowercase() != lower);
            if is_game {
                config.user_games.push(lower.clone());
            } else {
                config.user_not_games.push(lower.clone());
            }

            save_config(&config);
            s.engine.apply_config(config);
            s.candidates.forget(&lower);
        }
        ClientMsg::Reset => {
            let mut s = hub.shared.lock().unwrap();
            let mut config = s.engine.config.clone();
            config.user_games.clear();
            config.user_not_games.clear();
            save_config(&config);
            s.engine.apply_config(config);
            s.candidates = crate::games::CandidateLog::default();
        }
        ClientMsg::Autostart { enabled } => {
            crate::install::set_autostart(enabled);
        }
    }
}

fn config_path() -> PathBuf {
    let base = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
    base.join("Duckify").join("config.json")
}

pub fn load_config() -> Config {
    let path = config_path();
    let Ok(text) = fs::read_to_string(&path) else {
        return Config::default();
    };
    let text = text.strip_prefix('\u{feff}').unwrap_or(&text);
    serde_json::from_str(text).unwrap_or_default()
}

pub fn save_config(config: &Config) {
    let path = config_path();
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    if let Ok(text) = serde_json::to_string_pretty(config) {
        let _ = fs::write(&path, text);
    }
}
