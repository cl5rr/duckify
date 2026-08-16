use std::sync::atomic::{AtomicBool, AtomicI32, Ordering};
use std::sync::Mutex;

use windows::core::w;
use windows::Win32::Foundation::{COLORREF, HWND, LPARAM, LRESULT, RECT, WPARAM};
use windows::Win32::Graphics::Gdi::{
    BeginPaint, CreateFontW, CreatePen, CreateSolidBrush, DeleteObject, DrawTextW, EndPaint,
    FillRect, InvalidateRect, RoundRect, SelectObject, SetBkMode, SetTextColor,
    CLEARTYPE_QUALITY, CLIP_DEFAULT_PRECIS, DEFAULT_CHARSET, DT_CENTER, DT_LEFT, DT_SINGLELINE,
    DT_VCENTER, DT_WORDBREAK, FF_DONTCARE, FW_BOLD, FW_NORMAL, OUT_TT_PRECIS, PAINTSTRUCT,
    PS_SOLID, TRANSPARENT, VARIABLE_PITCH,
};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::WindowsAndMessaging::*;

pub const W: i32 = 460;
pub const H: i32 = 300;

pub const ID_PRIMARY: usize = 1;
pub const ID_SECONDARY: usize = 2;
pub const ID_TERTIARY: usize = 3;

pub static CHOICE: AtomicI32 = AtomicI32::new(0);
pub static BUSY: AtomicBool = AtomicBool::new(false);

static PROGRESS: AtomicI32 = AtomicI32::new(-1);

pub struct Ui {
    pub title: String,
    pub body: String,
    pub primary: String,
    pub secondary: Option<String>,
    pub tertiary: Option<String>,
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

pub fn animate_open(hwnd: HWND) {
    unsafe {
        let sw = GetSystemMetrics(SM_CXSCREEN);
        let sh = GetSystemMetrics(SM_CYSCREEN);

        let steps = 14;
        for i in 0..=steps {
            let t = i as f32 / steps as f32;

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

unsafe fn button_rects(width: i32, height: i32, count: usize) -> [RECT; 3] {
    let bh = 40;
    let gap = 10;
    let y = height - bh - 28;
    let margin = 28;
    let usable = width - margin * 2;

    let bw = match count {
        0 | 1 => 200,
        2 => 150,
        _ => (usable - gap * 2) / 3,
    };

    let n = count.max(1) as i32;
    let total = bw * n + gap * (n - 1);
    let x = (width - total) / 2;

    let mut out = [RECT::default(); 3];
    for i in 0..n {
        let left = x + (bw + gap) * i;
        out[i as usize] = RECT { left, top: y, right: left + bw, bottom: y + bh };
    }
    out
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

                let old = SelectObject(hdc, title_font);
                SetTextColor(hdc, rgb(0xFF, 0xFF, 0xFF));
                let mut tr = RECT { left: 32, top: 34, right: width - 32, bottom: 84 };
                let mut t: Vec<u16> = ui.title.encode_utf16().collect();
                DrawTextW(hdc, &mut t, &mut tr, DT_LEFT | DT_SINGLELINE);
                SelectObject(hdc, old);

                SelectObject(hdc, body_font);
                SetTextColor(hdc, rgb(0xA8, 0xA8, 0xB0));
                let mut br = RECT { left: 32, top: 92, right: width - 32, bottom: height - 90 };
                let mut b: Vec<u16> = ui.body.encode_utf16().collect();
                DrawTextW(hdc, &mut b, &mut br, DT_LEFT | DT_WORDBREAK);

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

                if !BUSY.load(Ordering::Relaxed) {
                    let labels: Vec<&String> = [Some(&ui.primary), ui.secondary.as_ref(), ui.tertiary.as_ref()]
                        .into_iter()
                        .flatten()
                        .collect();
                    let rects = button_rects(width, height, labels.len());

                    SelectObject(hdc, btn_font);
                    for (i, label) in labels.iter().enumerate() {
                        let r = rects[i];
                        let fill = if i == 0 {
                            rgb(0xF5, 0xA6, 0x23)
                        } else {
                            rgb(0x14, 0x14, 0x16)
                        };
                        let edge = if i == 0 {
                            rgb(0xF5, 0xA6, 0x23)
                        } else {
                            rgb(0x44, 0x44, 0x4C)
                        };

                        let brush = CreateSolidBrush(fill);
                        let pen = CreatePen(PS_SOLID, 1, edge);
                        let ob = SelectObject(hdc, brush);
                        let op = SelectObject(hdc, pen);
                        let _ = RoundRect(hdc, r.left, r.top, r.right, r.bottom, 8, 8);
                        SelectObject(hdc, ob);
                        SelectObject(hdc, op);
                        let _ = DeleteObject(brush);
                        let _ = DeleteObject(pen);

                        SetTextColor(
                            hdc,
                            if i == 0 { rgb(0x14, 0x14, 0x16) } else { rgb(0xD0, 0xD0, 0xD8) },
                        );
                        let mut rr = r;
                        let mut s: Vec<u16> = label.encode_utf16().collect();
                        DrawTextW(hdc, &mut s, &mut rr, DT_CENTER | DT_VCENTER | DT_SINGLELINE);
                    }
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
            let count = UI
                .lock()
                .unwrap()
                .as_ref()
                .map(|u| 1 + u.secondary.is_some() as usize + u.tertiary.is_some() as usize)
                .unwrap_or(1);
            let rects = button_rects(rc.right, rc.bottom, count);

            let hit = |r: RECT| x >= r.left && x <= r.right && y >= r.top && y <= r.bottom;
            for i in 0..count {
                if hit(rects[i]) {
                    CHOICE.store((i + 1) as i32, Ordering::Relaxed);
                    let _ = PostMessageW(hwnd, WM_APP, WPARAM(0), LPARAM(0));
                    break;
                }
            }
            LRESULT(0)
        }

        WM_KEYDOWN => {
            if wparam.0 == 0x1B && !BUSY.load(Ordering::Relaxed) {
                let _ = DestroyWindow(hwnd);
            }
            LRESULT(0)
        }

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
