//! The pets this copy of DeskPet has: the three built in (one per category), plus any installed as
//! `<id>.deskpet` files in the app's pets folder. Every pet is a module, built-in or not.
//! Also installing and removing pet files and keeping them up to date. Each app says where its
//! pets folder is, and lends its own HTTP to [`update`].

use crate::module::PetModule;
use petkit::engine::About;
use std::path::{Path, PathBuf};
use std::rc::Rc;

/// The built-in pets, made into `build/pets` (by windows\build.ps1 or macos/build.sh) before the app is compiled.
pub const BUILT_IN: [&[u8]; 3] = [
    include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../build/pets/mochi.deskpet")),
    include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../build/pets/ninja.deskpet")),
    include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../build/pets/moona.deskpet")),
];

/// Bigger than any pet should be (the biggest today is about 200 KB).
pub const MAX_FILE: usize = 8 << 20;

pub struct Entry {
    pub about: About,
    pub bytes: Rc<[u8]>,
    /// Where an installed pet's file is; None for a built-in pet (or one loaded from an extra folder).
    pub file: Option<PathBuf>,
}

/// The pet website. `DESKPET_SITE` points the app at another copy (e.g. `http://127.0.0.1:8788`
/// while trying the site locally with `wrangler pages dev`).
pub fn site() -> String {
    std::env::var("DESKPET_SITE").ok().filter(|s| !s.is_empty()).unwrap_or_else(|| "https://deskpet-vn.pages.dev".into()).trim_end_matches('/').to_string()
}

/// Reads a module's description (starting it briefly to ask).
pub fn describe(bytes: &[u8]) -> Result<About, String> {
    let about = PetModule::load(bytes, 0.0, 1)?.about()?;
    let id_ok = !about.id.is_empty() && about.id.len() <= 32 && about.id.bytes().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == b'-');
    if !id_ok {
        return Err(format!("its id \"{}\" isn't allowed", about.id));
    }
    Ok(about)
}

/// True if version `a` ("1.2.0") is newer than `b`.
pub fn newer(a: &str, b: &str) -> bool {
    let parse = |v: &str| v.split('.').map(|n| n.trim().parse::<u64>().unwrap_or(0)).collect::<Vec<_>>();
    let (a, b) = (parse(a), parse(b));
    (0..a.len().max(b.len())).map(|i| (a.get(i).copied().unwrap_or(0), b.get(i).copied().unwrap_or(0))).find(|(x, y)| x != y).is_some_and(|(x, y)| x > y)
}

/// Built-in pets first, then installed ones (from `folder`, and from `extra` if given), each pet
/// once: an installed copy of a built-in pet replaces it when it is a newer version.
pub fn catalog(folder: Option<&Path>, extra: Option<&Path>) -> Vec<Entry> {
    let mut out: Vec<Entry> = BUILT_IN
        .iter()
        .filter_map(|b| match describe(b) {
            Ok(about) => Some(Entry { about, bytes: Rc::from(*b), file: None }),
            Err(e) => {
                eprintln!("a built-in pet won't start: {e}");
                None
            }
        })
        .collect();
    // Only pets in the pets folder count as installed (and can be removed or updated).
    let files = pet_files(folder).into_iter().map(|p| (p, true)).chain(pet_files(extra).into_iter().map(|p| (p, false)));
    for (path, installed) in files {
        let Ok(bytes) = std::fs::read(&path) else { continue };
        let about = match describe(&bytes) {
            Ok(about) => about,
            Err(e) => {
                eprintln!("skipping {}: {e}", path.display());
                continue;
            }
        };
        let entry = Entry { about, bytes: Rc::from(bytes), file: installed.then_some(path) };
        match out.iter().position(|e| e.about.id == entry.about.id) {
            Some(i) if newer(&entry.about.version, &out[i].about.version) => out[i] = entry,
            Some(_) => {}
            None => out.push(entry),
        }
    }
    // Grouped by shelf, keeping the order within each.
    out.sort_by_key(|e| e.about.category as u8);
    out
}

/// The `.deskpet` files in a folder, by name.
fn pet_files(dir: Option<&Path>) -> Vec<PathBuf> {
    let Some(entries) = dir.and_then(|d| std::fs::read_dir(d).ok()) else { return vec![] };
    let mut files: Vec<PathBuf> = entries.flatten().map(|e| e.path()).filter(|p| is_pet_file(p)).collect();
    files.sort();
    files
}

pub fn is_pet_file(p: &Path) -> bool {
    p.extension().is_some_and(|x| x.eq_ignore_ascii_case("deskpet"))
}

// MARK: Installing and removing

/// Checks a pet file and copies it into the pets folder as `<id>.deskpet`. Returns the pet's id.
pub fn install(folder: &Path, path: &Path) -> Result<String, String> {
    let bytes = std::fs::read(path).map_err(|e| format!("couldn't read it ({e})"))?;
    if bytes.len() > MAX_FILE {
        return Err("it's far too big to be a pet".into());
    }
    let about = describe(&bytes)?;
    save(folder, &about.id, &bytes)?;
    Ok(about.id)
}

/// Writes a pet's file into the pets folder, replacing any older copy in one step.
fn save(folder: &Path, id: &str, bytes: &[u8]) -> Result<(), String> {
    std::fs::create_dir_all(folder).map_err(|e| format!("couldn't make the pets folder ({e})"))?;
    let (file, temp) = (folder.join(format!("{id}.deskpet")), folder.join(format!("{id}.deskpet.new")));
    std::fs::write(&temp, bytes).map_err(|e| format!("couldn't save it ({e})"))?;
    std::fs::rename(&temp, &file).map_err(|e| format!("couldn't save it ({e})"))
}

pub fn remove(file: &Path) -> Result<(), String> {
    std::fs::remove_file(file).map_err(|e| format!("couldn't remove it ({e})"))
}

// MARK: Updates

/// Asks the website (`site`) for each installed pet's latest version and downloads the newer ones,
/// using the app's HTTP: `get(url, limit)` returns the body of a 200 response of at most `limit`
/// bytes. Runs on a background thread; returns the ids it updated.
pub fn update(folder: &Path, site: &str, get: impl Fn(&str, usize) -> Result<Vec<u8>, String>) -> Vec<String> {
    let mut updated = vec![];
    for path in pet_files(Some(folder)) {
        let Some(About { id, version, .. }) = std::fs::read(&path).ok().and_then(|b| describe(&b).ok()) else { continue };
        if path.file_stem().and_then(|s| s.to_str()) != Some(id.as_str()) {
            continue; // not saved by DeskPet; leave it be
        }
        let Ok(reply) = get(&format!("{site}/api/update/{id}"), 4096) else { continue };
        let Some(latest) = json_string(&String::from_utf8_lossy(&reply), "version") else { continue };
        if !newer(&latest, &version) {
            continue;
        }
        let Ok(bytes) = get(&format!("{site}/download/{id}"), MAX_FILE) else { continue };
        // Only ever replace a pet with the same pet, newer.
        let same_pet_newer = describe(&bytes).is_ok_and(|about| about.id == id && newer(&about.version, &version));
        if same_pet_newer && save(folder, &id, &bytes).is_ok() {
            updated.push(id);
        }
    }
    updated
}

/// The string value of `"key": "…"` in a small JSON object (all the update check needs).
fn json_string(json: &str, key: &str) -> Option<String> {
    let rest = &json[json.find(&format!("\"{key}\""))? + key.len() + 2..];
    let rest = rest.trim_start().strip_prefix(':')?.trim_start().strip_prefix('"')?;
    Some(rest[..rest.find('"')?].to_string())
}

// MARK: Icons

/// The pet in its own colours, cropped to its drawn pixels and centred in a `size`×`size` square
/// (0xAARRGGBB, top row first), for menus and the tray or menu bar. Uses a whole number of pixels
/// per art pixel when that fills the square well, so it stays crisp.
pub fn icon_pixels(pet: &About, size: i32) -> Vec<u32> {
    let size = size.max(0);
    let mut px = vec![0u32; (size * size) as usize];
    let all = &pet.icon;
    let width = all.first().map_or(0, |r| r.len());
    let used: Vec<usize> = (0..all.len()).filter(|&r| all[r].iter().any(|c| c.is_some())).collect();
    let (Some(&first), Some(&last)) = (used.first(), used.last()) else { return px };
    let rows = &all[first..=last];
    let cols_used: Vec<usize> = (0..width).filter(|&c| rows.iter().any(|r| r.get(c).is_some_and(|c| c.is_some()))).collect();
    let (Some(&c0), Some(&c_last)) = (cols_used.first(), cols_used.last()) else { return px };
    let cols = c_last - c0 + 1;

    let n = size as f64;
    let fit = n / cols.max(rows.len()) as f64;
    let k = if fit.floor() >= 1.0 && fit.floor() / fit >= 0.8 { fit.floor() } else { fit };
    let ox = ((n - cols as f64 * k) / 2.0).round();
    let oy = ((n - rows.len() as f64 * k) / 2.0).round();

    for (r, row) in rows.iter().enumerate() {
        for c in 0..cols {
            let Some(Some(color)) = row.get(c0 + c) else { continue };
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn built_in_pets_start() {
        for bytes in BUILT_IN {
            if let Err(e) = describe(bytes) {
                panic!("a built-in pet won't start: {e}");
            }
        }
    }

    #[test]
    fn versions() {
        assert!(newer("1.2.0", "1.1.9"));
        assert!(newer("1.10.0", "1.9.0"));
        assert!(newer("2", "1.9"));
        assert!(!newer("1.0.0", "1.0"));
        assert!(!newer("1.0.0", "1.0.1"));
    }

    #[test]
    fn reads_the_version() {
        assert_eq!(json_string(r#"{"version": "1.2.0"}"#, "version").as_deref(), Some("1.2.0"));
        assert_eq!(json_string(r#"{"id":"x"}"#, "version"), None);
    }

    #[test]
    fn icons_have_the_pet() {
        let about = describe(BUILT_IN[0]).unwrap();
        let px = icon_pixels(&about, 32);
        assert_eq!(px.len(), 32 * 32);
        assert!(px.iter().filter(|&&p| p >> 24 == 255).count() > 100);
    }
}
