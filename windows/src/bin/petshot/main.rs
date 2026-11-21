//! petshot: the pet maker's tool. Runs pet modules the same way DeskPet does, off screen.
//!
//!   petshot check <pet.deskpet | folder>...    try every trick, activity and reminder, and report problems
//!   petshot gifs <pet.deskpet> <folder>        write hero.gif and <trick-id>.gif for the website
//!   petshot site <pets folder> <site folder>   check every pet, make its GIFs, copy its module and
//!                                               update pets.json (what the website is built from)

// The app's own module host and drawing code, so pets look exactly as they do on the taskbar.
#[allow(dead_code)]
#[path = "../../canvas.rs"]
mod canvas;
#[allow(dead_code)]
#[path = "../../module.rs"]
mod module;
#[allow(dead_code)]
#[path = "../../net.rs"]
mod net;
#[allow(dead_code)]
#[path = "../../pet.rs"]
mod pet;
#[allow(dead_code)]
#[path = "../../pets.rs"]
mod pets;
#[allow(dead_code)]
#[path = "../../sys.rs"]
mod sys;

mod gif;

use canvas::{Canvas, TextCache};
use pet::PetView;
use petkit::art::{prop_palette, Palette, PET_SIZE, PROP_ROOM};
use petkit::engine::{About, Command, Input, Reminder};
use petkit::look::pt;
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

/// The largest module the website takes.
const MAX_MODULE: u64 = 2 << 20;
/// The longest a trick may last, in seconds.
const MAX_TRICK: f64 = 15.0;
/// Simulation step, and GIF frame time (20 frames a second).
const DT: f64 = 0.05;
/// Device pixels per point in the GIFs (4 per art pixel, like a 100% display).
const SCALE: f64 = 1.0;
/// GIF background: the colour of the website's picture frames (--frame in site.css).
const BACKGROUND: u32 = 0xf1e3c8;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let result = match args.first().map(String::as_str) {
        Some("check") if args.len() > 1 => check_all(&args[1..]),
        Some("gifs") if args.len() == 3 => gifs(Path::new(&args[1]), Path::new(&args[2])),
        Some("site") if args.len() == 3 => site(Path::new(&args[1]), Path::new(&args[2])),
        _ => Err(include_str!("usage.txt").to_string()),
    };
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("{e}");
            ExitCode::FAILURE
        }
    }
}

/// The `.deskpet` files named, or in the folders named.
fn modules(args: &[String]) -> Vec<PathBuf> {
    let mut out = vec![];
    for a in args {
        let p = PathBuf::from(a);
        if p.is_dir() {
            let mut files: Vec<PathBuf> = std::fs::read_dir(&p)
                .into_iter()
                .flatten()
                .flatten()
                .map(|e| e.path())
                .filter(|f| f.extension().is_some_and(|x| x.eq_ignore_ascii_case("deskpet")))
                .collect();
            files.sort();
            out.extend(files);
        } else {
            out.push(p);
        }
    }
    out
}

// MARK: Running a pet off screen

/// A pet running in a pen of its own, with a clock.
struct Run {
    view: PetView,
    now: f64,
    w: f64,
    h: f64,
}

impl Run {
    fn new(bytes: &[u8], about: &About, range: f64) -> Result<Run, String> {
        let view = PetView::new(bytes, 0.0, 42)?;
        let (w, h) = (2.0 * range + PET_SIZE + 2.0 * PROP_ROOM, about.tallest + 70.0);
        let mut run = Run { view, now: 0.0, w, h };
        run.send(Command::Pen { width: w, height: h, range })?;
        let r = &run.view.r;
        let middle = (r.min_pet_x + r.max_pet_x) / 2.0;
        run.send(Command::PlaceAt(middle))?;
        Ok(run)
    }

    fn send(&mut self, c: Command) -> Result<(), String> {
        self.view.command(c);
        self.failed()
    }

    fn failed(&self) -> Result<(), String> {
        self.view.failed.clone().map_or(Ok(()), Err)
    }

    /// One step: someone busy at the desk (not idle, not typing), the pointer far away.
    fn step(&mut self) -> Result<(), String> {
        self.now += DT;
        self.view.tick(Input { dt: DT, now: self.now, idle: 0.0, key_idle: 99.0, mouse: pt(-1000.0, 1000.0) });
        self.failed()
    }

    /// The current frame as opaque 0xRRGGBB pixels on the background.
    fn picture(&self, text: &mut TextCache) -> (Vec<u32>, usize, usize) {
        let (w, h) = ((self.w * SCALE).round() as i32, (self.h * SCALE).round() as i32);
        let mut buf = vec![0u32; (w * h) as usize];
        let mut c = Canvas { buf: &mut buf, w, h, s: SCALE, text };
        c.clear();
        self.view.draw(&mut c);
        // The canvas is premultiplied: colour + background × (1 − alpha).
        let over = |p: u32| {
            let a = p >> 24;
            let ch = |s: u32| ((p >> s & 0xFF) + ((BACKGROUND >> s & 0xFF) * (255 - a) + 127) / 255).min(255);
            ch(16) << 16 | ch(8) << 8 | ch(0)
        };
        (buf.into_iter().map(over).collect(), w as usize, h as usize)
    }
}

// MARK: check

fn check_all(args: &[String]) -> Result<(), String> {
    let files = modules(args);
    if files.is_empty() {
        return Err("no .deskpet files there".into());
    }
    let mut bad = 0;
    for f in &files {
        match check(f) {
            Ok(about) => println!("ok    {} ({} {}, {} tricks)", f.display(), about.name, about.version, about.tricks.len()),
            Err(problems) => {
                bad += 1;
                println!("FAIL  {}", f.display());
                for p in problems {
                    println!("      - {p}");
                }
            }
        }
    }
    if bad > 0 {
        return Err(format!("{bad} of {} pets have problems", files.len()));
    }
    Ok(())
}

/// Everything a pet must get right. Returns its description, or every problem found.
fn check(file: &Path) -> Result<About, Vec<String>> {
    let bytes = std::fs::read(file).map_err(|e| vec![format!("can't read it: {e}")])?;
    let mut problems = vec![];
    if bytes.len() as u64 > MAX_MODULE {
        problems.push(format!("it's {} KB; the most a pet may be is {} KB", bytes.len() / 1024, MAX_MODULE / 1024));
    }
    let about = pets::describe(&bytes).map_err(|e| vec![e])?;
    problems.extend(about.problems.iter().cloned());

    let slug = |s: &str| !s.is_empty() && s.bytes().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == b'-');
    if file.file_stem().and_then(|s| s.to_str()) != Some(about.id.as_str()) {
        problems.push(format!("the file should be named {}.deskpet, after the pet's id", about.id));
    }
    if about.name.trim().is_empty() || about.name.len() > 40 {
        problems.push("its name should be 1 to 40 characters".into());
    }
    if about.description.trim().is_empty() {
        problems.push("it needs a description (PetInfo.description), one sentence for the website".into());
    }
    if about.author.trim().is_empty() {
        problems.push("it needs an author (PetInfo.author)".into());
    }
    let parts: Vec<&str> = about.version.split('.').collect();
    if parts.len() != 3 || parts.iter().any(|p| p.parse::<u32>().is_err()) {
        problems.push(format!("its version \"{}\" should look like 1.0.0 (set it in Cargo.toml)", about.version));
    }
    if about.tricks.is_empty() {
        problems.push("it has no tricks; add at least one with .trick(...)".into());
    }
    let (mut ids, mut titles) = (BTreeSet::new(), BTreeSet::new());
    for t in &about.tricks {
        if !slug(&t.id) {
            problems.push(format!("trick id \"{}\": use lowercase letters, digits and -", t.id));
        }
        if !ids.insert(t.id.clone()) {
            problems.push(format!("two tricks have the id \"{}\"", t.id));
        }
        if t.title.trim().is_empty() || !titles.insert(t.title.clone()) {
            problems.push(format!("trick \"{}\" needs a title of its own", t.id));
        }
        if !(t.seconds > 0.0 && t.seconds <= MAX_TRICK) {
            problems.push(format!("trick \"{}\" lasts {} s; keep it between 0 and {MAX_TRICK} s", t.id, t.seconds));
        }
    }

    // Play everything it can do, looking at every frame.
    let mut plays: Vec<(String, Command, f64)> = vec![("walking about".into(), Command::DemoWalk, 20.0)];
    for (i, t) in about.tricks.iter().enumerate() {
        plays.push((format!("trick \"{}\"", t.id), Command::Trick(i), t.seconds + 1.0));
    }
    plays.push(("typing".into(), Command::DemoTyping(6.0), 7.0));
    if about.has_pastime {
        plays.push(("its pastime".into(), Command::DemoPastime(6.0), 7.0));
    }
    if about.sweeps {
        plays.push(("sweeping".into(), Command::DemoSweep(6.0), 7.0));
    }
    for (name, r) in [("water", Reminder::Water), ("lunch", Reminder::Lunch), ("dinner", Reminder::Dinner), ("stretch", Reminder::Stretch)] {
        plays.push((format!("the {name} reminder"), Command::Remind(r), 8.0));
    }
    plays.push(("being petted".into(), Command::EndDrag { moved: false }, 3.0));
    plays.push(("a right-click".into(), Command::RightClick, 8.0));

    // Each colour problem is reported once, where it first shows. The body's own art problems are
    // already listed above (from PetInfo::problems), so they aren't repeated.
    let mut seen: BTreeSet<String> = BTreeSet::new();
    let body_checked = !about.problems.is_empty();
    match Run::new(&bytes, &about, 80.0) {
        Err(e) => problems.push(format!("it won't start: {e}")),
        Ok(mut run) => {
            for (what, command, seconds) in plays {
                if matches!(command, Command::EndDrag { .. }) {
                    let _ = run.send(Command::BeginDrag);
                }
                let mut result = run.send(command);
                let start = run.now;
                while result.is_ok() && run.now - start < seconds {
                    result = run.step();
                    for p in colour_problems(&run.view.r) {
                        if !(body_checked && p.starts_with("the body")) && seen.insert(p.clone()) {
                            problems.push(format!("{what} at {:.1} s: {p}", run.now - start));
                        }
                    }
                }
                if let Err(e) = result {
                    problems.push(format!("{what} broke at {:.1} s: {e}", run.now - start));
                    break; // a broken module can't go on
                }
            }
        }
    }
    if problems.is_empty() {
        Ok(about)
    } else {
        Err(problems)
    }
}

/// Characters in a frame with no colour: usually a typo in the art or a colour missing from the palette.
fn colour_problems(r: &petkit::engine::Render) -> Vec<String> {
    let (look, mut out) = (&r.look, vec![]);
    let missing = |rows: &mut dyn Iterator<Item = u8>, palette: &Palette| -> BTreeSet<char> {
        rows.filter(|&c| c != b'.' && c != b' ' && palette.get(c).is_none()).map(|c| c as char).collect()
    };
    let mut props = prop_palette();
    props.set('w', Some(petkit::art::Color::new(0, 0, 0, 255)));
    props.set('k', Some(petkit::art::Color::new(0, 0, 0, 255)));
    // A picture pet draws its picture rather than the 16×16 frame.
    let body: Box<dyn Iterator<Item = u8>> = match &r.picture {
        Some(pic) => Box::new(pic.rows.iter().flatten().copied()),
        None => Box::new(look.frame.iter().flatten().copied()),
    };
    for c in missing(&mut { body }, &look.palette) {
        out.push(format!("the body uses '{c}', which isn't in its palette"));
    }
    for p in &look.props {
        for c in missing(&mut p.rows.iter().flat_map(|r| r.bytes()), &props) {
            out.push(format!("a prop uses '{c}', which isn't a prop colour"));
        }
    }
    for f in look.free.iter().chain(&look.back) {
        let palette = if f.own_colors { &look.palette } else { &props };
        for c in missing(&mut f.rows.iter().flat_map(|r| r.bytes()), palette) {
            out.push(format!("a sprite uses '{c}', which isn't in its colours"));
        }
    }
    out
}

// MARK: gifs

fn gifs(file: &Path, out: &Path) -> Result<(), String> {
    let bytes = std::fs::read(file).map_err(|e| format!("can't read {}: {e}", file.display()))?;
    let about = pets::describe(&bytes)?;
    std::fs::create_dir_all(out).map_err(|e| e.to_string())?;
    let mut text = TextCache::new();

    // The big one: saying hello and walking about.
    let mut jobs: Vec<(String, Vec<Command>, f64)> = vec![("hero".into(), vec![Command::Hello, Command::DemoWalk], 6.0)];
    for (i, t) in about.tricks.iter().enumerate() {
        jobs.push((t.id.clone(), vec![Command::Trick(i)], t.seconds + 0.5));
    }
    for (name, commands, seconds) in jobs {
        let mut run = Run::new(&bytes, &about, 40.0)?;
        for _ in 0..10 {
            run.step()?; // settle for half a second
        }
        for c in commands {
            run.send(c)?;
        }
        let mut frames = vec![];
        let (mut w, mut h) = (0, 0);
        while run.now < 0.5 + seconds {
            let (pixels, pw, ph) = run.picture(&mut text);
            (w, h) = (pw, ph);
            frames.push(pixels);
            run.step()?;
        }
        let gif = crop(frames, w, h);
        let path = out.join(format!("{name}.gif"));
        std::fs::write(&path, gif.encode()).map_err(|e| format!("can't write {}: {e}", path.display()))?;
        println!("wrote {} ({}×{}, {} frames)", path.display(), gif.w, gif.h, gif.frames.len());
    }
    Ok(())
}

/// Trims the background around everything that ever moves, leaving a small margin, and merges
/// repeated frames into longer ones.
fn crop(frames: Vec<Vec<u32>>, w: usize, h: usize) -> gif::Gif {
    let (mut x0, mut y0, mut x1, mut y1) = (w, h, 0, 0);
    for f in &frames {
        for y in 0..h {
            for x in 0..w {
                if f[y * w + x] != BACKGROUND {
                    (x0, y0, x1, y1) = (x0.min(x), y0.min(y), x1.max(x + 1), y1.max(y + 1));
                }
            }
        }
    }
    let m = 8;
    let (x0, y0, x1, y1) = (x0.saturating_sub(m), y0.saturating_sub(m), (x1 + m).min(w), h);
    let (cw, ch) = (x1.max(x0 + 1) - x0, y1.max(y0 + 1) - y0);
    let mut out: Vec<(Vec<u32>, u16)> = vec![];
    let cs = (DT * 100.0).round() as u16;
    for f in frames {
        let mut c = Vec::with_capacity(cw * ch);
        for y in y0..y0 + ch {
            c.extend_from_slice(&f[y * w + x0..y * w + x0 + cw]);
        }
        match out.last_mut() {
            Some((last, delay)) if *last == c => *delay += cs,
            _ => out.push((c, cs)),
        }
    }
    gif::Gif { w: cw, h: ch, frames: out }
}

// MARK: site

fn site(pets_dir: &Path, site_dir: &Path) -> Result<(), String> {
    let files = modules(&[pets_dir.to_string_lossy().into_owned()]);
    let json_path = site_dir.join("pets.json");
    let old = std::fs::read_to_string(&json_path).unwrap_or_else(|_| "[]".into());
    let built_in: Vec<String> = pets::catalog(None).into_iter().filter(|e| e.file.is_none()).map(|e| e.about.id).collect();

    let mut entries = vec![];
    let mut bad = 0;
    for f in &files {
        let about = match check(f) {
            Ok(about) => about,
            Err(problems) => {
                bad += 1;
                println!("FAIL  {} (left out)", f.display());
                for p in problems {
                    println!("      - {p}");
                }
                continue;
            }
        };
        println!("ok    {}", about.name);
        let public = site_dir.join("public");
        gifs(f, &public.join("pets").join(&about.id))?;
        let size = std::fs::metadata(f).map(|m| m.len()).unwrap_or(0);
        let builtin = built_in.contains(&about.id);
        if !builtin {
            let dest = public.join("files").join(format!("{}.deskpet", about.id));
            std::fs::create_dir_all(dest.parent().unwrap()).map_err(|e| e.to_string())?;
            std::fs::copy(f, &dest).map_err(|e| format!("can't copy to {}: {e}", dest.display()))?;
        }
        entries.push((about, builtin, size));
    }
    // Same order as before for pets already listed (the site shows them in this order), new ones after.
    let position = |id: &str| old.find(&format!("\"id\": \"{id}\"")).unwrap_or(usize::MAX);
    entries.sort_by_key(|(a, _, _)| (a.category as u8, position(&a.id)));
    std::fs::write(&json_path, pets_json(&entries)).map_err(|e| format!("can't write {}: {e}", json_path.display()))?;
    println!("wrote {} ({} pets)", json_path.display(), entries.len());
    if bad > 0 {
        return Err(format!("{bad} pets have problems and were left out"));
    }
    Ok(())
}

fn pets_json(entries: &[(About, bool, u64)]) -> String {
    let s = |v: &str| {
        let mut out = String::from("\"");
        for c in v.chars() {
            match c {
                '"' => out.push_str("\\\""),
                '\\' => out.push_str("\\\\"),
                c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
                c => out.push(c),
            }
        }
        out + "\""
    };
    let pets: Vec<String> = entries
        .iter()
        .map(|(a, builtin, size)| {
            let tricks: Vec<String> =
                a.tricks.iter().map(|t| format!("      {{\n        \"id\": {},\n        \"title\": {}\n      }}", s(&t.id), s(&t.title))).collect();
            format!(
                "  {{\n    \"id\": {},\n    \"name\": {},\n    \"category\": {},\n    \"author\": {},\n    \"version\": {},\n    \"description\": {},\n    \"builtin\": {},\n    \"tricks\": [\n{}\n    ],\n    \"file_size\": {}\n  }}",
                s(&a.id),
                s(&a.name),
                s(a.category.id()),
                s(&a.author),
                s(&a.version),
                s(&a.description),
                builtin,
                tricks.join(",\n"),
                if *builtin { "null".to_string() } else { size.to_string() },
            )
        })
        .collect();
    format!("[\n{}\n]\n", pets.join(",\n"))
}
