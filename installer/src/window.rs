use std::sync::atomic::{AtomicBool, AtomicI32, Ordering};
use std::sync::Mutex;

use windows::core::{w, PCWSTR};
use windows::Win32::Foundation::{COLORREF, HWND, LPARAM, LRESULT, RECT, SIZE, WPARAM};
use windows::Win32::Graphics::Gdi::{
    BeginPaint, CreateFontW, CreateSolidBrush, DeleteObject, DrawTextW, EndPaint, FillRect,
    InvalidateRect, RoundRect, SelectObject, SetBkMode, SetTextColor, CreatePen, GetDC, ReleaseDC,
    DT_CENTER, DT_LEFT, DT_SINGLELINE, DT_VCENTER, DT_WORDBREAK, FW_BOLD, FW_NORMAL, HBRUSH,
    PAINTSTRUCT, PS_SOLID, TRANSPARENT, ANTIALIASED_QUALITY, CLEARTYPE_QUALITY, DEFAULT_CHARSET,
    FF_DONTCARE, OUT_TT_PRECIS, CLIP_DEFAULT_PRECIS, VARIABLE_PITCH,
};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::WindowsAndMessaging::*;

pub const W: i32 = 460;
pub const H: i32 = 300;

pub const ID_PRIMARY: usize = 1;
pub const ID_SECONDARY: usize = 2;

/// Which button the user pressed, read by main after the loop ends.
pub static CHOICE: AtomicI32 = AtomicI32::new(0);
pub static BUSY: AtomicBool = AtomicBool::new(false);

static PROGRESS: AtomicI32 = AtomicI32::new(-1);

pub struct Ui {
    pub title: String,
    pub body: String,
    pub primary: String,
    pub secondary: Option<String>,
}

static UI: Mutex<Option<Ui>> = Mutex::new(None);

pub fn set_ui(ui: Ui) {
    *UI.lock().unwrap() = Some(ui);
}

pub fn set_status(text: &str, progress: i32) {
    if let Some(ui) = UI.lock().unwrap().as_mut() {
        ui.body = text.to_string();
    }
    PROGRESS.store(progress, Ordering::Relaxed);
}

pub fn redraw(hwnd: HWND) {
    unsafe {
        let _ = InvalidateRect(hwnd, None, true);
    }
}

/// Create the window centered on the primary monitor, always on top, with no
/// frame the user could drag or resize.
pub fn create() -> windows::core::Result<HWND> {
    unsafe {
        let instance = GetModuleHandleW(None)?;
        let class = w!("DuckifySetup");

        let wc = WNDCLASSW {
            hCursor: LoadCursorW(None, IDC_ARROW)?,
            hInstance: instance.into(),
            lpszClassName: class,
            style: CS_HREDRAW | CS_VREDRAW,
            lpfnWndProc: Some(wndproc),
            ..Default::default()
        };
        RegisterClassW(&wc);

        let sw = GetSystemMetrics(SM_CXSCREEN);
        let sh = GetSystemMetrics(SM_CYSCREEN);
        let x = (sw - W) / 2;
        let y = (sh - H) / 2;

        // WS_POPUP: no title bar, so there is nothing to drag and no system
        // menu. TOPMOST keeps it above other windows; TOOLWINDOW keeps it out
        // of the taskbar and alt-tab.
        let hwnd = CreateWindowExW(
            WS_EX_TOPMOST | WS_EX_TOOLWINDOW,
            class,
            w!("Duckify"),
            WS_POPUP,
            x,
            y,
            W,
            H,
            None,
            None,
            instance,
            None,
        )?;

        Ok(hwnd)
    }
}

/// Grow the window from a small centered rectangle to full size, so it reads as
/// opening toward the user rather than appearing abruptly.
pub fn animate_open(hwnd: HWND) {
    unsafe {
        let sw = GetSystemMetrics(SM_CXSCREEN);
        let sh = GetSystemMetrics(SM_CYSCREEN);

        let steps = 14;
        for i in 0..=steps {
            let t = i as f32 / steps as f32;
            // Ease-out cubic: fast at the start, settling at the end.
            let e = 1.0 - (1.0 - t).powi(3);
            let scale = 0.82 + 0.18 * e;

            let cw = (W as f32 * scale) as i32;
            let ch = (H as f32 * scale) as i32;
            let cx = (sw - cw) / 2;
            let cy = (sh - ch) / 2;

            let _ = SetWindowPos(
                hwnd,
                HWND_TOPMOST,
                cx,
                cy,
                cw,
                ch,
                SWP_NOACTIVATE | SWP_SHOWWINDOW,
            );
            let _ = InvalidateRect(hwnd, None, true);
            std::thread::sleep(std::time::Duration::from_millis(11));
        }

        let _ = SetWindowPos(
            hwnd,
            HWND_TOPMOST,
            (sw - W) / 2,
            (sh - H) / 2,
            W,
            H,
            SWP_NOACTIVATE | SWP_SHOWWINDOW,
        );
        let _ = SetForegroundWindow(hwnd);
    }
}

fn rgb(r: u8, g: u8, b: u8) -> COLORREF {
    COLORREF(r as u32 | ((g as u32) << 8) | ((b as u32) << 16))
}

unsafe fn button_rects(width: i32, height: i32, two: bool) -> (RECT, RECT) {
    let bw = if two { 150 } else { 200 };
    let bh = 40;
    let gap = 12;
    let y = height - bh - 28;

    if two {
        let total = bw * 2 + gap;
        let x = (width - total) / 2;
        (
            RECT { left: x, top: y, right: x + bw, bottom: y + bh },
            RECT { left: x + bw + gap, top: y, right: x + total, bottom: y + bh },
        )
    } else {
        let x = (width - bw) / 2;
        (
            RECT { left: x, top: y, right: x + bw, bottom: y + bh },
            RECT::default(),
        )
    }
}

unsafe extern "system" fn wndproc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match msg {
        WM_PAINT => {
            let mut ps = PAINTSTRUCT::default();
            let hdc = BeginPaint(hwnd, &mut ps);

            let mut rc = RECT::default();
            let _ = GetClientRect(hwnd, &mut rc);
            let width = rc.right - rc.left;
            let height = rc.bottom - rc.top;

            // Ground
            let bg = CreateSolidBrush(rgb(0x14, 0x14, 0x16));
            FillRect(hdc, &rc, bg);
            let _ = DeleteObject(bg);

            SetBkMode(hdc, TRANSPARENT);

            let title_font = CreateFontW(
                34, 0, 0, 0, FW_BOLD.0 as i32, 0, 0, 0,
                DEFAULT_CHARSET.0 as u32, OUT_TT_PRECIS.0 as u32,
                CLIP_DEFAULT_PRECIS.0 as u32, CLEARTYPE_QUALITY.0 as u32,
                (VARIABLE_PITCH.0 | FF_DONTCARE.0) as u32, w!("Segoe UI"),
            );
            let body_font = CreateFontW(
                17, 0, 0, 0, FW_NORMAL.0 as i32, 0, 0, 0,
                DEFAULT_CHARSET.0 as u32, OUT_TT_PRECIS.0 as u32,
                CLIP_DEFAULT_PRECIS.0 as u32, CLEARTYPE_QUALITY.0 as u32,
                (VARIABLE_PITCH.0 | FF_DONTCARE.0) as u32, w!("Segoe UI"),
            );
            let btn_font = CreateFontW(
                17, 0, 0, 0, FW_BOLD.0 as i32, 0, 0, 0,
                DEFAULT_CHARSET.0 as u32, OUT_TT_PRECIS.0 as u32,
                CLIP_DEFAULT_PRECIS.0 as u32, CLEARTYPE_QUALITY.0 as u32,
                (VARIABLE_PITCH.0 | FF_DONTCARE.0) as u32, w!("Segoe UI"),
            );

            let guard = UI.lock().unwrap();
            if let Some(ui) = guard.as_ref() {
                // Title
                let old = SelectObject(hdc, title_font);
                SetTextColor(hdc, rgb(0xFF, 0xFF, 0xFF));
                let mut tr = RECT { left: 32, top: 34, right: width - 32, bottom: 84 };
                let mut t: Vec<u16> = ui.title.encode_utf16().collect();
                DrawTextW(hdc, &mut t, &mut tr, DT_LEFT | DT_SINGLELINE);
                SelectObject(hdc, old);

                // Body
                SelectObject(hdc, body_font);
                SetTextColor(hdc, rgb(0xA8, 0xA8, 0xB0));
                let mut br = RECT { left: 32, top: 92, right: width - 32, bottom: height - 90 };
                let mut b: Vec<u16> = ui.body.encode_utf16().collect();
                DrawTextW(hdc, &mut b, &mut br, DT_LEFT | DT_WORDBREAK);

                // Progress bar, only while working.
                let p = PROGRESS.load(Ordering::Relaxed);
                if p >= 0 {
                    let track = RECT {
                        left: 32,
                        top: height - 96,
                        right: width - 32,
                        bottom: height - 92,
                    };
                    let tb = CreateSolidBrush(rgb(0x2A, 0x2A, 0x30));
                    FillRect(hdc, &track, tb);
                    let _ = DeleteObject(tb);

                    let filled = RECT {
                        left: 32,
                        top: height - 96,
                        right: 32 + ((width - 64) * p.clamp(0, 100) / 100),
                        bottom: height - 92,
                    };
                    let fb = CreateSolidBrush(rgb(0xF5, 0xA6, 0x23));
                    FillRect(hdc, &filled, fb);
                    let _ = DeleteObject(fb);
                }

                // Buttons
                if !BUSY.load(Ordering::Relaxed) {
                    let two = ui.secondary.is_some();
                    let (r1, r2) = button_rects(width, height, two);

                    let accent = CreateSolidBrush(rgb(0xF5, 0xA6, 0x23));
                    let pen = CreatePen(PS_SOLID, 1, rgb(0xF5, 0xA6, 0x23));
                    let oldpen = SelectObject(hdc, pen);
                    let oldbrush = SelectObject(hdc, accent);
                    let _ = RoundRect(hdc, r1.left, r1.top, r1.right, r1.bottom, 8, 8);
                    SelectObject(hdc, oldbrush);

                    SelectObject(hdc, btn_font);
                    SetTextColor(hdc, rgb(0x14, 0x14, 0x16));
                    let mut p1 = r1;
                    let mut s1: Vec<u16> = ui.primary.encode_utf16().collect();
                    DrawTextW(hdc, &mut s1, &mut p1, DT_CENTER | DT_VCENTER | DT_SINGLELINE);

                    if let Some(sec) = &ui.secondary {
                        let dark = CreateSolidBrush(rgb(0x14, 0x14, 0x16));
                        let gpen = CreatePen(PS_SOLID, 1, rgb(0x44, 0x44, 0x4C));
                        SelectObject(hdc, gpen);
                        SelectObject(hdc, dark);
                        let _ = RoundRect(hdc, r2.left, r2.top, r2.right, r2.bottom, 8, 8);
                        let _ = DeleteObject(dark);

                        SetTextColor(hdc, rgb(0xD0, 0xD0, 0xD8));
                        let mut p2 = r2;
                        let mut s2: Vec<u16> = sec.encode_utf16().collect();
                        DrawTextW(hdc, &mut s2, &mut p2, DT_CENTER | DT_VCENTER | DT_SINGLELINE);
                        let _ = DeleteObject(gpen);
                    }

                    SelectObject(hdc, oldpen);
                    let _ = DeleteObject(pen);
                    let _ = DeleteObject(accent);
                }
            }

            drop(guard);
            let _ = DeleteObject(title_font);
            let _ = DeleteObject(body_font);
            let _ = DeleteObject(btn_font);
            let _ = EndPaint(hwnd, &ps);
            LRESULT(0)
        }

        WM_LBUTTONUP => {
            if BUSY.load(Ordering::Relaxed) {
                return LRESULT(0);
            }
            let x = (lparam.0 & 0xFFFF) as i16 as i32;
            let y = ((lparam.0 >> 16) & 0xFFFF) as i16 as i32;

            let mut rc = RECT::default();
            let _ = GetClientRect(hwnd, &mut rc);
            let two = UI
                .lock()
                .unwrap()
                .as_ref()
                .map(|u| u.secondary.is_some())
                .unwrap_or(false);
            let (r1, r2) = button_rects(rc.right, rc.bottom, two);

            let hit = |r: RECT| x >= r.left && x <= r.right && y >= r.top && y <= r.bottom;
            if hit(r1) {
                CHOICE.store(ID_PRIMARY as i32, Ordering::Relaxed);
                let _ = PostMessageW(hwnd, WM_APP, WPARAM(0), LPARAM(0));
            } else if two && hit(r2) {
                CHOICE.store(ID_SECONDARY as i32, Ordering::Relaxed);
                let _ = PostMessageW(hwnd, WM_APP, WPARAM(0), LPARAM(0));
            }
            LRESULT(0)
        }

        // Refuse to be moved: the window stays where it was placed.
        WM_NCHITTEST => LRESULT(HTCLIENT as isize),
        WM_MOVING | WM_SIZING => LRESULT(1),

        WM_CLOSE => {
            if !BUSY.load(Ordering::Relaxed) {
                let _ = DestroyWindow(hwnd);
            }
            LRESULT(0)
        }
        WM_DESTROY => {
            PostQuitMessage(0);
            LRESULT(0)
        }
        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}
