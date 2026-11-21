//! The pets this copy of DeskPet has: the three built in (one per category), plus any installed
//! as `<id>.deskpet` files in `%LOCALAPPDATA%\DeskPet\pets`. Every pet is a module, built-in or not.

use crate::module::PetModule;
use petkit::engine::About;
use std::path::{Path, PathBuf};
use std::rc::Rc;

/// The built-in pets, made by `build.ps1` into `build\pets` before the app is compiled.
const BUILT_IN: [&[u8]; 3] = [
    include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../build/pets/mochi.deskpet"
    )),
    include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../build/pets/ninja.deskpet"
    )),
    include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../build/pets/moona.deskpet"
    )),
];

pub struct Entry {
    pub about: About,
    pub bytes: Rc<[u8]>,
    /// Where an installed pet's file is; None for a built-in pet.
    pub file: Option<PathBuf>,
}

/// Reads a module's description (starting it briefly to ask).
pub fn describe(bytes: &[u8]) -> Result<About, String> {
    PetModule::load(bytes, 0.0, 1)?.about()
}

/// Where installed pets live.
pub fn folder() -> Option<PathBuf> {
    std::env::var_os("LOCALAPPDATA").map(|d| Path::new(&d).join("DeskPet").join("pets"))
}

/// Built-in pets first, then installed ones (from `folder()`, and from `extra` if given), each
/// pet once: an installed copy of a built-in pet replaces it when it is a newer version.
pub fn catalog(extra: Option<&Path>) -> Vec<Entry> {
    let mut out: Vec<Entry> = BUILT_IN
        .iter()
        .filter_map(|b| {
            Some(Entry {
                about: describe(b).ok()?,
                bytes: Rc::from(*b),
                file: None,
            })
        })
        .collect();
    let dirs = folder().into_iter().chain(extra.map(Path::to_path_buf));
    let mut files: Vec<PathBuf> = dirs
        .filter_map(|d| std::fs::read_dir(d).ok())
        .flatten()
        .flatten()
        .map(|e| e.path())
        .filter(|p| {
            p.extension()
                .is_some_and(|x| x.eq_ignore_ascii_case("deskpet"))
        })
        .collect();
    files.sort();
    for path in files {
        let Ok(bytes) = std::fs::read(&path) else {
            continue;
        };
        let Ok(about) = describe(&bytes) else {
            continue;
        };
        let entry = Entry {
            about,
            bytes: Rc::from(bytes),
            file: Some(path),
        };
        match out.iter().position(|e| e.about.id == entry.about.id) {
            Some(i) if entry.about.version > out[i].about.version => out[i] = entry,
            Some(_) => {}
            None => out.push(entry),
        }
    }
    out
}
