//! Settings in HKCU\Software\DeskPet, and Open at Login through the user's Run key.
//! Same names and value types as the earlier C# build, so settings carry over.

use crate::sys;

const KEY: &str = r"Software\DeskPet";

pub fn get_string(name: &str) -> Option<String> {
    sys::reg_get_string(KEY, name)
}
pub fn set_string(name: &str, value: &str) {
    sys::reg_set_string(KEY, name, value);
}
pub fn get_bool(name: &str, fallback: bool) -> bool {
    sys::reg_get_dword(KEY, name)
        .map(|v| v != 0)
        .unwrap_or(fallback)
}
pub fn set_bool(name: &str, value: bool) {
    sys::reg_set_dword(KEY, name, value as u32);
}
pub fn get_f64(name: &str) -> Option<f64> {
    get_string(name).and_then(|s| s.trim().parse().ok())
}
pub fn set_f64(name: &str, value: f64) {
    set_string(name, &value.to_string());
}
pub fn remove(name: &str) {
    sys::reg_delete(KEY, name);
}

/// Every unlock code entered on this PC. "codes" holds them separated by spaces; "license" is the
/// single code earlier versions kept.
#[derive(Clone, Copy, PartialEq)]
pub enum LoginState {
    Off,
    On,
    TurnedOffInTaskManager,
}

const RUN: &str = r"Software\Microsoft\Windows\CurrentVersion\Run";
const APPROVED: &str = r"Software\Microsoft\Windows\CurrentVersion\Explorer\StartupApproved\Run";
const NAME: &str = "DeskPet";

fn command() -> String {
    format!("\"{}\"", sys::exe_path())
}

/// Task Manager's Startup apps page can switch it off; that's kept separately from the Run key.
pub fn login_state() -> LoginState {
    if sys::reg_get_string(RUN, NAME).is_none() {
        return LoginState::Off;
    }
    match sys::reg_get_binary_first_byte(APPROVED, NAME) {
        Some(b) if b & 1 == 1 => LoginState::TurnedOffInTaskManager,
        _ => LoginState::On,
    }
}

pub fn login_on() -> bool {
    let ok = sys::reg_set_string(RUN, NAME, &command());
    sys::reg_delete(APPROVED, NAME);
    ok
}

pub fn login_off() {
    sys::reg_delete(RUN, NAME);
    sys::reg_delete(APPROVED, NAME);
}

/// If DeskPet was moved, point the login entry at where it is now.
pub fn refresh_login_path() {
    if let Some(cmd) = sys::reg_get_string(RUN, NAME) {
        let now = command();
        if !cmd.eq_ignore_ascii_case(&now) {
            sys::reg_set_string(RUN, NAME, &now);
        }
    }
}
