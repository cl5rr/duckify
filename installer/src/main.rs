#![windows_subsystem = "windows"]

mod actions;
mod window;

use std::sync::atomic::Ordering;

use windows::Win32::Foundation::HWND;
use windows::Win32::UI::WindowsAndMessaging::{
    DispatchMessageW, GetMessageW, PostQuitMessage, TranslateMessage, MSG, WM_APP,
};

use actions::*;
use window::{Ui, BUSY, CHOICE, ID_PRIMARY, ID_SECONDARY};

#[derive(Clone, Copy, PartialEq)]
enum Screen {
    NeedSpotify,
    NeedSpicetify,
    Update,
    Install,
    Remove,
    Done,
}

fn main() -> windows::core::Result<()> {
    let installed = is_installed();

    let mut screen = if !spotify_installed() {
        Screen::NeedSpotify
    } else if !spicetify_ready() {
        Screen::NeedSpicetify
    } else if installed {
        Screen::Remove
    } else {
        Screen::Install
    };

    let have = installed_version();

    let mut remote: Option<Release> = None;
    if screen == Screen::Install || screen == Screen::Remove {
        remote = latest_release();
        if let Some(r) = &remote {
            if is_newer(&r.tag, VERSION) {
                screen = Screen::Update;
            }
        }
    }

    window::set_ui(ui_for(screen, &remote, have.clone()));
    let hwnd = window::create()?;
    window::animate_open(hwnd);

    run_loop(hwnd, screen, remote);
    Ok(())
}

fn spicetify_ready() -> bool {
    spicetify().is_some()
}

fn ui_for(screen: Screen, remote: &Option<Release>, installed: Option<String>) -> Ui {
    match screen {
        Screen::NeedSpotify => Ui {
            title: "Spotify needed".into(),
            body: "Duckify works inside the Spotify desktop app, which is not on this computer.\n\nInstall Spotify, then Spicetify, then open this installer again.".into(),
            primary: "Get Spotify".into(),
            secondary: Some("Cancel".into()),
            tertiary: None,
        },
        Screen::NeedSpicetify => Ui {
            title: "Spicetify needed".into(),
            body: "Duckify adds a button inside Spotify, which needs Spicetify installed first.\n\nGet Spicetify, run it once, then open this installer again.".into(),
            primary: "Get Spicetify".into(),
            secondary: Some("Cancel".into()),
            tertiary: None,
        },
        Screen::Update => {
            let tag = remote
                .as_ref()
                .map(|r| r.tag.trim_start_matches(['v', 'V']).to_string())
                .unwrap_or_default();
            let body = match &installed {
                Some(have) => format!(
                    "You have Duckify {have}. Version {tag} is available.\n\nThis installer sets up version {VERSION}."
                ),
                None => format!(
                    "This installer sets up version {VERSION}, but version {tag} is available.\n\nYou can get the newer one, or install this version anyway."
                ),
            };
            Ui {
                title: "Update available".into(),
                body,
                primary: "Get newer version".into(),
                secondary: Some(if installed.is_some() {
                    format!("Reinstall {VERSION}")
                } else {
                    format!("Install {VERSION}")
                }),
                tertiary: Some("Cancel".into()),
            }
        }
        Screen::Install => Ui {
            title: "Install Duckify".into(),
            body: format!(
                "Duckify quiets Spotify when a game, a call, or a video is making sound, then brings it back afterwards.\n\nThis installs version {VERSION}: the background helper and the Spotify button."
            ),
            primary: "Install".into(),
            secondary: Some("Cancel".into()),
            tertiary: None,
        },
        Screen::Remove => {
            let have = installed.clone().unwrap_or_else(|| "unknown".into());
            let same = have == VERSION;
            Ui {
                title: "Duckify is installed".into(),
                body: if same {
                    format!(
                        "Version {have} is on this computer, and this installer is the same version.\n\nYou can reinstall it, which repairs a broken setup, or remove it completely."
                    )
                } else {
                    format!(
                        "Version {have} is on this computer. This installer has version {VERSION}.\n\nInstalling replaces what is there, or you can remove Duckify completely."
                    )
                },
                primary: if same {
                    "Reinstall".into()
                } else {
                    format!("Update to {VERSION}")
                },
                secondary: Some("Remove".into()),
                tertiary: Some("Cancel".into()),
            }
        }
        Screen::Done => Ui {
            title: "Finished".into(),
            body: String::new(),
            primary: "Close".into(),
            secondary: None,
            tertiary: None,
        },
    }
}

fn run_loop(hwnd: HWND, mut screen: Screen, remote: Option<Release>) {
    unsafe {
        let mut msg = MSG::default();
        while GetMessageW(&mut msg, None, 0, 0).into() {
            if msg.message == WM_APP {
                let choice = CHOICE.swap(0, Ordering::Relaxed) as usize;
                match (screen, choice) {
                    (Screen::NeedSpotify, ID_PRIMARY) => {
                        open_url("https://www.spotify.com/download/windows/");
                    }
                    (Screen::NeedSpicetify, ID_PRIMARY) => {
                        open_url("https://spicetify.app/");
                    }

                    (Screen::Update, ID_PRIMARY) => {
                        let url = remote
                            .as_ref()
                            .and_then(|r| r.url.clone())
                            .unwrap_or_else(|| format!("https://github.com/{REPO}/releases/latest"));
                        open_url(&url);
                        PostQuitMessage(0);
                    }
                    (Screen::Update, ID_SECONDARY) => {
                        screen = Screen::Done;
                        do_work(hwnd, true);
                    }

                    (Screen::Install, ID_PRIMARY) | (Screen::Remove, ID_PRIMARY) => {
                        screen = Screen::Done;
                        do_work(hwnd, true);
                    }
                    (Screen::Remove, ID_SECONDARY) => {
                        screen = Screen::Done;
                        do_work(hwnd, false);
                    }

                    _ => {
                        PostQuitMessage(0);
                    }
                }
                continue;
            }
            let _ = TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
    }
}

fn do_work(hwnd: HWND, installing: bool) {
    BUSY.store(true, Ordering::Relaxed);

    let report = |text: &str, pct: i32| {
        window::set_status(text, pct);
        window::redraw(hwnd);
        pump(hwnd);
    };

    let result = if installing {
        actions::install(report)
    } else {
        actions::uninstall(report)
    };

    BUSY.store(false, Ordering::Relaxed);

    let ui = match result {
        Ok(()) if installing => Ui {
            title: "Duckify is ready".into(),
            body: "It is running now and will start with Windows.\n\nOpen Spotify and look for the duck icon in the top bar, next to Marketplace.".into(),
            primary: "Close".into(),
            secondary: None,
            tertiary: None,
        },
        Ok(()) => Ui {
            title: "Duckify removed".into(),
            body: "The helper, the startup entry, and the Spotify button are gone.\n\nYour settings file was left in place in case you reinstall.".into(),
            primary: "Close".into(),
            secondary: None,
            tertiary: None,
        },
        Err(e) => Ui {
            title: "That did not work".into(),
            body: format!("{e}\n\nClosing Spotify and running this installer again usually fixes it."),
            primary: "Close".into(),
            secondary: None,
            tertiary: None,
        },
    };

    window::set_status("", -1);
    window::set_ui(ui);
    window::redraw(hwnd);
}

fn pump(_hwnd: HWND) {
    unsafe {
        use windows::Win32::UI::WindowsAndMessaging::{PeekMessageW, PM_REMOVE};
        let mut msg = MSG::default();
        while PeekMessageW(&mut msg, None, 0, 0, PM_REMOVE).as_bool() {
            let _ = TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
    }
}
