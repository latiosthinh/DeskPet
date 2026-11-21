//! Orbis the witcher.

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
        .trick(tricks::orb::Orb)
        .trick(tricks::whirl::Whirl)
        .trick(tricks::weigh::Weigh)
        .trick(tricks::vanish::Vanish)
        .trick(tricks::sun_strike::SunStrike)
        .trick(tricks::tornado::Tornado)
        .trick(tricks::meteor::Meteor)
        .trick(tricks::moon_blast::MoonBlast)
        .trick(tricks::spirits::Spirits)
        .typing(typing::Orbs)
        .decorate(orbs)
}

/// His orbs turn on their wheel whenever he stands or walks (not in his other poses).
fn orbs(cx: &Stage, look: &mut Look) {
    if let Some(pic) = &cx.pet.picture {
        let i = pic.index_for(&look.frame);
        if i < 9 || pic.walk.contains(&i) {
            look.back.splice(0..0, orb_wheel(cx));
        }
    }
}

fn info() -> PetInfo {
    PetInfo {
        palette: Palette::new(&witcher_palette()),
        paw: b'L',
        drinks: false,
        picture: Some(witcher()),
        click_trick: Some("orb"),
        right_click_trick: Some("whirl"),
        ..PetInfo::new("witcher", "Orbis the witcher", Category::Legend)
    }
}
