//! Sprout the seedling.

mod art;
mod helpers;
mod tricks;
mod typing;

pub(crate) use art::*;
pub(crate) use helpers::*;
pub(crate) use petkit::prelude::*;

petkit::export_pet!(pet);

/// The whole pet: what it looks like, its tricks, and what it does while you type.
pub fn pet() -> PetDef {
    PetDef::new(info())
        .trick(tricks::give_flower::GiveFlower)
        .trick(tricks::shower::Shower)
        .trick(tricks::plant_self::PlantSelf)
        .trick(tricks::grow_up::GrowUp)
        .trick(tricks::chase_butterfly::ChaseButterfly::default())
        .typing(typing::Digging)
}

fn info() -> PetInfo {
    PetInfo {
        palette: Palette::new(&[
            ('k', rgb(0.14, 0.25, 0.16)),
            ('w', rgb(0.71, 0.88, 0.54)),
            ('s', rgb(0.55, 0.76, 0.40)),
            ('L', rgb(0.36, 0.70, 0.32)),
            ('S', rgb(0.25, 0.54, 0.23)),
            ('p', rgb(1.00, 0.62, 0.70)),
        ]),
        body: [
            "..kkk...kkk.....",
            ".kLLLk.kLLLk....",
            "..kLLLkLLLk.....",
            "..kkkkSkkkkkkk..",
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
        pastime: Some(Pastime::Flower),
        description: "A tiny seedling who plants itself, grows up, chases butterflies and gives you flowers.",
        ..PetInfo::new("sprout", "Sprout the seedling", Category::Critter)
    }
}
