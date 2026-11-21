//! Puddle the slime.

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
        .trick(tricks::ice_cream::IceCream)
        .trick(tricks::twin::Split::default())
        .trick(tricks::color_shift::ColorShift)
        .trick(tricks::pop_food::PopFood::default())
        .typing(typing::ReadingGlasses)
        .decorate(glasses)
}

/// Puts its reading glasses on while it sits with a book (while you type, [`typing::ReadingGlasses`] draws them).
fn glasses(cx: &Stage, look: &mut Look) {
    if cx.mode == Mode::Pastime && !cx.dragging {
        look.free.push(glasses_sprite(cx));
    }
}

fn info() -> PetInfo {
    PetInfo {
        palette: Palette::new(&[
            ('k', rgb(0.14, 0.31, 0.42)),
            ('c', rgb(0.56, 0.85, 0.95)),
            ('H', rgb(1.00, 1.00, 1.00)),
            ('d', rgb(0.39, 0.71, 0.86)),
            ('p', rgb(1.00, 0.62, 0.70)),
        ]),
        body: [
            "................",
            "................",
            "................",
            ".....kkkkkk.....",
            "...kkcccccckk...",
            "..kcHcccccccck..",
            ".kcHcccccccccck.",
            ".kcccccccccccck.",
            ".kccccccpccccpk.",
            ".kcccccccccccck.",
            ".kdccccccccccck.",
            ".kddcccccccccck.",
            ".kddddcccccccck.",
        ],
        feet: ["..kkkkkkkkkkkk..", ".kkkkkkkkkkkkkk."],
        sit_base: Some("..kkkkkkkkkkkk.."),
        paw: b'c',
        pastime: Some(Pastime::Book),
        ..PetInfo::new("puddle", "Puddle the slime", Category::Critter)
    }
}
