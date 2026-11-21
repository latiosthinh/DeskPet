// DeskPet for Windows — a tiny pixel pet (Mochi, Ember, Pip, Puddle or Sprout) that lives on your taskbar.
//
// It wanders back and forth inside a limited "pen" above the bottom-right of the taskbar.
// When you type, it gets busy too. It reminds you to drink water, eat your main meals and
// stretch, and falls asleep when you're away.
//
// Drag the pet to move its home. Click it to pet it (or dismiss a reminder). Right-click it,
// or click its tray icon, for the menu.

#![windows_subsystem = "windows"]

mod canvas;
mod module;
mod net;
mod pet;
mod pets;
mod settings;
mod sys;

use canvas::{Canvas, TextCache};
use pet::PetView;
use petkit::art::{Color, PET_SIZE, PROP_ROOM};
use petkit::engine::{About, Command, Input, Reminder};
use petkit::look::pt;
use petkit::random::rand;
use pets::Entry;
use settings::LoginState;
use std::cell::{Cell, RefCell};
use std::collections::VecDeque;
use std::path::{Path, PathBuf};
use std::ptr::{null, null_mut};
use std::sync::OnceLock;
use std::time::Instant;
use sys::{wide, Dib, Menu, MenuItem};
use windows_sys::Win32::Foundation::*;
use windows_sys::Win32::Graphics::Gdi::HBITMAP;
use windows_sys::Win32::UI::WindowsAndMessaging::*;

const RANGE_OPTIONS: [(f64, &str); 5] =
    [(0.0, "Stay Put"), (80.0, "Tiny  (±80 px)"), (160.0, "Small  (±160 px)"), (300.0, "Medium  (±300 px)"), (500.0, "Large  (±500 px)")];
const CORNER_MARGIN: f64 = 4.0;
/// Each interval is re-picked at random within its range, so reminders don't feel mechanical.
const BREAK: (f64, f64) = (40.0 * 60.0, 60.0 * 60.0);
const WATER: (f64, f64) = (20.0 * 60.0, 30.0 * 60.0);
const BREAK_RESET_IDLE: f64 = 5.0 * 60.0;
/// Main meals as (reminder, name, minutes after midnight). Each can fire once a day, up to 2 hours after its time.
const MEALS: [(Reminder, &str, u32); 2] = [(Reminder::Lunch, "lunch", 12 * 60), (Reminder::Dinner, "dinner", 18 * 60 + 30)];

const FRAME_MS: u32 = 33;
const HIDDEN_MS: u32 = 250;

// Messages to the owner window.
const WM_PETS_UPDATED: u32 = WM_APP + 2;
const WM_INSTALL: u32 = WM_APP + 3;

// Menu command ids.
const CMD_SHOW: u32 = 1;
const CMD_SURPRISE: u32 = 3;
const CMD_SHOW_AREA: u32 = 4;
const CMD_CORNER: u32 = 5;
const CMD_WATER: u32 = 6;
const CMD_MEALS: u32 = 7;
const CMD_BREAKS: u32 = 8;
const CMD_LOGIN: u32 = 9;
const CMD_QUIT: u32 = 10;
const CMD_GET_PETS: u32 = 11;
const CMD_REMOVE: u32 = 12;
const CMD_TRICK: u32 = 200;
const CMD_RANGE: u32 = 300;
const CMD_PET: u32 = 1000;

fn uptime() -> f64 {
    static START: OnceLock<Instant> = OnceLock::new();
    START.get_or_init(Instant::now).elapsed().as_secs_f64()
}

/// What a menu command needs done after the app is no longer borrowed (anything with its own message loop).
enum After {
    Nothing,
    LoginFailed,
    /// Ask before removing an installed pet: its file and name.
    Remove(PathBuf, String),
    Quit,
}

struct App {
    pets: Vec<Entry>,
    /// The folder given with `--pets`, so the list can be re-read.
    extra: Option<PathBuf>,
    view: PetView,
    pet_hwnd: HWND,
    owner: HWND,

    // The pen window, in physical pixels.
    x: i32,
    y: i32,
    w: i32,
    h: i32,
    scale: f64,
    min_x: i32,
    max_x: i32,

    range: f64,
    home_x: Option<i32>, // centre of the pen, physical screen pixels
    breaks_on: bool,
    water_on: bool,
    meals_on: bool,
    visible: bool,
    fullscreen: bool,
    shown: bool,
    menu_open: bool,
    click_through: bool,
    last_work: RECT,
    last_dpi: u32,

    last_tick: f64,
    streak_start: f64,
    last_key_down: f64,
    last_input_stamp: u32,
    last_slow_check: f64,
    break_due: f64,
    active_since_water: f64,
    water_due: f64,
    last_meal_check: f64,

    /// `--demo` and friends: commands to send, each followed by a wait in seconds.
    demo: VecDeque<(f64, Command)>,
    next_demo: f64,

    tray_icon: HICON,

    mouse_down: bool,
    drag_moved: bool,
    drag_start_mouse_x: i32,
    drag_start_x: i32,

    dib: Option<Dib>,
    text: TextCache,
    last_frame: Vec<u32>,
    last_pos: (i32, i32, i32, i32),
    timer_ms: u32,
}

thread_local! {
    static APP: RefCell<Option<App>> = const { RefCell::new(None) };
    static TASKBAR_CREATED: Cell<u32> = const { Cell::new(0) };
    static OWNER: Cell<HWND> = const { Cell::new(null_mut()) };
    /// Pet files waiting to be installed (dropped, double-clicked or handed over by a second DeskPet).
    static TO_INSTALL: RefCell<VecDeque<PathBuf>> = const { RefCell::new(VecDeque::new()) };
}

/// Runs `f` with the app, unless it isn't made yet or is already in use further up the stack.
fn with_app<T>(f: impl FnOnce(&mut App) -> T) -> Option<T> {
    APP.with(|a| match a.try_borrow_mut() {
        Ok(mut guard) => guard.as_mut().map(f),
        Err(_) => None,
    })
}

/// The pet in its own colours, cropped to its drawn pixels and centred in a `size`×`size` square (ARGB).
/// Uses a whole number of device pixels per art pixel when that fills the square well, so it stays crisp.
fn pet_pixels(pet: &About, size: i32) -> Vec<u32> {
    let all = &pet.icon;
    let width = all[0].len();
    let used: Vec<usize> = (0..all.len()).filter(|&r| all[r].iter().any(|c| c.is_some())).collect();
    let rows = &all[used[0]..=*used.last().unwrap()];
    let cols_used: Vec<usize> = (0..width).filter(|&c| rows.iter().any(|r| r[c].is_some())).collect();
    let c0 = cols_used[0];
    let cols = cols_used.last().unwrap() - c0 + 1;

    let n = size as f64;
    let fit = n / cols.max(rows.len()) as f64;
    let k = if fit.floor() >= 1.0 && fit.floor() / fit >= 0.8 { fit.floor() } else { fit };
    let ox = ((n - cols as f64 * k) / 2.0).round();
    let oy = ((n - rows.len() as f64 * k) / 2.0).round();

    let mut px = vec![0u32; (size * size) as usize];
    for (r, row) in rows.iter().enumerate() {
        for c in 0..cols {
            let Some(color) = row[c0 + c] else { continue };
            let argb = (color.a as u32) << 24 | (color.r as u32) << 16 | (color.g as u32) << 8 | color.b as u32;
            let (x0, x1) = ((ox + c as f64 * k).round() as i32, (ox + (c + 1) as f64 * k).round() as i32);
            let (y0, y1) = ((oy + r as f64 * k).round() as i32, (oy + (r + 1) as f64 * k).round() as i32);
            for y in y0.max(0)..y1.min(size) {
                for x in x0.max(0)..x1.min(size) {
                    px[(y * size + x) as usize] = argb;
                }
            }
        }
    }
    px
}

fn pet_icon(pet: &About, size: i32) -> HICON {
    sys::icon_from_pixels(&pet_pixels(pet, size), size)
}

/// A fresh seed for a pet's random numbers.
fn seed() -> u64 {
    std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_or(1, |d| d.as_nanos() as u64)
}

/// Starts pet `i`, or the first built-in pet if that one won't start.
fn start_pet(pets: &[Entry], i: usize) -> PetView {
    PetView::new(&pets[i].bytes, uptime(), seed()).or_else(|_| PetView::new(&pets[0].bytes, uptime(), seed())).expect("the built-in pets start")
}

impl App {
    /// `wanted`: a pet id to start with instead of the saved one (`--pet`).
    fn new(extra: Option<PathBuf>, wanted: Option<&str>, pet_hwnd: HWND, owner: HWND) -> App {
        let now = uptime();
        let pets = pets::catalog(extra.as_deref());
        let saved_id = settings::get_string("pet").unwrap_or_else(|| "mochi".into());
        let id = wanted.unwrap_or(&saved_id);
        let pet = pets.iter().position(|p| p.about.id.eq_ignore_ascii_case(id)).unwrap_or(0);
        settings::refresh_login_path();

        App {
            view: start_pet(&pets, pet),
            pets,
            extra,
            pet_hwnd,
            owner,
            x: 0,
            y: 0,
            w: 1,
            h: 1,
            scale: 1.0,
            min_x: 0,
            max_x: 0,
            range: settings::get_f64("range").unwrap_or(80.0),
            home_x: settings::get_f64("homeX").map(|v| v as i32),
            breaks_on: settings::get_bool("breaks", true),
            water_on: settings::get_bool("water", true),
            meals_on: settings::get_bool("meals", true),
            visible: settings::get_bool("visible", true),
            fullscreen: false,
            shown: false,
            menu_open: false,
            click_through: true,
            last_work: RECT { left: 0, top: 0, right: 0, bottom: 0 },
            last_dpi: 96,
            last_tick: now,
            streak_start: now,
            last_key_down: -1000.0,
            last_input_stamp: 0,
            last_slow_check: 0.0,
            break_due: rand(BREAK.0, BREAK.1),
            active_since_water: 0.0,
            water_due: rand(WATER.0, WATER.1),
            last_meal_check: 0.0,
            demo: VecDeque::new(),
            next_demo: 0.0,
            tray_icon: null_mut(),
            mouse_down: false,
            drag_moved: false,
            drag_start_mouse_x: 0,
            drag_start_x: 0,
            dib: None,
            text: TextCache::new(),
            last_frame: Vec::new(),
            last_pos: (0, 0, 0, 0),
            timer_ms: 0,
        }
    }

    // MARK: Layout

    /// Sizes the pen window to the roam range around home, standing on top of the taskbar.
    /// With no saved home, the pen sits in the bottom-right corner, above the clock.
    fn layout(&mut self) {
        let screen = sys::primary_screen();
        let wa = screen.work;
        self.last_work = wa;
        self.last_dpi = screen.dpi;
        let s = screen.dpi as f64 / 96.0;
        let width = ((wa.right - wa.left) as f64 / s).min(2.0 * self.range + PET_SIZE + 2.0 * PROP_ROOM);
        let tallest = self.pets.iter().map(|p| p.about.tallest).fold(PET_SIZE, f64::max);
        let height = tallest + 70.0;
        let (w, h) = ((width * s).round() as i32, (height * s).round() as i32);
        let home = self.home_x.unwrap_or(wa.right - (CORNER_MARGIN * s).round() as i32 - w / 2);

        self.scale = s;
        self.x = (home - w / 2).max(wa.left).min(wa.right - w);
        self.y = wa.bottom - h;
        self.w = w;
        self.h = h;
        self.min_x = wa.left;
        self.max_x = wa.right;
        self.view.command(Command::Pen { width: w as f64 / s, height: h as f64 / s, range: self.range });
    }

    fn pet_screen_x(&self) -> f64 {
        self.x as f64 + self.view.r.pet_x * self.scale
    }

    fn keep_pet(&mut self, screen_x: f64) {
        self.view.command(Command::PlaceAt((screen_x - self.x as f64) / self.scale));
    }

    fn screens_changed(&mut self) {
        if self.view.r.dragging {
            return;
        }
        let abs = self.pet_screen_x();
        let old_dpi = self.last_dpi;
        self.layout();
        self.keep_pet(abs);
        if self.last_dpi != old_dpi {
            self.update_tray_icon();
        }
        if self.shown {
            self.render();
        }
    }

    fn save_home_if_moved(&mut self) {
        if self.view.r.home_moved {
            self.view.command(Command::HomeSaved);
            let home = self.x + self.w / 2;
            self.home_x = Some(home);
            settings::set_f64("homeX", home as f64);
        }
    }

    // MARK: Frame loop

    fn set_timer(&mut self, ms: u32) {
        if self.timer_ms != ms {
            self.timer_ms = ms;
            unsafe { SetTimer(self.owner, 1, ms, None) };
        }
    }

    fn set_shown(&mut self, show: bool) {
        if show == self.shown {
            return;
        }
        self.shown = show;
        if show {
            self.last_frame.clear();
            self.render(); // put the first frame in place before showing
        }
        sys::show(self.pet_hwnd, show);
        self.set_timer(if show { FRAME_MS } else { HIDDEN_MS });
    }

    fn tick(&mut self) {
        let now = uptime();
        let dt = (now - self.last_tick).min(0.1);
        self.last_tick = now;

        // Keys are only polled when the system has seen new input since the last frame.
        let (input_idle, stamp) = sys::last_input();
        if stamp != self.last_input_stamp {
            self.last_input_stamp = stamp;
            if sys::typing_key_down() {
                self.last_key_down = now;
            }
        }
        let key_idle = now - self.last_key_down;
        let idle = input_idle.min(key_idle);

        if now - self.last_slow_check > 1.0 {
            self.last_slow_check = now;
            self.fullscreen = sys::fullscreen_app_in_front();
            let screen = sys::primary_screen();
            let (a, b) = (screen.work, self.last_work);
            if (a.left, a.top, a.right, a.bottom) != (b.left, b.top, b.right, b.bottom) || screen.dpi != self.last_dpi {
                self.screens_changed();
            }
            if self.shown {
                sys::keep_on_top(self.pet_hwnd);
            }
        }

        self.check_reminders(now, dt, idle);

        let show = self.visible && !self.fullscreen;
        self.set_shown(show);
        if !show {
            return;
        }

        if now >= self.next_demo {
            if let Some((seconds, step)) = self.demo.pop_front() {
                self.view.command(step);
                self.next_demo = now + seconds;
            }
        }

        let (cx, cy) = sys::cursor();
        let demo_running = !self.demo.is_empty() || now < self.next_demo; // the demo ignores real typing and idleness
        let mouse = pt((cx - self.x) as f64 / self.scale, (self.y + self.h - cy) as f64 / self.scale);
        let (idle, key_idle) = if demo_running { (0.0, 99.0) } else { (idle, key_idle) };
        self.view.tick(Input { dt, now, idle, key_idle, mouse });
        if self.view.failed.is_some() {
            // The pet's module broke: the first built-in pet takes over.
            let name = self.view.about.short_name.clone();
            self.switch_to(0);
            self.view.say(&format!("{name} ran into a problem"), 3.0);
        }
        self.save_home_if_moved();

        // Only the pet itself catches clicks; the rest of the pen is click-through.
        let through = !(self.view.r.dragging || self.pet_rect_contains(cx, cy));
        if through != self.click_through {
            self.click_through = through;
            sys::set_click_through(self.pet_hwnd, through);
        }
        self.render();
    }

    /// Whether a screen point is on the pet's clickable rectangle.
    fn pet_rect_contains(&self, px: i32, py: i32) -> bool {
        let (rx, _, rw, rh) = self.view.r.pet_rect;
        let s = self.scale;
        let left = self.x + (rx * s).round() as i32;
        let right = self.x + ((rx + rw) * s).round() as i32;
        let bottom = self.y + self.h;
        let top = bottom - (rh * s).round() as i32;
        px >= left && px < right && py >= top && py < bottom
    }

    /// Draws a frame and pushes it to the screen, unless nothing changed since the last one.
    fn render(&mut self) {
        let (w, h) = (self.w, self.h);
        if self.dib.as_ref().map_or(true, |d| d.w != w || d.h != h) {
            self.dib = Some(Dib::new(w, h));
            self.last_frame.clear();
        }
        let dib = self.dib.as_mut().unwrap();
        {
            let mut c = Canvas { buf: dib.pixels(), w, h, s: self.scale, text: &mut self.text };
            c.clear();
            // A nearly invisible fill so the whole sprite area takes clicks, not just its painted pixels.
            let (rx, ry, rw, rh) = self.view.r.pet_rect;
            c.fill_rect(rx, ry, rw, rh, Color::new(0, 0, 0, 1));
            self.view.draw(&mut c);
        }
        let pos = (self.x, self.y, w, h);
        let pixels = dib.pixels();
        if pos != self.last_pos || self.last_frame.len() != pixels.len() {
            self.last_frame.clear();
            self.last_frame.extend_from_slice(pixels);
            self.last_pos = pos;
            sys::update_layered(self.pet_hwnd, dib, self.x, self.y, None);
            return;
        }
        // Same place and size: send only the rectangle that changed, or nothing.
        let (mut x0, mut y0, mut x1, mut y1) = (w, h, 0, 0);
        for y in 0..h {
            let row = (y * w) as usize..((y + 1) * w) as usize;
            let (new, old) = (&pixels[row.clone()], &mut self.last_frame[row]);
            if new == &old[..] {
                continue;
            }
            let first = new.iter().zip(old.iter()).position(|(a, b)| a != b).unwrap() as i32;
            let last = w - 1 - new.iter().rev().zip(old.iter().rev()).position(|(a, b)| a != b).unwrap() as i32;
            (x0, x1, y0, y1) = (x0.min(first), x1.max(last + 1), y0.min(y), y + 1);
            old.copy_from_slice(new);
        }
        if x1 > x0 {
            let dirty = RECT { left: x0, top: y0, right: x1, bottom: y1 };
            sys::update_layered(self.pet_hwnd, dib, self.x, self.y, Some(dirty));
        }
    }

    // MARK: Reminders

    fn check_reminders(&mut self, now: f64, dt: f64, idle: f64) {
        let at_desk = idle < 60.0;
        if at_desk {
            self.active_since_water += dt;
        }
        if idle > BREAK_RESET_IDLE {
            self.streak_start = now; // a 5-minute pause counts as a break
        }
        if !(self.visible && !self.fullscreen && at_desk && self.view.r.can_remind) {
            return;
        }
        if self.breaks_on && now - self.streak_start > self.break_due {
            self.streak_start = now;
            self.break_due = rand(BREAK.0, BREAK.1);
            self.view.command(Command::Remind(Reminder::Stretch));
        } else if self.water_on && self.active_since_water > self.water_due {
            self.active_since_water = 0.0;
            self.water_due = rand(WATER.0, WATER.1);
            self.view.command(Command::Remind(Reminder::Water));
        } else if self.meals_on && now - self.last_meal_check > 30.0 {
            self.last_meal_check = now;
            self.check_meals();
        }
    }

    fn check_meals(&mut self) {
        let ((y, m, d), minutes) = sys::local_time();
        let today = format!("{y:04}-{m:02}-{d:02}");
        for (reminder, key, start) in MEALS {
            let name = format!("lastMeal.{key}");
            if minutes >= start && minutes < start + 120 && settings::get_string(&name).as_deref() != Some(today.as_str()) {
                settings::set_string(&name, &today);
                self.view.command(Command::Remind(reminder));
                return;
            }
        }
    }

    // MARK: Tray icon

    fn update_tray_icon(&mut self) {
        let size = sys::small_icon_size(self.last_dpi);
        let old = self.tray_icon;
        self.tray_icon = pet_icon(&self.view.about, size);
        let tip = format!("DeskPet · {}", self.view.about.short_name);
        if !sys::tray(self.owner, sys::TRAY_MODIFY, self.tray_icon, &tip) {
            sys::tray(self.owner, sys::TRAY_ADD, self.tray_icon, &tip);
        }
        sys::destroy_icon(old);
    }

    // MARK: Menu

    /// The menu, rebuilt every time it opens, plus the bitmaps to free once it closes.
    fn build_menu(&self) -> (Menu, Vec<HBITMAP>) {
        let mut bitmaps = Vec::new();
        let icon = sys::small_icon_size(self.last_dpi);
        let mut menu = Menu::new();
        menu.add(MenuItem { checked: self.visible, ..MenuItem::new("Show Pet", CMD_SHOW) });

        let mut pets = Menu::new();
        for (i, p) in self.pets.iter().enumerate() {
            if i > 0 && p.about.category != self.pets[i - 1].about.category {
                pets.separator(); // the list is grouped by category
            }
            let current = p.about.id == self.view.about.id;
            let bmp = sys::bitmap_from_pixels(&pet_pixels(&p.about, icon), icon);
            bitmaps.push(bmp);
            pets.add(MenuItem { checked: current, bold: current, bitmap: bmp, ..MenuItem::new(&p.about.name, CMD_PET + i as u32) });
        }
        pets.separator();
        pets.add(MenuItem::new("Get More Pets…", CMD_GET_PETS));
        let remove = self.current().file.as_ref().map(|_| format!("Remove {}", self.view.about.name));
        if let Some(remove) = &remove {
            pets.add(MenuItem::new(remove, CMD_REMOVE));
        }
        menu.submenu("Pet", pets);

        if !self.view.about.tricks.is_empty() {
            let mut tricks = Menu::new();
            tricks.add(MenuItem::new("Surprise Me", CMD_SURPRISE));
            tricks.separator();
            for (i, (title, _)) in self.view.about.tricks.iter().enumerate() {
                tricks.add(MenuItem::new(title, CMD_TRICK + i as u32));
            }
            menu.submenu("Tricks", tricks);
        }
        menu.separator();

        let mut ranges = Menu::new();
        for (i, (value, title)) in RANGE_OPTIONS.iter().enumerate() {
            ranges.add(MenuItem { checked: *value == self.range, radio: true, ..MenuItem::new(title, CMD_RANGE + i as u32) });
        }
        menu.submenu("Roam Range", ranges);
        menu.add(MenuItem::new("Show Roam Area", CMD_SHOW_AREA));
        menu.add(MenuItem::new("Move Home to Corner", CMD_CORNER));
        menu.separator();

        let mut reminders = Menu::new();
        reminders.add(MenuItem { checked: self.water_on, ..MenuItem::new("Drink Water (every 20–30 min)", CMD_WATER) });
        reminders.add(MenuItem { checked: self.meals_on, ..MenuItem::new("Main Meals (lunch 12:00, dinner 18:30)", CMD_MEALS) });
        reminders.add(MenuItem { checked: self.breaks_on, ..MenuItem::new("Stretch Break (every 40–60 min)", CMD_BREAKS) });
        menu.submenu("Reminders", reminders);

        // The setting can also change in Task Manager → Startup apps, so it's re-read every time.
        match settings::login_state() {
            LoginState::TurnedOffInTaskManager => {
                menu.add(MenuItem { checked: true, radio: true, ..MenuItem::new("Open at Login (turned off in Task Manager)", CMD_LOGIN) })
            }
            state => menu.add(MenuItem { checked: state == LoginState::On, ..MenuItem::new("Open at Login", CMD_LOGIN) }),
        }
        menu.separator();

        let pet = &self.view.about;
        if let (Some(click), Some(right)) = (&pet.click_trick, &pet.right_click_trick) {
            // A pet whose clicks do tricks (Moona) says what they do, and how to reach this menu.
            let hint = format!("Click: {click} · right-click: {right}");
            menu.add(MenuItem { enabled: false, ..MenuItem::new(&hint, 0) });
            menu.add(MenuItem { enabled: false, ..MenuItem::new("Drag to move its home · Shift+right-click: this menu", 0) });
        } else {
            menu.add(MenuItem { enabled: false, ..MenuItem::new("Drag the pet to move its home · click to pet", 0) });
        }
        menu.add(MenuItem::new("Quit DeskPet", CMD_QUIT));
        (menu, bitmaps)
    }

    fn command(&mut self, id: u32) -> After {
        match id {
            CMD_SHOW => {
                self.visible = !self.visible;
                settings::set_bool("visible", self.visible);
                if self.visible {
                    self.view.say("hi!", 1.5);
                }
                let show = self.visible && !self.fullscreen;
                self.set_shown(show);
            }
            CMD_SURPRISE => {
                self.view.command(Command::SurpriseTrick);
            }
            CMD_SHOW_AREA => {
                self.view.command(Command::ShowPen(3.0));
            }
            CMD_CORNER => {
                let abs = self.pet_screen_x();
                self.home_x = None;
                settings::remove("homeX");
                self.layout();
                self.keep_pet(abs);
                self.view.command(Command::ShowPen(2.0));
            }
            CMD_WATER => {
                self.water_on = !self.water_on;
                settings::set_bool("water", self.water_on);
                self.active_since_water = 0.0;
            }
            CMD_MEALS => {
                self.meals_on = !self.meals_on;
                settings::set_bool("meals", self.meals_on);
            }
            CMD_BREAKS => {
                self.breaks_on = !self.breaks_on;
                settings::set_bool("breaks", self.breaks_on);
                self.streak_start = uptime();
            }
            CMD_LOGIN => {
                if settings::login_state() == LoginState::On {
                    settings::login_off();
                } else if !settings::login_on() {
                    return After::LoginFailed;
                }
            }
            CMD_QUIT => return After::Quit,
            CMD_GET_PETS => sys::open_url(&net::site()),
            CMD_REMOVE => {
                if let Some(file) = &self.current().file {
                    return After::Remove(file.clone(), self.view.about.name.clone());
                }
            }
            _ if (CMD_PET..CMD_PET + self.pets.len() as u32).contains(&id) => {
                self.switch_to((id - CMD_PET) as usize);
            }
            _ if (CMD_TRICK..CMD_TRICK + self.view.about.tricks.len() as u32).contains(&id) => {
                self.view.command(Command::Trick((id - CMD_TRICK) as usize));
            }
            _ if (CMD_RANGE..CMD_RANGE + RANGE_OPTIONS.len() as u32).contains(&id) => {
                let abs = self.pet_screen_x();
                self.range = RANGE_OPTIONS[(id - CMD_RANGE) as usize].0;
                settings::set_f64("range", self.range);
                self.layout();
                self.keep_pet(abs);
                self.view.command(Command::StopRoaming);
                self.view.command(Command::ShowPen(2.5));
            }
            _ => {}
        }
        After::Nothing
    }

    /// Swaps in another pet where this one stands; it says hello.
    fn switch_to(&mut self, i: usize) {
        let abs = self.pet_screen_x();
        self.view = start_pet(&self.pets, i);
        self.layout();
        self.keep_pet(abs);
        settings::set_string("pet", &self.view.about.id);
        self.update_tray_icon();
        self.view.command(Command::Hello);
    }

    /// The catalog entry of the pet on screen.
    fn current(&self) -> &Entry {
        self.pets.iter().find(|p| p.about.id == self.view.about.id).unwrap_or(&self.pets[0])
    }

    /// Re-reads the pet list after pets were added, removed or updated. Shows pet `show` if given;
    /// otherwise restarts the pet on screen if its file changed, or swaps in the first pet if it's gone.
    fn reload(&mut self, show: Option<&str>) {
        self.pets = pets::catalog(self.extra.as_deref());
        let id = show.unwrap_or(&self.view.about.id).to_string();
        match self.pets.iter().position(|p| p.about.id == id) {
            Some(i) if show.is_some() || self.pets[i].about.version != self.view.about.version => self.switch_to(i),
            Some(_) => {}
            None => self.switch_to(0),
        }
    }

    fn quit(&mut self) {
        unsafe {
            KillTimer(self.owner, 1);
        }
        sys::tray(self.owner, sys::TRAY_DELETE, null_mut(), "");
        sys::destroy_icon(self.tray_icon);
        self.tray_icon = null_mut();
        unsafe {
            DestroyWindow(self.pet_hwnd);
            DestroyWindow(self.owner);
            PostQuitMessage(0);
        }
    }
}

// MARK: Things that run their own message loop, so they run with the app not borrowed

fn show_menu() {
    let Some((menu, bitmaps, owner)) = with_app(|a| {
        if a.menu_open {
            return None;
        }
        a.menu_open = true;
        let (m, b) = a.build_menu();
        Some((m, b, a.owner))
    })
    .flatten() else {
        return;
    };
    let id = menu.show(owner);
    drop(menu);
    for b in bitmaps {
        sys::delete_bitmap(b);
    }
    let after = with_app(|a| {
        a.menu_open = false;
        a.command(id)
    });
    match after {
        Some(After::LoginFailed) => sys::message_box("Couldn't change Open at Login.\n\nYou can add DeskPet yourself in Settings → Apps → Startup.", true),
        Some(After::Remove(file, name)) if sys::ask(&format!("Remove {name}?\n\nYou can download it again from the DeskPet website.")) => match pets::remove(&file) {
            Ok(()) => {
                with_app(|a| a.reload(None));
            }
            Err(e) => sys::message_box(&format!("Couldn't remove {name}: {e}."), true),
        },
        Some(After::Quit) => {
            with_app(|a| a.quit());
        }
        _ => {}
    }
}

/// Installs the pet files waiting in `TO_INSTALL` and shows the last one.
fn install_waiting() {
    while let Some(path) = TO_INSTALL.with(|q| q.borrow_mut().pop_front()) {
        match pets::install(&path) {
            Ok(id) => {
                with_app(|a| a.reload(Some(&id)));
            }
            Err(e) => {
                let name = path.file_name().map_or(String::new(), |n| n.to_string_lossy().into_owned());
                sys::message_box(&format!("Couldn't add the pet {name}: {e}."), true);
            }
        }
    }
}

/// Queues pet files to install; non-pet files are ignored.
fn queue_install(paths: impl IntoIterator<Item = String>) {
    let pets: Vec<PathBuf> = paths.into_iter().map(PathBuf::from).filter(|p| is_pet_file(p)).collect();
    if !pets.is_empty() {
        TO_INSTALL.with(|q| q.borrow_mut().extend(pets));
        unsafe { PostMessageW(OWNER.with(Cell::get), WM_INSTALL, 0, 0) };
    }
}

fn is_pet_file(p: &Path) -> bool {
    p.extension().is_some_and(|x| x.eq_ignore_ascii_case("deskpet"))
}

// MARK: Window procedures

unsafe extern "system" fn pet_proc(hwnd: HWND, msg: u32, wp: WPARAM, lp: LPARAM) -> LRESULT {
    match msg {
        WM_MOUSEACTIVATE => return MA_NOACTIVATE as LRESULT,
        WM_LBUTTONDOWN => {
            let began = with_app(|a| {
                a.mouse_down = true;
                a.drag_moved = false;
                a.drag_start_mouse_x = sys::cursor().0;
                a.drag_start_x = a.x;
                a.view.command(Command::BeginDrag);
            });
            if began.is_some() {
                sys::capture(hwnd);
            }
            return 0;
        }
        WM_MOUSEMOVE => {
            with_app(|a| {
                if a.mouse_down {
                    let dx = sys::cursor().0 - a.drag_start_mouse_x;
                    if dx.abs() as f64 > 3.0 * a.scale {
                        a.drag_moved = true;
                    }
                    if a.drag_moved {
                        a.x = (a.drag_start_x + dx).max(a.min_x).min(a.min_x.max(a.max_x - a.w));
                        a.render();
                    }
                }
            });
            return 0;
        }
        WM_LBUTTONUP | WM_CAPTURECHANGED => {
            let ended = with_app(|a| {
                if !a.mouse_down {
                    return false;
                }
                a.mouse_down = false;
                a.view.command(Command::EndDrag { moved: a.drag_moved });
                a.save_home_if_moved();
                true
            });
            if msg == WM_LBUTTONUP && ended == Some(true) {
                sys::release_capture();
            }
            return 0;
        }
        WM_RBUTTONUP => {
            // Shift+right-click always opens the menu, for pets whose right-click does a trick.
            let shift = windows_sys::Win32::UI::Input::KeyboardAndMouse::GetKeyState(windows_sys::Win32::UI::Input::KeyboardAndMouse::VK_SHIFT as i32) < 0;
            if shift || with_app(|a| a.view.command(Command::RightClick)) != Some(true) {
                show_menu();
            }
            return 0;
        }
        WM_DPICHANGED => {
            with_app(|a| a.screens_changed());
            return 0;
        }
        WM_DROPFILES => {
            // A pet file dropped on the pet.
            queue_install(sys::dropped_files(wp as _));
            return 0;
        }
        _ => {}
    }
    DefWindowProcW(hwnd, msg, wp, lp)
}

unsafe extern "system" fn owner_proc(hwnd: HWND, msg: u32, wp: WPARAM, lp: LPARAM) -> LRESULT {
    match msg {
        WM_COPYDATA => {
            // A second DeskPet (from double-clicking a pet file) hands the file over.
            return match sys::received_pet_file(lp) {
                Some(path) => {
                    queue_install([path]);
                    1
                }
                None => 0,
            };
        }
        WM_INSTALL => {
            install_waiting();
            return 0;
        }
        WM_PETS_UPDATED => {
            with_app(|a| a.reload(None));
            return 0;
        }
        WM_TIMER => {
            with_app(|a| a.tick());
            return 0;
        }
        sys::TRAY_CALLBACK => {
            let event = (lp & 0xFFFF) as u32;
            if event == WM_LBUTTONUP || event == WM_RBUTTONUP {
                show_menu();
            }
            return 0;
        }
        WM_DISPLAYCHANGE | WM_SETTINGCHANGE | WM_DPICHANGED => {
            with_app(|a| a.screens_changed());
        }
        _ if msg != 0 && msg == TASKBAR_CREATED.with(|t| t.get()) => {
            // Explorer restarted: the tray icon has to be added again.
            with_app(|a| {
                let tip = format!("DeskPet · {}", a.view.about.short_name);
                sys::tray(a.owner, sys::TRAY_ADD, a.tray_icon, &tip);
            });
            return 0;
        }
        _ => {}
    }
    DefWindowProcW(hwnd, msg, wp, lp)
}

fn main() {
    // `--pet <id>` starts with that pet for this run; `--pets <folder>` also loads the pets in a
    // folder (build-pets.ps1 makes them in build\pets). A pet file (double-clicked) is installed.
    let args: Vec<String> = std::env::args().skip(1).collect();
    let value = |flag: &str| args.iter().position(|a| a == flag).and_then(|i| args.get(i + 1)).map(String::as_str);
    let pet_files: Vec<String> = args.iter().filter(|a| is_pet_file(Path::new(a)) && Some(a.as_str()) != value("--pets")).cloned().collect();

    let Some(_single) = sys::single_instance() else {
        // Already running: hand it the pet files.
        for f in &pet_files {
            let full = std::fs::canonicalize(f).map_or(f.clone(), |p| p.to_string_lossy().trim_start_matches(r"\\?\").to_string());
            sys::send_pet_file("DeskPetOwner", &full);
        }
        return;
    };
    sys::allow_dark_menus();
    uptime();
    TASKBAR_CREATED.with(|t| t.set(sys::taskbar_created_message()));

    let pet_class = wide("DeskPetPen");
    let owner_class = wide("DeskPetOwner");
    sys::register_class(&pet_class, Some(pet_proc), null_mut());
    sys::register_class(&owner_class, Some(owner_proc), null_mut());
    let title = wide("DeskPet");
    let (owner, pet_hwnd) = unsafe {
        let owner =
            CreateWindowExW(WS_EX_TOOLWINDOW, owner_class.as_ptr(), title.as_ptr(), WS_POPUP, 0, 0, 0, 0, null_mut(), null_mut(), sys::instance(), null());
        let pet = CreateWindowExW(
            WS_EX_LAYERED | WS_EX_TOOLWINDOW | WS_EX_TOPMOST | WS_EX_NOACTIVATE | WS_EX_TRANSPARENT,
            pet_class.as_ptr(),
            title.as_ptr(),
            WS_POPUP,
            0,
            0,
            1,
            1,
            null_mut(),
            null_mut(),
            sys::instance(),
            null(),
        );
        (owner, pet)
    };

    OWNER.with(|o| o.set(owner));
    unsafe { windows_sys::Win32::UI::Shell::DragAcceptFiles(pet_hwnd, TRUE) };
    pets::register_file_type();

    let mut app = App::new(value("--pets").map(PathBuf::from), value("--pet"), pet_hwnd, owner);
    app.layout();
    let r = &app.view.r;
    let middle = (r.min_pet_x + r.max_pet_x) / 2.0;
    app.view.command(Command::PlaceAt(middle));
    app.view.say("hi!", 2.0);
    app.update_tray_icon();

    // `--demo` cycles through every activity, reminder and trick; `--tricks` plays only the pet's tricks;
    // `--typing` and `--pastime` show just that activity.
    let has = |f: &str| args.iter().any(|a| a == f);
    let (demo, tricks, typing, pastime) = (has("--demo"), has("--tricks"), has("--typing"), has("--pastime"));
    if demo || tricks || typing || pastime {
        let about = &app.view.about;
        let q = &mut app.demo;
        if demo {
            q.push_back((6.0, Command::DemoWalk));
            q.push_back((6.0, Command::DemoWalk));
            q.push_back((7.0, Command::DemoTyping(6.0)));
            if about.sweeps {
                q.push_back((7.0, Command::DemoSweep(6.0)));
            }
            if about.has_pastime {
                q.push_back((7.0, Command::DemoPastime(6.0)));
            }
            q.push_back((7.0, Command::Remind(Reminder::Water)));
            q.push_back((7.0, Command::Remind(Reminder::Lunch)));
            q.push_back((7.0, Command::Remind(Reminder::Stretch)));
        }
        if typing {
            q.push_back((9.0, Command::DemoTyping(8.0)));
        } else if pastime {
            q.push_back((9.0, Command::DemoPastime(8.0)));
        } else {
            for (i, (_, seconds)) in about.tricks.iter().enumerate() {
                q.push_back((seconds + 0.5, Command::Trick(i)));
            }
        }
        app.next_demo = uptime() + 2.0;
    }

    let show = app.visible;
    APP.with(|a| *a.borrow_mut() = Some(app));
    with_app(|a| {
        a.set_shown(show);
        a.set_timer(if show { FRAME_MS } else { HIDDEN_MS });
    });
    queue_install(pet_files);

    // Installed pets are kept up to date from the website: shortly after starting, then once a day.
    let owner_id = owner as isize;
    std::thread::spawn(move || {
        std::thread::sleep(std::time::Duration::from_secs(20));
        loop {
            if !pets::update().is_empty() {
                unsafe { PostMessageW(owner_id as HWND, WM_PETS_UPDATED, 0, 0) };
            }
            std::thread::sleep(std::time::Duration::from_secs(24 * 60 * 60));
        }
    });

    unsafe {
        let mut msg: MSG = std::mem::zeroed();
        while GetMessageW(&mut msg, null_mut(), 0, 0) > 0 {
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
    }
}
