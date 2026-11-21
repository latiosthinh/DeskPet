//! Thin wrappers over the Win32 calls DeskPet needs.

use std::ffi::c_void;
use std::ptr::{null, null_mut};
use windows_sys::Win32::Foundation::*;
use windows_sys::Win32::Graphics::Gdi::*;
use windows_sys::Win32::System::LibraryLoader::*;
use windows_sys::Win32::System::Registry::*;
use windows_sys::Win32::System::SystemInformation::*;
use windows_sys::Win32::UI::HiDpi::*;
use windows_sys::Win32::UI::Input::KeyboardAndMouse::*;
use windows_sys::Win32::UI::Shell::*;
use windows_sys::Win32::UI::WindowsAndMessaging::*;

/// A null-terminated UTF-16 copy of `s`.
pub fn wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}

// MARK: Fonts and text

pub struct Font(HFONT);

impl Font {
    /// `px` is the em height in device pixels.
    pub fn new(face: &str, px: i32, weight: i32) -> Font {
        let face = wide(face);
        let h = unsafe {
            CreateFontW(-px, 0, 0, 0, weight, 0, 0, 0, DEFAULT_CHARSET as u32, OUT_TT_PRECIS as u32, CLIP_DEFAULT_PRECIS as u32,
                        ANTIALIASED_QUALITY as u32, (DEFAULT_PITCH | FF_DONTCARE) as u32, face.as_ptr())
        };
        Font(h)
    }
    pub fn handle(&self) -> HFONT {
        self.0
    }
}

impl Drop for Font {
    fn drop(&mut self) {
        unsafe { DeleteObject(self.0) };
    }
}

fn screen_dc<T>(f: impl FnOnce(HDC) -> T) -> T {
    unsafe {
        let dc = GetDC(null_mut());
        let out = f(dc);
        ReleaseDC(null_mut(), dc);
        out
    }
}

/// Width and line height of `text` in device pixels.
pub fn text_extent(font: HFONT, text: &str) -> (i32, i32) {
    let w: Vec<u16> = text.encode_utf16().collect();
    screen_dc(|dc| unsafe {
        let old = SelectObject(dc, font);
        let mut size = SIZE { cx: 0, cy: 0 };
        GetTextExtentPoint32W(dc, w.as_ptr(), w.len() as i32, &mut size);
        SelectObject(dc, old);
        (size.cx, size.cy)
    })
}

// MARK: DIB sections

/// A top-down 32 bpp DIB section selected into its own memory DC.
pub struct Dib {
    pub dc: HDC,
    bitmap: HBITMAP,
    old: HGDIOBJ,
    bits: *mut u32,
    pub w: i32,
    pub h: i32,
}

pub fn create_dib(w: i32, h: i32) -> (HBITMAP, *mut u32) {
    let info = BITMAPINFO {
        bmiHeader: BITMAPINFOHEADER {
            biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
            biWidth: w,
            biHeight: -h,
            biPlanes: 1,
            biBitCount: 32,
            biCompression: BI_RGB,
            biSizeImage: 0,
            biXPelsPerMeter: 0,
            biYPelsPerMeter: 0,
            biClrUsed: 0,
            biClrImportant: 0,
        },
        bmiColors: [RGBQUAD { rgbBlue: 0, rgbGreen: 0, rgbRed: 0, rgbReserved: 0 }],
    };
    let mut bits: *mut c_void = null_mut();
    let bmp = unsafe { CreateDIBSection(null_mut(), &info, DIB_RGB_COLORS, &mut bits, null_mut(), 0) };
    (bmp, bits as *mut u32)
}

impl Dib {
    pub fn new(w: i32, h: i32) -> Dib {
        unsafe {
            let dc = CreateCompatibleDC(null_mut());
            let (bitmap, bits) = create_dib(w, h);
            let old = SelectObject(dc, bitmap);
            Dib { dc, bitmap, old, bits, w, h }
        }
    }

    pub fn pixels(&mut self) -> &mut [u32] {
        unsafe { std::slice::from_raw_parts_mut(self.bits, (self.w * self.h) as usize) }
    }

    /// Draws white text on black at the top left and returns the pixels (row stride `self.w`).
    pub fn draw_text(&mut self, font: HFONT, text: &str, w: i32, h: i32) -> &[u32] {
        let t: Vec<u16> = text.encode_utf16().collect();
        unsafe {
            let rect = RECT { left: 0, top: 0, right: w, bottom: h };
            FillRect(self.dc, &rect, GetStockObject(BLACK_BRUSH) as HBRUSH);
            let old = SelectObject(self.dc, font);
            SetTextColor(self.dc, 0x00FF_FFFF);
            SetBkMode(self.dc, TRANSPARENT as _);
            TextOutW(self.dc, 0, 0, t.as_ptr(), t.len() as i32);
            SelectObject(self.dc, old);
            GdiFlush();
        }
        &*self.pixels()
    }
}

impl Drop for Dib {
    fn drop(&mut self) {
        unsafe {
            SelectObject(self.dc, self.old);
            DeleteObject(self.bitmap);
            DeleteDC(self.dc);
        }
    }
}

/// Pushes a premultiplied DIB to a layered window at (x, y) in screen pixels.
/// `dirty` (in DIB pixels) limits the copy to the part that changed.
pub fn update_layered(hwnd: HWND, dib: &Dib, x: i32, y: i32, dirty: Option<RECT>) {
    let dst = POINT { x, y };
    let size = SIZE { cx: dib.w, cy: dib.h };
    let src = POINT { x: 0, y: 0 };
    let blend = BLENDFUNCTION { BlendOp: AC_SRC_OVER as u8, BlendFlags: 0, SourceConstantAlpha: 255, AlphaFormat: AC_SRC_ALPHA as u8 };
    let info = UPDATELAYEREDWINDOWINFO {
        cbSize: std::mem::size_of::<UPDATELAYEREDWINDOWINFO>() as u32,
        hdcDst: null_mut(),
        pptDst: &dst,
        psize: &size,
        hdcSrc: dib.dc,
        pptSrc: &src,
        crKey: 0,
        pblend: &blend,
        dwFlags: ULW_ALPHA,
        prcDirty: dirty.as_ref().map_or(null(), |r| r as *const RECT),
    };
    unsafe { UpdateLayeredWindowIndirect(hwnd, &info) };
}

// MARK: Bitmaps and icons from pixels

/// A 32 bpp DIB holding `pixels` (ARGB, rows top-down). The caller owns the bitmap.
pub fn bitmap_from_pixels(pixels: &[u32], size: i32) -> HBITMAP {
    let (bmp, bits) = create_dib(size, size);
    unsafe { std::ptr::copy_nonoverlapping(pixels.as_ptr(), bits, pixels.len()) };
    bmp
}

pub fn icon_from_pixels(pixels: &[u32], size: i32) -> HICON {
    unsafe {
        let color = bitmap_from_pixels(pixels, size);
        let zeros = vec![0u8; (((size + 15) / 16) * 2 * size) as usize]; // AND mask: all drawn from colour + alpha
        let mask = CreateBitmap(size, size, 1, 1, zeros.as_ptr() as *const c_void);
        let info = ICONINFO { fIcon: TRUE, xHotspot: 0, yHotspot: 0, hbmMask: mask, hbmColor: color };
        let icon = CreateIconIndirect(&info);
        DeleteObject(color);
        DeleteObject(mask);
        icon
    }
}

pub fn destroy_icon(icon: HICON) {
    if !icon.is_null() {
        unsafe { DestroyIcon(icon) };
    }
}

pub fn delete_bitmap(b: HBITMAP) {
    unsafe { DeleteObject(b) };
}

// MARK: Windows

pub fn instance() -> HINSTANCE {
    unsafe { GetModuleHandleW(null()) }
}

pub fn register_class(name: &[u16], proc_: WNDPROC, background: HBRUSH) {
    unsafe {
        let wc = WNDCLASSEXW {
            cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
            style: 0,
            lpfnWndProc: proc_,
            cbClsExtra: 0,
            cbWndExtra: 0,
            hInstance: instance(),
            hIcon: LoadIconW(instance(), 1 as _),
            hCursor: LoadCursorW(null_mut(), IDC_ARROW),
            hbrBackground: background,
            lpszMenuName: null(),
            lpszClassName: name.as_ptr(),
            hIconSm: null_mut(),
        };
        RegisterClassExW(&wc);
    }
}

pub fn set_click_through(hwnd: HWND, through: bool) {
    unsafe {
        let ex = GetWindowLongPtrW(hwnd, GWL_EXSTYLE);
        let t = WS_EX_TRANSPARENT as isize;
        let new = if through { ex | t } else { ex & !t };
        if new != ex {
            SetWindowLongPtrW(hwnd, GWL_EXSTYLE, new);
        }
    }
}

pub fn keep_on_top(hwnd: HWND) {
    unsafe { SetWindowPos(hwnd, HWND_TOPMOST, 0, 0, 0, 0, SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE) };
}

pub fn show(hwnd: HWND, visible: bool) {
    unsafe { ShowWindow(hwnd, if visible { SW_SHOWNOACTIVATE } else { SW_HIDE }) };
}

pub fn cursor() -> (i32, i32) {
    let mut p = POINT { x: 0, y: 0 };
    unsafe { GetCursorPos(&mut p) };
    (p.x, p.y)
}


pub fn message_box(text: &str, warning: bool) {
    let t = wide(text);
    let c = wide("DeskPet");
    unsafe { MessageBoxW(null_mut(), t.as_ptr(), c.as_ptr(), if warning { MB_ICONWARNING } else { MB_ICONINFORMATION } | MB_TOPMOST) };
}

// MARK: Screen and DPI

pub struct ScreenInfo {
    pub work: RECT,
    pub dpi: u32,
}

/// The primary monitor's work area (screen minus taskbar) and DPI.
pub fn primary_screen() -> ScreenInfo {
    unsafe {
        let mon = MonitorFromPoint(POINT { x: 0, y: 0 }, MONITOR_DEFAULTTOPRIMARY);
        let mut info: MONITORINFO = std::mem::zeroed();
        info.cbSize = std::mem::size_of::<MONITORINFO>() as u32;
        GetMonitorInfoW(mon, &mut info);
        let (mut dx, mut dy) = (96u32, 96u32);
        if GetDpiForMonitor(mon, MDT_EFFECTIVE_DPI, &mut dx, &mut dy) != 0 || dx == 0 {
            dx = 96;
        }
        ScreenInfo { work: info.rcWork, dpi: dx }
    }
}

pub fn small_icon_size(dpi: u32) -> i32 {
    let v = unsafe { GetSystemMetricsForDpi(SM_CXSMICON, dpi) };
    if v > 0 { v } else { (16 * dpi as i32) / 96 }
}

// MARK: Input

/// Seconds since the last keyboard or mouse input anywhere in the session, and the raw tick stamp.
pub fn last_input() -> (f64, u32) {
    let mut info = LASTINPUTINFO { cbSize: std::mem::size_of::<LASTINPUTINFO>() as u32, dwTime: 0 };
    unsafe {
        if GetLastInputInfo(&mut info) == 0 {
            return (0.0, 0);
        }
        (GetTickCount().wrapping_sub(info.dwTime) as f64 / 1000.0, info.dwTime)
    }
}

/// True if a key that counts as typing is down (or was pressed since the last check):
/// everything except mouse buttons and modifier keys on their own.
pub fn typing_key_down() -> bool {
    (0x08..=0xFEi32).any(|vk| {
        let modifier = (0x10..=0x12).contains(&vk) || (0xA0..=0xA5).contains(&vk) || vk == 0x5B || vk == 0x5C;
        !modifier && unsafe { GetAsyncKeyState(vk) } as u16 & 0x8001 != 0
    })
}

pub fn capture(hwnd: HWND) {
    unsafe { SetCapture(hwnd) };
}

pub fn release_capture() {
    unsafe { ReleaseCapture() };
}

/// True while a fullscreen app, a fullscreen Direct3D game or presentation mode is in front.
pub fn fullscreen_app_in_front() -> bool {
    let mut state = 0;
    unsafe {
        if SHQueryUserNotificationState(&mut state) != 0 {
            return false;
        }
    }
    state == QUNS_BUSY || state == QUNS_RUNNING_D3D_FULL_SCREEN || state == QUNS_PRESENTATION_MODE
}

// MARK: Time

/// Local date as (year, month, day) and minutes since midnight.
pub fn local_time() -> ((u16, u16, u16), u32) {
    let mut t: SYSTEMTIME = unsafe { std::mem::zeroed() };
    unsafe { GetLocalTime(&mut t) };
    ((t.wYear, t.wMonth, t.wDay), t.wHour as u32 * 60 + t.wMinute as u32)
}

// MARK: Registry (HKCU)

pub fn reg_get_string(subkey: &str, name: &str) -> Option<String> {
    let (k, n) = (wide(subkey), wide(name));
    let mut len: u32 = 0;
    unsafe {
        if RegGetValueW(HKEY_CURRENT_USER, k.as_ptr(), n.as_ptr(), RRF_RT_REG_SZ, null_mut(), null_mut(), &mut len) != 0 {
            return None;
        }
        let mut buf = vec![0u16; (len as usize / 2) + 1];
        let mut len2 = (buf.len() * 2) as u32;
        if RegGetValueW(HKEY_CURRENT_USER, k.as_ptr(), n.as_ptr(), RRF_RT_REG_SZ, null_mut(), buf.as_mut_ptr() as *mut c_void, &mut len2) != 0 {
            return None;
        }
        let end = buf.iter().position(|&c| c == 0).unwrap_or(buf.len());
        Some(String::from_utf16_lossy(&buf[..end]))
    }
}

pub fn reg_get_dword(subkey: &str, name: &str) -> Option<u32> {
    let (k, n) = (wide(subkey), wide(name));
    let mut v: u32 = 0;
    let mut len = 4u32;
    let r = unsafe { RegGetValueW(HKEY_CURRENT_USER, k.as_ptr(), n.as_ptr(), RRF_RT_REG_DWORD, null_mut(), &mut v as *mut u32 as *mut c_void, &mut len) };
    (r == 0).then_some(v)
}

pub fn reg_get_binary_first_byte(subkey: &str, name: &str) -> Option<u8> {
    let (k, n) = (wide(subkey), wide(name));
    let mut buf = [0u8; 64];
    let mut len = buf.len() as u32;
    let r = unsafe { RegGetValueW(HKEY_CURRENT_USER, k.as_ptr(), n.as_ptr(), RRF_RT_REG_BINARY, null_mut(), buf.as_mut_ptr() as *mut c_void, &mut len) };
    (r == 0 && len > 0).then_some(buf[0])
}

pub fn reg_set_string(subkey: &str, name: &str, value: &str) -> bool {
    let (k, n, v) = (wide(subkey), wide(name), wide(value));
    unsafe { RegSetKeyValueW(HKEY_CURRENT_USER, k.as_ptr(), n.as_ptr(), REG_SZ, v.as_ptr() as *const c_void, (v.len() * 2) as u32) == 0 }
}

pub fn reg_set_dword(subkey: &str, name: &str, value: u32) -> bool {
    let (k, n) = (wide(subkey), wide(name));
    unsafe { RegSetKeyValueW(HKEY_CURRENT_USER, k.as_ptr(), n.as_ptr(), REG_DWORD, &value as *const u32 as *const c_void, 4) == 0 }
}

pub fn reg_delete(subkey: &str, name: &str) {
    let (k, n) = (wide(subkey), wide(name));
    unsafe { RegDeleteKeyValueW(HKEY_CURRENT_USER, k.as_ptr(), n.as_ptr()) };
}

pub fn exe_path() -> String {
    let mut buf = vec![0u16; 1024];
    let n = unsafe { GetModuleFileNameW(null_mut(), buf.as_mut_ptr(), buf.len() as u32) } as usize;
    String::from_utf16_lossy(&buf[..n])
}

// MARK: Single instance

/// Holds the named mutex for the life of the process. None if DeskPet is already running.
pub fn single_instance() -> Option<HANDLE> {
    let name = wide("Local\\DeskPet.SingleInstance");
    unsafe {
        let h = windows_sys::Win32::System::Threading::CreateMutexW(null(), TRUE, name.as_ptr());
        if h.is_null() || GetLastError() == ERROR_ALREADY_EXISTS {
            return None;
        }
        Some(h)
    }
}

// MARK: Native popup menus

/// A native Win32 popup menu, so the menu looks like the rest of Windows 11.
pub struct Menu {
    pub handle: HMENU,
    count: u32,
}

pub struct MenuItem<'a> {
    pub text: &'a str,
    pub id: u32, // 0 for an item that does nothing
    pub checked: bool,
    pub enabled: bool,
    pub bold: bool,
    pub radio: bool,
    pub bitmap: HBITMAP,
}

impl<'a> MenuItem<'a> {
    pub fn new(text: &'a str, id: u32) -> MenuItem<'a> {
        MenuItem { text, id, checked: false, enabled: true, bold: false, radio: false, bitmap: null_mut() }
    }
}

impl Menu {
    pub fn new() -> Menu {
        Menu { handle: unsafe { CreatePopupMenu() }, count: 0 }
    }

    fn insert(&mut self, info: &mut MENUITEMINFOW) {
        info.cbSize = std::mem::size_of::<MENUITEMINFOW>() as u32;
        unsafe { InsertMenuItemW(self.handle, self.count, TRUE, info) };
        self.count += 1;
    }

    pub fn add(&mut self, item: MenuItem) {
        let mut text = wide(&item.text.replace('&', "&&"));
        let mut info: MENUITEMINFOW = unsafe { std::mem::zeroed() };
        info.fMask = MIIM_ID | MIIM_STRING | MIIM_STATE | MIIM_FTYPE | if item.bitmap.is_null() { 0 } else { MIIM_BITMAP };
        info.fType = if item.radio { MFT_RADIOCHECK } else { MFT_STRING };
        info.fState = (if item.checked { MFS_CHECKED } else { 0 })
            | (if item.enabled { 0 } else { MFS_DISABLED })
            | (if item.bold { MFS_DEFAULT } else { 0 });
        info.wID = item.id;
        info.dwTypeData = text.as_mut_ptr();
        info.hbmpItem = item.bitmap;
        self.insert(&mut info);
    }

    pub fn submenu(&mut self, text: &str, sub: Menu) {
        let mut text = wide(text);
        let mut info: MENUITEMINFOW = unsafe { std::mem::zeroed() };
        info.fMask = MIIM_STRING | MIIM_SUBMENU;
        info.hSubMenu = sub.handle;
        info.dwTypeData = text.as_mut_ptr();
        self.insert(&mut info);
        std::mem::forget(sub); // destroyed with its parent
    }

    pub fn separator(&mut self) {
        let mut info: MENUITEMINFOW = unsafe { std::mem::zeroed() };
        info.fMask = MIIM_FTYPE;
        info.fType = MFT_SEPARATOR;
        self.insert(&mut info);
    }

    /// Shows the menu at the cursor and returns the chosen item's id (0 if none).
    pub fn show(&self, owner: HWND) -> u32 {
        let (x, y) = cursor();
        unsafe {
            SetForegroundWindow(owner);
            let id = TrackPopupMenuEx(self.handle, TPM_RIGHTBUTTON | TPM_BOTTOMALIGN | TPM_RETURNCMD | TPM_NONOTIFY, x, y, owner, null());
            PostMessageW(owner, WM_NULL, 0, 0);
            id as u32
        }
    }
}

impl Drop for Menu {
    fn drop(&mut self) {
        unsafe { DestroyMenu(self.handle) };
    }
}

/// Lets popup menus follow the system's dark mode, using the uxtheme exports Windows' own apps use.
pub fn allow_dark_menus() {
    unsafe {
        let name = wide("uxtheme.dll");
        let lib = LoadLibraryW(name.as_ptr());
        if lib.is_null() {
            return;
        }
        if let Some(set_mode) = GetProcAddress(lib, 135 as _) {
            let set_mode: unsafe extern "system" fn(i32) -> i32 = std::mem::transmute(set_mode);
            set_mode(1); // AllowDark
        }
        if let Some(flush) = GetProcAddress(lib, 136 as _) {
            let flush: unsafe extern "system" fn() = std::mem::transmute(flush);
            flush();
        }
    }
}

// MARK: Tray icon

pub const TRAY_CALLBACK: u32 = WM_APP + 1;

pub fn tray(hwnd: HWND, action: u32, icon: HICON, tip: &str) -> bool {
    unsafe {
        let mut data: NOTIFYICONDATAW = std::mem::zeroed();
        data.cbSize = std::mem::size_of::<NOTIFYICONDATAW>() as u32;
        data.hWnd = hwnd;
        data.uID = 1;
        data.uFlags = NIF_MESSAGE | NIF_ICON | NIF_TIP;
        data.uCallbackMessage = TRAY_CALLBACK;
        data.hIcon = icon;
        for (i, c) in tip.encode_utf16().take(127).enumerate() {
            data.szTip[i] = c;
        }
        Shell_NotifyIconW(action, &data) != 0
    }
}

pub const TRAY_ADD: u32 = NIM_ADD;
pub const TRAY_MODIFY: u32 = NIM_MODIFY;
pub const TRAY_DELETE: u32 = NIM_DELETE;

pub fn taskbar_created_message() -> u32 {
    let name = wide("TaskbarCreated");
    unsafe { RegisterWindowMessageW(name.as_ptr()) }
}
