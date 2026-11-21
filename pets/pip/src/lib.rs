//! Pip the penguin.

mod helpers;
mod tricks;
mod typing;

pub(crate) use helpers::*;
pub(crate) use petkit::prelude::*;

petkit::export_pet!(pet);

/// The whole pet: what it looks like, its tricks, and what it does while you type.
pub fn pet() -> PetDef {
    PetDef::new(info())
        .trick(tricks::hide_in_water::HideInWater)
        .trick(tricks::spit_water::SpitWater)
        .trick(tricks::swim::Swim::default())
        .trick(tricks::bubbles::Bubbles)
        .typing(typing::Swimming::default())
}

fn info() -> PetInfo {
    PetInfo {
        palette: Palette::new(&[
            ('k', rgb(0.12, 0.13, 0.20)),
            ('b', rgb(0.24, 0.29, 0.43)),
            ('w', rgb(1.00, 1.00, 1.00)),
            ('o', rgb(1.00, 0.65, 0.24)),
            ('p', rgb(1.00, 0.62, 0.70)),
        ]),
        body: [
            "................",
            "................",
            ".....kkkkkk.....",
            "...kkbbbbbbkk...",
            "..kbbbbbbwwwwk..",
            ".kbbbbbbwwwwwwk.",
            ".kbbbbbwwwwwwwk.",
            ".kbbbbbwwwwwwwko",
            ".kbbbbbwwwwwwpk.",
            ".kbbkbbwwwwwwwk.",
            ".kbbbkbwwwwwwwk.",
            "..kbbbkwwwwwwk..",
            "...kkkkkkkkkk...",
        ],
        feet: ["...kook..kook...", "....kook.kook..."],
        pastime: Some(Pastime::Knit),
        description: "A splashy penguin who swims, blows bubbles and hides in a puddle when you get too close.",
        ..PetInfo::new("pip", "Pip the penguin", Category::Critter)
    }
}
