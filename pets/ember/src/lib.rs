//! Ember the fox.

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
        .trick(tricks::firework::Firework::default())
        .trick(tricks::flashlight::Flashlight)
        .trick(tricks::flip_egg::FlipEgg)
        .trick(tricks::campfire::Campfire)
        .typing(typing::Cooking)
}

fn info() -> PetInfo {
    PetInfo {
        palette: Palette::new(&[('k', rgb(0.23, 0.14, 0.13)), ('w', rgb(0.95, 0.53, 0.23)), ('s', rgb(0.83, 0.42, 0.16)), ('c', rgb(1.00, 0.95, 0.86))]),
        body: [
            ".....k.....k....",
            "....kwk...kwk...",
            "....kwwk.kwwk...",
            "...kwwwkkkwwwk..",
            "...kwwwwwwwwwwk.",
            "...kwwwwwwwwwwk.",
            ".k.kwwwwwwwwwwk.",
            "kckkwwwwwwwwwwk.",
            "kwkkwwwwccccckk.",
            "kwwkswwwcccccck.",
            "kwwksswwwccccck.",
            ".kkkssswwwwwwk..",
            "....kkkkkkkkk...",
        ],
        feet: ["....kwk...kwk...", ".....kwk.kwk...."],
        pastime: Some(Pastime::Yoyo),
        description: "A curious fox who lights fireworks, explores with a flashlight and toasts marshmallows by the fire.",
        ..PetInfo::new("ember", "Ember the fox", Category::Critter)
    }
}
