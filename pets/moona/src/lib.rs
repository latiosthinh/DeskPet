//! Moona the moon knight.

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
        .trick(tricks::pounce::Pounce)
        .trick(tricks::glaive::Glaive)
        .trick(tricks::triplicate::Triplicate)
        .trick(tricks::moonlight::Moonlight)
        .typing(typing::Moonlight)
}

fn info() -> PetInfo {
    PetInfo {
        palette: Palette::new(&moona_palette()),
        paw: b'f',
        drinks: false,
        picture: Some(moona()),
        trick_gap: (30.0, 40.0),
        favorite_trick: Some("glaive"),
        turn_trick: Some("glaive"),
        wake_trick: Some("moonlight"),
        click_trick: Some("glaive"),
        right_click_trick: Some("moonlight"),
        description: "A moon knight riding a black panther. Throws her glaive, splits in three and calls down the moonlight.",
        ..PetInfo::new("moona", "Moona the moon knight", Category::Legend)
    }
}
