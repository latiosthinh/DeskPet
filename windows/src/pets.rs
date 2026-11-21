//! Where this app keeps installed pets (`%LOCALAPPDATA%\DeskPet\pets`), the shared catalog with
//! that folder filled in, and the `.deskpet` file type.

use crate::{net, sys};
#[allow(unused_imports)] // the app and petshot each use some of these
pub use deskpet_host::catalog::{describe, is_pet_file, remove, Entry};
use std::path::{Path, PathBuf};

/// Where installed pets live.
pub fn folder() -> Option<PathBuf> {
    std::env::var_os("LOCALAPPDATA").map(|d| Path::new(&d).join("DeskPet").join("pets"))
}

/// The built-in pets, then the installed ones (and those in `extra`, from `--pets`).
pub fn catalog(extra: Option<&Path>) -> Vec<Entry> {
    deskpet_host::catalog::catalog(folder().as_deref(), extra)
}

/// Checks a pet file and copies it into the pets folder. Returns the pet's id.
pub fn install(path: &Path) -> Result<String, String> {
    deskpet_host::catalog::install(&folder().ok_or("no folder for pets")?, path)
}

/// Brings installed pets up to date from the website. Runs on a background thread; returns the ids it updated.
pub fn update() -> Vec<String> {
    match folder() {
        Some(dir) => deskpet_host::catalog::update(&dir, &net::site(), net::get),
        None => vec![],
    }
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
