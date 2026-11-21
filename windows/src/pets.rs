//! The pets this build knows. For now every pet is linked into the app; later only the built-in
//! three are, and the rest are installed as modules.

use petkit::engine::{About, Brain};
use petkit::pet::PetDef;

pub struct Entry {
    pub about: About,
    pub make: fn() -> PetDef,
}

pub fn catalog() -> Vec<Entry> {
    let makers: [fn() -> PetDef; 15] = [
        pet_mochi::pet,
        pet_ember::pet,
        pet_pip::pet,
        pet_puddle::pet,
        pet_sprout::pet,
        pet_grumbo::pet,
        pet_wispaling::pet,
        pet_hexley::pet,
        pet_mossback::pet,
        pet_sirclank::pet,
        pet_lunette::pet,
        pet_ninja::pet,
        pet_samurai::pet,
        pet_moona::pet,
        pet_witcher::pet,
    ];
    makers.into_iter().map(|make| Entry { about: Brain::new(make(), 0.0).about(), make }).collect()
}
