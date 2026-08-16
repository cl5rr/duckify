use std::collections::HashMap;

use windows::core::Interface;
use windows::Win32::Foundation::{CloseHandle, MAX_PATH};
use windows::Win32::Media::Audio::Endpoints::IAudioMeterInformation;
use windows::Win32::Media::Audio::{
    eMultimedia, eRender, AudioSessionStateActive, IAudioSessionControl2, IAudioSessionManager2,
    IMMDeviceEnumerator, MMDeviceEnumerator,
};
use windows::Win32::System::Com::{CoCreateInstance, CLSCTX_ALL};
use windows::Win32::System::ProcessStatus::GetModuleBaseNameW;
use windows::Win32::System::Threading::{
    OpenProcess, PROCESS_QUERY_LIMITED_INFORMATION, PROCESS_VM_READ,
};

#[derive(Debug, Clone)]
pub struct SessionPeak {
    pub process: String,
    pub pid: u32,
    pub peak: f32,
}

pub struct AudioMonitor {
    enumerator: IMMDeviceEnumerator,
    name_cache: HashMap<u32, String>,
}

impl AudioMonitor {
    pub fn new() -> windows::core::Result<Self> {
        let enumerator: IMMDeviceEnumerator =
            unsafe { CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL)? };
        Ok(Self {
            enumerator,
            name_cache: HashMap::new(),
        })
    }

    pub fn poll(&mut self) -> windows::core::Result<Vec<SessionPeak>> {
        let mut out = Vec::new();

        let device = unsafe {
            self.enumerator
                .GetDefaultAudioEndpoint(eRender, eMultimedia)?
        };
        let manager: IAudioSessionManager2 =
            unsafe { device.Activate(CLSCTX_ALL, None)? };
        let sessions = unsafe { manager.GetSessionEnumerator()? };
        let count = unsafe { sessions.GetCount()? };

        for i in 0..count {
            let ctrl = match unsafe { sessions.GetSession(i) } {
                Ok(c) => c,
                Err(_) => continue,
            };

            if let Ok(state) = unsafe { ctrl.GetState() } {
                if state != AudioSessionStateActive {
                    continue;
                }
            }

            let ctrl2: IAudioSessionControl2 = match ctrl.cast() {
                Ok(c) => c,
                Err(_) => continue,
            };

            if unsafe { ctrl2.IsSystemSoundsSession() } == windows::Win32::Foundation::S_OK {
                continue;
            }

            let pid = match unsafe { ctrl2.GetProcessId() } {
                Ok(p) if p != 0 => p,
                _ => continue,
            };

            let meter: IAudioMeterInformation = match ctrl.cast() {
                Ok(m) => m,
                Err(_) => continue,
            };
            let peak = match unsafe { meter.GetPeakValue() } {
                Ok(p) => p,
                Err(_) => continue,
            };

            let process = self.resolve_name(pid);
            out.push(SessionPeak { process, pid, peak });
        }

        Ok(out)
    }

    fn resolve_name(&mut self, pid: u32) -> String {
        if let Some(name) = self.name_cache.get(&pid) {
            return name.clone();
        }
        let name = process_name(pid).unwrap_or_else(|| format!("pid:{pid}"));
        self.name_cache.insert(pid, name.clone());
        name
    }

    pub fn prune_cache(&mut self, live: &[u32]) {
        self.name_cache.retain(|pid, _| live.contains(pid));
    }

    #[allow(dead_code)]
    pub fn debug_count_all_sessions(&mut self) -> String {
        let mut lines = Vec::new();
        unsafe {
            let Ok(device) = self.enumerator.GetDefaultAudioEndpoint(eRender, eMultimedia)
            else {
                return "<no default endpoint>".into();
            };
            let manager: IAudioSessionManager2 = match device.Activate(CLSCTX_ALL, None) {
                Ok(m) => m,
                Err(e) => return format!("<activate failed: {e}>"),
            };
            let Ok(sessions) = manager.GetSessionEnumerator() else {
                return "<no enumerator>".into();
            };
            let count = sessions.GetCount().unwrap_or(0);
            lines.push(format!("{count} total"));

            for i in 0..count {
                let Ok(ctrl) = sessions.GetSession(i) else {
                    continue;
                };
                let state = ctrl.GetState().map(|s| s.0).unwrap_or(-1);
                let pid = ctrl
                    .cast::<IAudioSessionControl2>()
                    .ok()
                    .and_then(|c| c.GetProcessId().ok())
                    .unwrap_or(0);
                let peak = ctrl
                    .cast::<IAudioMeterInformation>()
                    .ok()
                    .and_then(|m| m.GetPeakValue().ok())
                    .unwrap_or(-1.0);
                let name = process_name(pid).unwrap_or_else(|| format!("pid:{pid}"));
                lines.push(format!("      {name} state={state} peak={peak:.4}"));
            }
        }
        lines.join("\n")
    }
}

pub fn process_name_public(pid: u32) -> Option<String> {
    process_name(pid)
}

fn process_name(pid: u32) -> Option<String> {
    unsafe {
        let handle = OpenProcess(
            PROCESS_QUERY_LIMITED_INFORMATION | PROCESS_VM_READ,
            false,
            pid,
        )
        .ok()?;

        let mut buf = [0u16; MAX_PATH as usize];
        let len = GetModuleBaseNameW(handle, None, &mut buf);
        let _ = CloseHandle(handle);

        if len == 0 {
            return None;
        }
        Some(String::from_utf16_lossy(&buf[..len as usize]).to_lowercase())
    }
}
