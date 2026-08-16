#![windows_subsystem = "windows"]

mod audio;
mod games;
mod install;
mod rules;
mod server;

use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use windows::Win32::Foundation::HWND;
use windows::Win32::System::Com::{CoInitializeEx, COINIT_MULTITHREADED};
use windows::Win32::System::Threading::GetCurrentProcessId;
use windows::Win32::UI::WindowsAndMessaging::{
    GetForegroundWindow, GetWindowThreadProcessId,
};

use crate::audio::AudioMonitor;
use crate::games::CandidateLog;
use crate::rules::{Decision, Engine};

const POLL_INTERVAL: Duration = Duration::from_millis(50);

pub const CANDIDATE_MIN_HITS: u32 = 5;

pub struct Shared {
    pub engine: Engine,
    pub candidates: CandidateLog,
    pub last: Option<Decision>,
}

fn main() {
    if std::env::args().any(|a| a == "--install") {
        install::install();
        return;
    }
    if std::env::args().any(|a| a == "--uninstall") {
        install::uninstall();
        return;
    }

    let config = server::load_config();
    let shared = Arc::new(Mutex::new(Shared {
        engine: Engine::new(config),
        candidates: CandidateLog::default(),
        last: None,
    }));

    let Some(hub) = server::start(shared.clone()) else {
        return;
    };

    install::ensure_registered();

    poll_loop(shared, hub);
}

fn poll_loop(shared: Arc<Mutex<Shared>>, hub: server::Hub) {
    unsafe {
        let _ = CoInitializeEx(None, COINIT_MULTITHREADED);
    }

    let mut monitor = match AudioMonitor::new() {
        Ok(m) => m,
        Err(_) => return,
    };

    let self_pid = unsafe { GetCurrentProcessId() };
    let mut tick: u64 = 0;
    let mut live_pids: Vec<u32> = Vec::new();

    loop {
        let started = Instant::now();

        if !hub.has_clients() {
            std::thread::sleep(Duration::from_millis(500));
            continue;
        }

        let sessions = match monitor.poll() {
            Ok(s) => s,
            Err(_) => {
                std::thread::sleep(Duration::from_millis(250));
                continue;
            }
        };

        let foreground = {
            let s = shared.lock().unwrap();
            if s.engine.wants_foreground() {
                drop(s);
                foreground_process(self_pid)
            } else {
                None
            }
        };

        tick += 1;
        let heartbeat = tick % 4 == 0;

        let sending = {
            let mut s = shared.lock().unwrap();
            let (decision, seen) = s.engine.evaluate(&sessions, foreground.as_deref(), started);

            let threshold = s.engine.config.audible_threshold;
            for (proc, peak) in &seen {
                if *peak > threshold {
                    s.candidates.note(proc, *peak);
                } else {
                    s.candidates.mark_silent(proc);
                }
            }
            s.candidates.mark_missing_silent(&seen);

            let changed = s.last.as_ref() != Some(&decision);
            if changed {
                s.last = Some(decision.clone());
            }

            if changed || heartbeat {
                Some((decision, s.candidates.pending(CANDIDATE_MIN_HITS)))
            } else {
                None
            }
        };

        if let Some((decision, candidates)) = sending {
            hub.broadcast_decision(&decision, &candidates);
        }

        if tick % 600 == 0 {
            live_pids.clear();
            live_pids.extend(sessions.iter().map(|s| s.pid));
            monitor.prune_cache(&live_pids);
        }

        let elapsed = started.elapsed();
        if elapsed < POLL_INTERVAL {
            std::thread::sleep(POLL_INTERVAL - elapsed);
        }
    }
}

fn foreground_process(self_pid: u32) -> Option<String> {
    unsafe {
        let hwnd: HWND = GetForegroundWindow();
        if hwnd.0.is_null() {
            return None;
        }
        let mut pid: u32 = 0;
        GetWindowThreadProcessId(hwnd, Some(&mut pid));
        if pid == 0 || pid == self_pid {
            return None;
        }
        audio::process_name_public(pid)
    }
}
