//! The C interface the Mac app (Swift) uses to run pet modules, the same `.deskpet` files the
//! Windows app runs, through the shared host crate. Declared for Swift in `deskpet.h`.
//!
//! The app owns the window, menus, input, settings and timers; this side owns the pets: the list
//! (built in and installed), the running pet's module, and drawing it into the app's pixel buffer.
//! Text is drawn by the app (a [`TextFn`]), and the update check uses the app's HTTP (a [`GetFn`]).
//!
//! Everything except [`dp_update`] runs on the app's main thread. Strings returned stay valid until
//! the next call that returns a string; the app copies them straight away.

// Every function takes the host pointer dp_new returned (or the pointers its docs name), from the app.
#![allow(clippy::missing_safety_doc)]

use deskpet_host::canvas::{Canvas, Font, Mask, TextCache, TextRaster};
use deskpet_host::catalog::{self, Entry};
use deskpet_host::pet::PetView;
use petkit::engine::{Command, Input, Reminder};
use petkit::look::pt;
use std::ffi::{c_char, c_void, CStr, CString};
use std::path::{Path, PathBuf};

/// Draws one line of text white on black and hands the coverage to [`dp_mask_set`] with `sink`.
/// `font`: 0 symbol glyphs (bold), 1 UI text (semibold). `px`: font size in device pixels.
pub type TextFn = extern "C" fn(text: *const u8, len: usize, font: u8, px: i32, sink: *mut c_void);
/// GETs `url`; on a 200 response of at most `limit` bytes, hands the body to [`dp_bytes_set`] with
/// `sink` and returns 1, otherwise returns 0. Called on a background thread.
pub type GetFn = extern "C" fn(url: *const c_char, limit: usize, sink: *mut c_void) -> i32;

/// Where the pet is, for the app's hit testing and saving its home.
#[repr(C)]
#[derive(Default)]
pub struct DPState {
    pub pet_x: f64,
    pub min_pet_x: f64,
    pub max_pet_x: f64,
    /// The pet's clickable rectangle in pen points (y up from the floor).
    pub rect_x: f64,
    pub rect_y: f64,
    pub rect_w: f64,
    pub rect_h: f64,
    pub dragging: i32,
    /// The pet was dragged to a new home: the app saves it, then calls [`dp_home_saved`].
    pub home_moved: i32,
    /// Free to show a reminder (awake, not dragged, not talking).
    pub can_remind: i32,
}

struct AppText(TextFn);

impl TextRaster for AppText {
    fn render(&mut self, text: &str, font: Font, px: i32) -> Mask {
        let mut m = Mask { w: 0, h: 0, cov: vec![] };
        (self.0)(text.as_ptr(), text.len(), font as u8, px, &mut m as *mut Mask as *mut c_void);
        m
    }
}

pub struct Host {
    folder: PathBuf,
    extra: Option<PathBuf>,
    pets: Vec<Entry>,
    current: usize,
    view: Option<PetView>,
    text: TextCache,
    /// The last string handed out.
    out: CString,
    /// The last install or remove result: the pet's id, or what went wrong.
    message: String,
}

unsafe fn host<'a>(h: *mut Host) -> &'a mut Host {
    &mut *h
}

unsafe fn string(s: *const c_char) -> Option<String> {
    (!s.is_null()).then(|| CStr::from_ptr(s).to_string_lossy().into_owned())
}

impl Host {
    fn give(&mut self, s: String) -> *const c_char {
        self.out = CString::new(s.replace('\0', "")).unwrap_or_default();
        self.out.as_ptr()
    }

    fn command(&mut self, c: Command) -> bool {
        self.view.as_mut().is_some_and(|v| v.command(c))
    }
}

/// A JSON string literal.
fn js(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for ch in s.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

fn js_opt(s: &Option<String>) -> String {
    s.as_deref().map_or("null".into(), js)
}

// MARK: Making and freeing

/// Reads the pets: the built-in ones, those installed in `folder`, and those in `extra` (may be
/// NULL; the `--pets` folder). No pet runs until [`dp_start`].
#[no_mangle]
pub unsafe extern "C" fn dp_new(folder: *const c_char, extra: *const c_char, text: TextFn) -> *mut Host {
    let folder = PathBuf::from(string(folder).unwrap_or_default());
    let extra = string(extra).map(PathBuf::from);
    let pets = catalog::catalog(Some(&folder), extra.as_deref());
    let h =
        Host { folder, extra, pets, current: 0, view: None, text: TextCache::new(Box::new(AppText(text))), out: CString::default(), message: String::new() };
    Box::into_raw(Box::new(h))
}

#[no_mangle]
pub unsafe extern "C" fn dp_free(h: *mut Host) {
    if !h.is_null() {
        drop(Box::from_raw(h));
    }
}

/// Hands text coverage back from a [`TextFn`]: `w`×`h` bytes, rows `stride` bytes apart, top row first.
#[no_mangle]
pub unsafe extern "C" fn dp_mask_set(sink: *mut c_void, w: i32, h: i32, cov: *const u8, stride: usize) {
    if sink.is_null() || cov.is_null() || w <= 0 || h <= 0 || stride < w as usize {
        return;
    }
    let m = &mut *(sink as *mut Mask);
    let mut bytes = Vec::with_capacity((w * h) as usize);
    for y in 0..h as usize {
        bytes.extend_from_slice(std::slice::from_raw_parts(cov.add(y * stride), w as usize));
    }
    *m = Mask { w, h, cov: bytes };
}

// MARK: The pet list

/// Every pet as JSON: `[{"id", "name", "shortName", "category", "version", "author", "installed", "tallest"}]`,
/// built-in pets first, grouped by category ("critter", "hero", "legend").
#[no_mangle]
pub unsafe extern "C" fn dp_catalog_json(h: *mut Host) -> *const c_char {
    let h = host(h);
    let items: Vec<String> = h
        .pets
        .iter()
        .map(|e| {
            let a = &e.about;
            format!(
                "{{\"id\":{},\"name\":{},\"shortName\":{},\"category\":{},\"version\":{},\"author\":{},\"installed\":{},\"tallest\":{}}}",
                js(&a.id),
                js(&a.name),
                js(&a.short_name),
                js(&format!("{:?}", a.category).to_lowercase()),
                js(&a.version),
                js(&a.author),
                e.file.is_some(),
                a.tallest
            )
        })
        .collect();
    let json = format!("[{}]", items.join(","));
    h.give(json)
}

/// The pet at `index` in its own colours, cropped and centred in a `size`×`size` square: 0xAARRGGBB
/// words, top row first, into `out` (room for `size * size`).
#[no_mangle]
pub unsafe extern "C" fn dp_icon(h: *mut Host, index: u32, size: i32, out: *mut u32) {
    let h = host(h);
    let Some(e) = h.pets.get(index as usize) else { return };
    if out.is_null() || size <= 0 {
        return;
    }
    let px = catalog::icon_pixels(&e.about, size);
    std::ptr::copy_nonoverlapping(px.as_ptr(), out, px.len());
}

/// Re-reads the pet list after pets were added, removed or updated. Returns the index of the pet
/// the app should switch to: `show` (an id, may be NULL) if given, the pet on screen if its file
/// changed version, or the first pet if it's gone; -1 if the pet on screen can stay.
#[no_mangle]
pub unsafe extern "C" fn dp_reload(h: *mut Host, show: *const c_char) -> i32 {
    let h = host(h);
    let show = string(show);
    let (id, version) = h.view.as_ref().map(|v| (v.about.id.clone(), v.about.version.clone())).unwrap_or_default();
    h.pets = catalog::catalog(Some(&h.folder), h.extra.as_deref());
    let wanted = show.clone().unwrap_or(id);
    match h.pets.iter().position(|p| p.about.id == wanted) {
        Some(i) if show.is_some() || h.pets[i].about.version != version => i as i32,
        Some(i) => {
            h.current = i;
            -1
        }
        None => 0,
    }
}

/// Where the pet website is (`DESKPET_SITE` overrides it).
#[no_mangle]
pub unsafe extern "C" fn dp_site(h: *mut Host) -> *const c_char {
    let h = host(h);
    h.give(catalog::site())
}

// MARK: Installing, removing and updating

/// Checks a pet file and copies it into the pets folder. Returns 1 and the pet's id in
/// [`dp_message`], or 0 and what went wrong. Call [`dp_reload`] with that id to show it.
#[no_mangle]
pub unsafe extern "C" fn dp_install(h: *mut Host, path: *const c_char) -> i32 {
    let h = host(h);
    let path = PathBuf::from(string(path).unwrap_or_default());
    let r = catalog::install(&h.folder, &path);
    let ok = r.is_ok();
    h.message = r.unwrap_or_else(|e| e);
    ok as i32
}

/// Deletes an installed pet's file. Returns 1, or 0 with what went wrong in [`dp_message`].
/// Call [`dp_reload`] afterwards.
#[no_mangle]
pub unsafe extern "C" fn dp_remove(h: *mut Host, index: u32) -> i32 {
    let h = host(h);
    let Some(file) = h.pets.get(index as usize).and_then(|e| e.file.clone()) else {
        h.message = "it came with the app".into();
        return 0;
    };
    match catalog::remove(&file) {
        Ok(()) => 1,
        Err(e) => {
            h.message = e;
            0
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn dp_message(h: *mut Host) -> *const c_char {
    let h = host(h);
    let m = h.message.clone();
    h.give(m)
}

/// Hands a downloaded body back from a [`GetFn`].
#[no_mangle]
pub unsafe extern "C" fn dp_bytes_set(sink: *mut c_void, bytes: *const u8, len: usize) {
    if sink.is_null() || (bytes.is_null() && len > 0) {
        return;
    }
    let v = &mut *(sink as *mut Vec<u8>);
    v.clear();
    if len > 0 {
        v.extend_from_slice(std::slice::from_raw_parts(bytes, len));
    }
}

/// Brings the pets installed in `folder` up to date from the website, with the app's HTTP. Blocks;
/// call it on a background thread (it touches no [`Host`]). Returns how many pets it updated; then
/// call [`dp_reload`] on the main thread.
#[no_mangle]
pub unsafe extern "C" fn dp_update(folder: *const c_char, get: GetFn) -> i32 {
    let Some(folder) = string(folder) else { return 0 };
    let fetch = |url: &str, limit: usize| -> Result<Vec<u8>, String> {
        let url = CString::new(url).map_err(|_| "bad address".to_string())?;
        let mut body: Vec<u8> = Vec::new();
        if get(url.as_ptr(), limit, &mut body as *mut Vec<u8> as *mut c_void) == 1 && body.len() <= limit {
            Ok(body)
        } else {
            Err("download failed".into())
        }
    };
    catalog::update(Path::new(&folder), &catalog::site(), fetch).len() as i32
}

// MARK: The pet on screen

/// Starts the pet at `index` (the first pet if that one won't start). Returns the index started.
/// `now` is seconds on the app's clock (the same clock as [`dp_tick`]).
#[no_mangle]
pub unsafe extern "C" fn dp_start(h: *mut Host, index: u32, now: f64, seed: u64) -> i32 {
    let h = host(h);
    let mut i = (index as usize).min(h.pets.len().saturating_sub(1));
    let mut view = PetView::new(&h.pets[i].bytes, now, seed);
    if view.is_err() && i != 0 {
        i = 0;
        view = PetView::new(&h.pets[0].bytes, now, seed);
    }
    h.view = view.ok();
    h.current = i;
    i as i32
}

/// The running pet as JSON: `{"id", "name", "shortName", "version", "sweeps", "hasPastime",
/// "clickTrick", "rightClickTrick", "tricks": [{"id", "title", "seconds"}]}` (null if none runs).
#[no_mangle]
pub unsafe extern "C" fn dp_pet_json(h: *mut Host) -> *const c_char {
    let h = host(h);
    let json = match &h.view {
        None => "null".to_string(),
        Some(v) => {
            let a = &v.about;
            let tricks: Vec<String> =
                a.tricks.iter().map(|t| format!("{{\"id\":{},\"title\":{},\"seconds\":{}}}", js(&t.id), js(&t.title), t.seconds)).collect();
            format!(
                "{{\"index\":{},\"id\":{},\"name\":{},\"shortName\":{},\"version\":{},\"sweeps\":{},\"hasPastime\":{},\"clickTrick\":{},\"rightClickTrick\":{},\"tricks\":[{}]}}",
                h.current,
                js(&a.id),
                js(&a.name),
                js(&a.short_name),
                js(&a.version),
                a.sweeps,
                a.has_pastime,
                js_opt(&a.click_trick),
                js_opt(&a.right_click_trick),
                tricks.join(",")
            )
        }
    };
    h.give(json)
}

/// One frame: seconds since the last frame and on the app's clock, seconds since any input and since
/// the last key press, and the pointer in pen points (y up from the floor). Returns 1 if the pet's
/// module just broke (it panicked or ran too long): the app then switches to the first pet.
#[no_mangle]
pub unsafe extern "C" fn dp_tick(h: *mut Host, dt: f64, now: f64, idle: f64, key_idle: f64, mouse_x: f64, mouse_y: f64) -> i32 {
    let h = host(h);
    let Some(v) = h.view.as_mut() else { return 0 };
    v.tick(Input { dt, now, idle, key_idle, mouse: pt(mouse_x, mouse_y) });
    v.failed.is_some() as i32
}

/// Why the pet's module broke, or NULL.
#[no_mangle]
pub unsafe extern "C" fn dp_failed(h: *mut Host) -> *const c_char {
    let h = host(h);
    match h.view.as_ref().and_then(|v| v.failed.clone()) {
        Some(e) => h.give(e),
        None => std::ptr::null(),
    }
}

#[no_mangle]
pub unsafe extern "C" fn dp_state(h: *mut Host, out: *mut DPState) {
    let h = host(h);
    let (Some(v), false) = (h.view.as_ref(), out.is_null()) else { return };
    let r = &v.r;
    let (x, y, w, ht) = r.pet_rect;
    *out = DPState {
        pet_x: r.pet_x,
        min_pet_x: r.min_pet_x,
        max_pet_x: r.max_pet_x,
        rect_x: x,
        rect_y: y,
        rect_w: w,
        rect_h: ht,
        dragging: r.dragging as i32,
        home_moved: r.home_moved as i32,
        can_remind: r.can_remind as i32,
    };
}

/// Draws the pet into `pixels`: `w`×`h` premultiplied 0xAARRGGBB words (BGRA bytes), top row first,
/// with `scale` device pixels per point. The buffer is cleared first.
#[no_mangle]
pub unsafe extern "C" fn dp_draw(h: *mut Host, pixels: *mut u32, w: i32, ht: i32, scale: f64) {
    let h = host(h);
    if pixels.is_null() || w <= 0 || ht <= 0 {
        return;
    }
    let buf = std::slice::from_raw_parts_mut(pixels, (w * ht) as usize);
    let mut c = Canvas { buf, w, h: ht, s: scale, text: &mut h.text };
    c.clear();
    if let Some(v) = &h.view {
        v.draw(&mut c);
    }
}

// MARK: Commands

/// The pen's size in points, and the roam range picked in the menu.
#[no_mangle]
pub unsafe extern "C" fn dp_pen(h: *mut Host, width: f64, height: f64, range: f64) {
    host(h).command(Command::Pen { width, height, range });
}

/// Puts the pet at this x in the pen (kept inside it).
#[no_mangle]
pub unsafe extern "C" fn dp_place_at(h: *mut Host, x: f64) {
    host(h).command(Command::PlaceAt(x));
}

#[no_mangle]
pub unsafe extern "C" fn dp_say(h: *mut Host, text: *const c_char, seconds: f64) {
    let text = string(text).unwrap_or_default();
    host(h).command(Command::Say(text, seconds));
}

#[no_mangle]
pub unsafe extern "C" fn dp_show_pen(h: *mut Host, seconds: f64) {
    host(h).command(Command::ShowPen(seconds));
}

/// Plays the running pet's trick number `index` (as listed in [`dp_pet_json`]).
#[no_mangle]
pub unsafe extern "C" fn dp_trick(h: *mut Host, index: u32) {
    host(h).command(Command::Trick(index as usize));
}

#[no_mangle]
pub unsafe extern "C" fn dp_surprise(h: *mut Host) {
    host(h).command(Command::SurpriseTrick);
}

/// The pet was just picked in the menu: it says hello.
#[no_mangle]
pub unsafe extern "C" fn dp_hello(h: *mut Host) {
    host(h).command(Command::Hello);
}

/// 0 water, 1 lunch, 2 dinner, 3 stretch.
#[no_mangle]
pub unsafe extern "C" fn dp_remind(h: *mut Host, which: u32) {
    let r = match which {
        0 => Reminder::Water,
        1 => Reminder::Lunch,
        2 => Reminder::Dinner,
        _ => Reminder::Stretch,
    };
    host(h).command(Command::Remind(r));
}

#[no_mangle]
pub unsafe extern "C" fn dp_begin_drag(h: *mut Host) {
    host(h).command(Command::BeginDrag);
}

/// The mouse button went up: `moved` != 0 for a drag, otherwise it was a click.
#[no_mangle]
pub unsafe extern "C" fn dp_end_drag(h: *mut Host, moved: i32) {
    host(h).command(Command::EndDrag { moved: moved != 0 });
}

/// A right-click on the pet. Returns 1 if the pet did a trick with it, 0 if the app should show its menu.
#[no_mangle]
pub unsafe extern "C" fn dp_right_click(h: *mut Host) -> i32 {
    host(h).command(Command::RightClick) as i32
}

/// Stops walking or sweeping (the pen just changed size).
#[no_mangle]
pub unsafe extern "C" fn dp_stop_roaming(h: *mut Host) {
    host(h).command(Command::StopRoaming);
}

#[no_mangle]
pub unsafe extern "C" fn dp_home_saved(h: *mut Host) {
    host(h).command(Command::HomeSaved);
}

/// The `--demo` steps: 0 walk to the far side, 1 type, 2 pastime, 3 sweep (for `seconds`).
#[no_mangle]
pub unsafe extern "C" fn dp_demo(h: *mut Host, kind: u32, seconds: f64) {
    let c = match kind {
        0 => Command::DemoWalk,
        1 => Command::DemoTyping(seconds),
        2 => Command::DemoPastime(seconds),
        _ => Command::DemoSweep(seconds),
    };
    host(h).command(c);
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Stands in for the Mac's text drawing: a solid box a little wider than the text.
    extern "C" fn boxes(_text: *const u8, len: usize, _font: u8, px: i32, sink: *mut c_void) {
        let (w, h) = (px * len as i32 / 2 + 1, px + 2);
        let cov = vec![255u8; (w * h) as usize];
        unsafe { dp_mask_set(sink, w, h, cov.as_ptr(), w as usize) };
    }

    extern "C" fn offline(_url: *const c_char, _limit: usize, _sink: *mut c_void) -> i32 {
        0
    }

    fn text(p: *const c_char) -> String {
        unsafe { CStr::from_ptr(p).to_string_lossy().into_owned() }
    }

    #[test]
    fn runs_and_draws_a_pet() {
        let folder = std::env::temp_dir().join(format!("deskpet-mac-test-{}", std::process::id()));
        let dir = CString::new(folder.to_string_lossy().as_ref()).unwrap();
        unsafe {
            let h = dp_new(dir.as_ptr(), std::ptr::null(), boxes);
            let list = text(dp_catalog_json(h));
            assert!(list.starts_with("[{\"id\":\"mochi\""), "{list}");
            assert!(list.contains("\"category\":\"legend\""));

            assert_eq!(dp_start(h, 1, 0.0, 7), 1);
            let pet = text(dp_pet_json(h));
            assert!(pet.contains("\"id\":\"ninja\"") && pet.contains("\"tricks\":[{"), "{pet}");

            let (w, ht, s) = (400.0, 170.0, 2.0);
            dp_pen(h, w, ht, 80.0);
            dp_place_at(h, 150.0);
            let hi = CString::new("hi!").unwrap();
            dp_say(h, hi.as_ptr(), 2.0);
            for i in 0..60 {
                assert_eq!(dp_tick(h, 1.0 / 30.0, i as f64 / 30.0, 0.0, 99.0, 0.0, 0.0), 0);
            }
            let mut state = DPState::default();
            dp_state(h, &mut state);
            assert!(state.rect_w > 0.0 && state.pet_x >= state.min_pet_x);

            let (pw, ph) = ((w * s) as i32, (ht * s) as i32);
            let mut px = vec![0u32; (pw * ph) as usize];
            dp_draw(h, px.as_mut_ptr(), pw, ph, s);
            assert!(px.iter().filter(|&&p| p >> 24 == 255).count() > 500, "the pet and its bubble are drawn");

            let mut icon = vec![0u32; 36 * 36];
            dp_icon(h, 0, 36, icon.as_mut_ptr());
            assert!(icon.iter().any(|&p| p != 0));

            dp_trick(h, 0);
            for i in 0..90 {
                dp_tick(h, 1.0 / 30.0, 2.0 + i as f64 / 30.0, 0.0, 99.0, 0.0, 0.0);
            }
            assert!(dp_failed(h).is_null());

            let nothing = CString::new(folder.join("nope.deskpet").to_string_lossy().as_ref()).unwrap();
            assert_eq!(dp_install(h, nothing.as_ptr()), 0);
            assert!(text(dp_message(h)).starts_with("couldn't read it"));
            assert_eq!(dp_reload(h, std::ptr::null()), -1, "the pet on screen stays");
            assert_eq!(dp_update(dir.as_ptr(), offline), 0);
            dp_free(h);
        }
    }

    #[test]
    fn escapes_json() {
        assert_eq!(js("a\"b\\c\n"), "\"a\\\"b\\\\c\\n\"");
    }
}
