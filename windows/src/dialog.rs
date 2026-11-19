//! The unlock-code dialog, laid out by hand at the monitor's DPI, Windows 11 style:
//! a white content area and a grey button bar along the bottom.

use crate::license;
use crate::sys::{self, wide};
use std::cell::{Cell, RefCell};
use std::ptr::{null, null_mut};
use std::rc::Rc;
use windows_sys::Win32::Foundation::*;
use windows_sys::Win32::Graphics::Gdi::*;
use windows_sys::Win32::UI::HiDpi::AdjustWindowRectExForDpi;
use windows_sys::Win32::UI::Input::KeyboardAndMouse::SetFocus;
use windows_sys::Win32::UI::WindowsAndMessaging::*;

const PROBLEM: &str = "Check the code and try again. It has 20 letters and numbers; dashes, spaces and capitals don't matter.";
const CUE: &str = "XXXXX-XXXXX-XXXXX-XXXXX";
const EM_SETCUEBANNER: u32 = 0x1501;
const EM_SETSEL: u32 = 0x00B1;
const EM_LIMITTEXT: u32 = 0x00C5;
const SS_NOPREFIX: u32 = 0x80;
const STYLE: u32 = WS_POPUP | WS_CAPTION | WS_SYSMENU;
const EX_STYLE: u32 = WS_EX_TOPMOST | WS_EX_CONTROLPARENT;

struct State {
    s: f64,
    dpi: u32,
    heading: HWND,
    body: HWND,
    edit: HWND,
    ok: HWND,
    cancel: HWND,
    heading_font: sys::Font,
    body_font: sys::Font,
    edit_font: sys::Font,
    white: HBRUSH,
    grey: HBRUSH,
    error: Cell<bool>,
    footer_top: Cell<i32>,
    done: Cell<bool>,
    result: RefCell<Option<String>>,
}

impl Drop for State {
    fn drop(&mut self) {
        unsafe {
            DeleteObject(self.white);
            DeleteObject(self.grey);
        }
    }
}

thread_local! {
    static DIALOG: RefCell<Option<Rc<State>>> = const { RefCell::new(None) };
}

fn state() -> Option<Rc<State>> {
    DIALOG.with(|d| d.borrow().clone())
}

fn rgb(r: u8, g: u8, b: u8) -> COLORREF {
    r as u32 | (g as u32) << 8 | (b as u32) << 16
}

fn set_text(hwnd: HWND, text: &str) {
    let t = wide(text);
    unsafe { SetWindowTextW(hwnd, t.as_ptr()) };
}

fn get_text(hwnd: HWND) -> String {
    let mut buf = [0u16; 128];
    let n = unsafe { GetWindowTextW(hwnd, buf.as_mut_ptr(), buf.len() as i32) } as usize;
    String::from_utf16_lossy(&buf[..n])
}

/// Height of `text` wrapped to `width` device pixels.
fn wrapped_height(font: HFONT, text: &str, width: i32) -> i32 {
    let t: Vec<u16> = text.encode_utf16().collect();
    unsafe {
        let dc = GetDC(null_mut());
        let old = SelectObject(dc, font);
        let mut r = RECT { left: 0, top: 0, right: width, bottom: 0 };
        DrawTextW(dc, t.as_ptr(), t.len() as i32, &mut r, DT_CALCRECT | DT_WORDBREAK | DT_NOPREFIX);
        SelectObject(dc, old);
        ReleaseDC(null_mut(), dc);
        r.bottom - r.top
    }
}

fn layout(hwnd: HWND, st: &State, center: bool) {
    let p = |v: f64| (v * st.s).round() as i32;
    let (pad, width) = (p(24.0), p(420.0));
    let inner = width - 2 * pad;
    let mut y = p(20.0);
    unsafe {
        let hh = wrapped_height(st.heading_font.handle(), &get_text(st.heading), inner);
        MoveWindow(st.heading, pad, y, inner, hh, TRUE);
        y += hh + p(8.0);

        let bh = wrapped_height(st.body_font.handle(), &get_text(st.body), inner);
        MoveWindow(st.body, pad, y, inner, bh + p(2.0), TRUE);
        y += bh + p(16.0);

        let eh = sys::text_extent(st.edit_font.handle(), "X").1 + p(9.0);
        MoveWindow(st.edit, pad, y, inner, eh, TRUE);
        y += eh + p(24.0);

        let (footer_h, bw, bh) = (p(72.0), p(100.0), p(32.0));
        st.footer_top.set(y);
        MoveWindow(st.cancel, width - pad - bw, y + (footer_h - bh) / 2, bw, bh, TRUE);
        MoveWindow(st.ok, width - pad - 2 * bw - p(8.0), y + (footer_h - bh) / 2, bw, bh, TRUE);

        let mut r = RECT { left: 0, top: 0, right: width, bottom: y + footer_h };
        AdjustWindowRectExForDpi(&mut r, STYLE, FALSE, EX_STYLE, st.dpi);
        let (ww, wh) = (r.right - r.left, r.bottom - r.top);
        if center {
            let wa = sys::primary_screen().work;
            let x = wa.left + (wa.right - wa.left - ww) / 2;
            let y = wa.top + (wa.bottom - wa.top - wh) / 2;
            SetWindowPos(hwnd, HWND_TOPMOST, x, y, ww, wh, SWP_NOACTIVATE);
        } else {
            SetWindowPos(hwnd, null_mut(), 0, 0, ww, wh, SWP_NOMOVE | SWP_NOZORDER | SWP_NOACTIVATE);
        }
        InvalidateRect(hwnd, null(), TRUE);
    }
}

fn paint_background(hwnd: HWND, dc: HDC, st: &State) {
    unsafe {
        let mut r = RECT { left: 0, top: 0, right: 0, bottom: 0 };
        GetClientRect(hwnd, &mut r);
        let split = st.footer_top.get().min(r.bottom);
        FillRect(dc, &RECT { bottom: split, ..r }, st.white);
        FillRect(dc, &RECT { top: split, ..r }, st.grey);
    }
}

fn try_unlock(hwnd: HWND, st: &State) {
    let code = get_text(st.edit);
    if license::is_valid(&code) {
        *st.result.borrow_mut() = Some(code.trim().to_string());
        st.done.set(true);
        unsafe { DestroyWindow(hwnd) };
        return;
    }
    st.error.set(true);
    set_text(st.heading, "That code didn't work");
    set_text(st.body, PROBLEM);
    layout(hwnd, st, false);
    unsafe {
        SetFocus(st.edit);
        SendMessageW(st.edit, EM_SETSEL, 0, -1);
    }
}

unsafe extern "system" fn proc_(hwnd: HWND, msg: u32, wp: WPARAM, lp: LPARAM) -> LRESULT {
    let Some(st) = state() else { return DefWindowProcW(hwnd, msg, wp, lp) };
    match msg {
        WM_ERASEBKGND => {
            paint_background(hwnd, wp as HDC, &st);
            return 1;
        }
        WM_PRINTCLIENT => {
            paint_background(hwnd, wp as HDC, &st);
            return 0;
        }
        WM_CTLCOLORSTATIC => {
            let dc = wp as HDC;
            let ctl = lp as HWND;
            let color = if ctl == st.heading {
                if st.error.get() { rgb(196, 43, 28) } else { rgb(26, 26, 26) }
            } else {
                rgb(70, 70, 70)
            };
            SetTextColor(dc, color);
            SetBkColor(dc, rgb(255, 255, 255));
            return st.white as LRESULT;
        }
        WM_CTLCOLORBTN => return st.grey as LRESULT,
        WM_COMMAND => {
            match (wp & 0xFFFF) as i32 {
                IDOK => try_unlock(hwnd, &st),
                IDCANCEL => {
                    st.done.set(true);
                    DestroyWindow(hwnd);
                }
                _ => {}
            }
            return 0;
        }
        WM_CLOSE => {
            st.done.set(true);
            DestroyWindow(hwnd);
            return 0;
        }
        _ => {}
    }
    DefWindowProcW(hwnd, msg, wp, lp)
}

fn child(parent: HWND, class: &str, text: &str, style: u32, ex: u32, id: i32, font: HFONT) -> HWND {
    let (c, t) = (wide(class), wide(text));
    unsafe {
        let h = CreateWindowExW(ex, c.as_ptr(), t.as_ptr(), WS_CHILD | WS_VISIBLE | style, 0, 0, 10, 10, parent, id as isize as HMENU, sys::instance(), null());
        SendMessageW(h, WM_SETFONT, font as WPARAM, 1);
        h
    }
}

/// Shows the dialog until the code works or the person cancels. Returns the code once it works.
pub fn ask_for_code(intro: &str, dpi: u32, big_icon: HICON, small_icon: HICON) -> Option<String> {
    let class = wide("DeskPetUnlock");
    thread_local!(static REGISTERED: Cell<bool> = const { Cell::new(false) });
    if !REGISTERED.with(|r| r.replace(true)) {
        sys::register_class(&class, Some(proc_), null_mut());
    }
    let s = dpi as f64 / 96.0;
    let p = |v: f64| (v * s).round() as i32;
    let title = wide("DeskPet");
    unsafe {
        let hwnd = CreateWindowExW(EX_STYLE, class.as_ptr(), title.as_ptr(), STYLE, 0, 0, 100, 100, null_mut(), null_mut(), sys::instance(), null());
        if hwnd.is_null() {
            return None;
        }
        SendMessageW(hwnd, WM_SETICON, ICON_BIG as WPARAM, big_icon as LPARAM);
        SendMessageW(hwnd, WM_SETICON, ICON_SMALL as WPARAM, small_icon as LPARAM);

        let heading_font = sys::Font::new("Segoe UI", p(20.0), 600);
        let body_font = sys::Font::new("Segoe UI", p(12.0), 400);
        let edit_font = sys::Font::new("Consolas", p(15.0), 400);
        let heading = child(hwnd, "STATIC", "Unlock All Pets", SS_NOPREFIX, 0, 100, heading_font.handle());
        let body = child(hwnd, "STATIC", intro, SS_NOPREFIX, 0, 101, body_font.handle());
        let edit = child(hwnd, "EDIT", "", WS_TABSTOP | ES_AUTOHSCROLL as u32 | ES_UPPERCASE as u32, WS_EX_CLIENTEDGE, 102, edit_font.handle());
        let ok = child(hwnd, "BUTTON", "Unlock", WS_TABSTOP | BS_DEFPUSHBUTTON as u32, 0, IDOK, body_font.handle());
        let cancel = child(hwnd, "BUTTON", "Cancel", WS_TABSTOP | BS_PUSHBUTTON as u32, 0, IDCANCEL, body_font.handle());
        SendMessageW(edit, EM_LIMITTEXT, 40, 0);
        let cue = wide(CUE);
        SendMessageW(edit, EM_SETCUEBANNER, 1, cue.as_ptr() as LPARAM);

        let st = Rc::new(State {
            s,
            dpi,
            heading,
            body,
            edit,
            ok,
            cancel,
            heading_font,
            body_font,
            edit_font,
            white: CreateSolidBrush(rgb(255, 255, 255)),
            grey: CreateSolidBrush(rgb(243, 243, 243)),
            error: Cell::new(false),
            footer_top: Cell::new(0),
            done: Cell::new(false),
            result: RefCell::new(None),
        });
        DIALOG.with(|d| *d.borrow_mut() = Some(st.clone()));
        layout(hwnd, &st, true);
        ShowWindow(hwnd, SW_SHOW);
        SetForegroundWindow(hwnd);
        SetFocus(edit);

        let mut msg: MSG = std::mem::zeroed();
        while !st.done.get() {
            let r = GetMessageW(&mut msg, null_mut(), 0, 0);
            if r == 0 {
                PostQuitMessage(msg.wParam as i32); // let the main loop see it too
                break;
            }
            if r == -1 {
                break;
            }
            if IsDialogMessageW(hwnd, &msg) == 0 {
                TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }
        }
        if IsWindow(hwnd) != 0 {
            DestroyWindow(hwnd);
        }
        DIALOG.with(|d| *d.borrow_mut() = None);
        let out = st.result.borrow_mut().take();
        out
    }
}
