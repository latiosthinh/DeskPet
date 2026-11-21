//! Mochi.

mod art;
mod helpers;
mod tricks;

pub(crate) use art::*;
pub(crate) use helpers::*;
pub(crate) use petkit::prelude::*;

petkit::export_pet!(pet);

/// The whole pet: what it looks like, its tricks, and what it does while you type.
pub fn pet() -> PetDef {
    PetDef::new(info())
        .trick(tricks::yarn::Yarn::default())
        .trick(tricks::box_nap::BoxNap)
        .trick(tricks::laser::Laser::default())
        .trick(tricks::fishing::Fishing)
}

fn info() -> PetInfo {
    PetInfo {
        palette: Palette::new(&[('k', rgb(0.20, 0.16, 0.24)), ('w', rgb(0.99, 0.95, 0.89)), ('s', rgb(0.89, 0.82, 0.76)), ('p', rgb(1.00, 0.62, 0.70))]),
        body: [
            "................",
            "...kk......kk...",
            "..kppk....kppk..",
            "..kpwkkkkkkwpk..",
            ".kwwwwwwwwwwwwk.",
            ".kwwwwwwwwwwwwk.",
            ".kwwwwwwwwwwwwk.",
            ".kwwwwwwwwwwwwk.",
            ".kwwwwwwpwwwwpk.",
            ".kswwwwwwwwwwwk.",
            ".ksswwwwwwwwwwk.",
            "..ksssswwwwwwk..",
            "...kkkkkkkkkk...",
        ],
        feet: ["..kwk.....kwk...", "....kwk.kwk....."],
        sweeps: true,
        pastime: Some(Pastime::Book),
        description: "A sleepy little cat who naps on your taskbar, chases your cursor and sweeps up after you type.",
        ..PetInfo::new("mochi", "Mochi", Category::Critter)
    }
}
