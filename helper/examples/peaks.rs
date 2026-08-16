#[path = "../src/audio.rs"]
mod audio;

use std::collections::HashMap;
use std::time::{Duration, Instant};

use windows::Win32::System::Com::{CoInitializeEx, COINIT_MULTITHREADED};

fn main() {
    unsafe {
        let _ = CoInitializeEx(None, COINIT_MULTITHREADED);
    }

    let mut monitor = audio::AudioMonitor::new().expect("open audio endpoint");
    let mut peak_seen: HashMap<String, f32> = HashMap::new();
    let start = Instant::now();

    let all = monitor.debug_count_all_sessions();
    println!("sessions on default endpoint (any state): {all}");
    println!("sampling for 8s — play some audio…\n");

    while start.elapsed() < Duration::from_secs(8) {
        if let Ok(sessions) = monitor.poll() {
            for s in sessions {
                let e = peak_seen.entry(s.process).or_insert(0.0);
                if s.peak > *e {
                    *e = s.peak;
                }
            }
        }
        std::thread::sleep(Duration::from_millis(50));
    }

    let mut rows: Vec<_> = peak_seen.into_iter().collect();
    rows.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

    if rows.is_empty() {
        println!("no active audio sessions seen");
    }
    for (proc, peak) in rows {
        let audible = if peak > 0.01 { "AUDIBLE" } else { "silent" };
        println!("  {proc:<32} peak={peak:.4}  {audible}");
    }
}
