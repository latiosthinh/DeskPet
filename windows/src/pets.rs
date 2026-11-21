//! The pets this copy of DeskPet has: the three built in (one per category), plus any installed
//! as `<id>.deskpet` files in `%LOCALAPPDATA%\DeskPet\pets`. Every pet is a module, built-in or not.
//! Also installing and removing pet files, keeping them up to date, and the `.deskpet` file type.

use crate::module::PetModule;
use crate::{net, sys};
use petkit::engine::About;
use std::path::{Path, PathBuf};
use std::rc::Rc;

/// The built-in pets, made by `build.ps1` into `build\pets` before the app is compiled.
const BUILT_IN: [&[u8]; 3] = [
    include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/../build/pets/mochi.deskpet")),
    include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/../build/pets/ninja.deskpet")),
    include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/../build/pets/moona.deskpet")),
];

/// Bigger than any pet should be (the biggest today is about 200 KB).
const MAX_FILE: usize = 8 << 20;

pub struct Entry {
    pub about: About,
    pub bytes: Rc<[u8]>,
    /// Where an installed pet's file is; None for a built-in pet.
    pub file: Option<PathBuf>,
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

/// Where installed pets live.
pub fn folder() -> Option<PathBuf> {
    std::env::var_os("LOCALAPPDATA").map(|d| Path::new(&d).join("DeskPet").join("pets"))
}

/// True if version `a` ("1.2.0") is newer than `b`.
pub fn newer(a: &str, b: &str) -> bool {
    let parse = |v: &str| v.split('.').map(|n| n.trim().parse::<u64>().unwrap_or(0)).collect::<Vec<_>>();
    let (a, b) = (parse(a), parse(b));
    (0..a.len().max(b.len())).map(|i| (a.get(i).copied().unwrap_or(0), b.get(i).copied().unwrap_or(0))).find(|(x, y)| x != y).is_some_and(|(x, y)| x > y)
}

/// Built-in pets first, then installed ones (from `folder()`, and from `extra` if given), each
/// pet once: an installed copy of a built-in pet replaces it when it is a newer version.
pub fn catalog(extra: Option<&Path>) -> Vec<Entry> {
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
    let files = pet_files(folder()).into_iter().map(|p| (p, true)).chain(pet_files(extra.map(Path::to_path_buf)).into_iter().map(|p| (p, false)));
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
fn pet_files(dir: Option<PathBuf>) -> Vec<PathBuf> {
    let Some(entries) = dir.and_then(|d| std::fs::read_dir(d).ok()) else { return vec![] };
    let mut files: Vec<PathBuf> = entries.flatten().map(|e| e.path()).filter(|p| p.extension().is_some_and(|x| x.eq_ignore_ascii_case("deskpet"))).collect();
    files.sort();
    files
}

// MARK: Installing and removing

/// Checks a pet file and copies it into the pets folder as `<id>.deskpet`. Returns the pet's id.
pub fn install(path: &Path) -> Result<String, String> {
    let bytes = std::fs::read(path).map_err(|e| format!("couldn't read it ({e})"))?;
    if bytes.len() > MAX_FILE {
        return Err("it's far too big to be a pet".into());
    }
    let about = describe(&bytes)?;
    save(&about.id, &bytes)?;
    Ok(about.id)
}

/// Writes a pet's file into the pets folder, replacing any older copy in one step.
fn save(id: &str, bytes: &[u8]) -> Result<(), String> {
    let dir = folder().ok_or("no folder for pets")?;
    std::fs::create_dir_all(&dir).map_err(|e| format!("couldn't make the pets folder ({e})"))?;
    let (file, temp) = (dir.join(format!("{id}.deskpet")), dir.join(format!("{id}.deskpet.new")));
    std::fs::write(&temp, bytes).map_err(|e| format!("couldn't save it ({e})"))?;
    std::fs::rename(&temp, &file).map_err(|e| format!("couldn't save it ({e})"))
}

pub fn remove(file: &Path) -> Result<(), String> {
    std::fs::remove_file(file).map_err(|e| format!("couldn't remove it ({e})"))
}

// MARK: Updates

/// Asks the website for each installed pet's latest version and downloads the newer ones. Runs on
/// a background thread; returns the ids it updated.
pub fn update() -> Vec<String> {
    let site = net::site();
    let mut updated = vec![];
    for path in pet_files(folder()) {
        let Some(About { id, version, .. }) = std::fs::read(&path).ok().and_then(|b| describe(&b).ok()) else { continue };
        if path.file_stem().and_then(|s| s.to_str()) != Some(id.as_str()) {
            continue; // not saved by DeskPet; leave it be
        }
        let Ok(reply) = net::get(&format!("{site}/api/update/{id}"), 4096) else { continue };
        let Some(latest) = json_string(&String::from_utf8_lossy(&reply), "version") else { continue };
        if !newer(&latest, &version) {
            continue;
        }
        let Ok(bytes) = net::get(&format!("{site}/download/{id}"), MAX_FILE) else { continue };
        // Only ever replace a pet with the same pet, newer.
        let same_pet_newer = describe(&bytes).is_ok_and(|about| about.id == id && newer(&about.version, &version));
        if same_pet_newer && save(&id, &bytes).is_ok() {
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

// MARK: The .deskpet file type

/// Makes double-clicking a `.deskpet` file open it with this DeskPet (which installs it). Per
/// user, so it needs no admin rights; rewritten only when it changed (the app moved).
pub fn register_file_type() {
    let exe = sys::exe_path();
    let command = format!("\"{exe}\" \"%1\"");
    let key = r"Software\Classes\DeskPet.Pet";
    if sys::reg_get_string(&format!(r"{key}\shell\open\command"), "").as_deref() == Some(command.as_str()) {
        return;
    }
    sys::reg_set_string(r"Software\Classes\.deskpet", "", "DeskPet.Pet");
    sys::reg_set_string(key, "", "DeskPet pet");
    sys::reg_set_string(&format!(r"{key}\DefaultIcon"), "", &format!("\"{exe}\",0"));
    sys::reg_set_string(&format!(r"{key}\shell\open\command"), "", &command);
    sys::file_types_changed();
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
}
