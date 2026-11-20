//! Premium pets are unlocked with 20-character codes (XXXXX-XXXXX-XXXXX-XXXXX): a 40-bit serial plus
//! 60 bits of HMAC-SHA256, made by Scripts/make-codes.ps1 (or .swift). Same check as mac/main.swift.
//!
//! - An All Pets code signs `[1] + serial`. It unlocks every premium pet, including ones added later.
//!   (Every code made before per-pet codes existed is one of these.)
//! - A single-pet code signs `[2] + serial + pet id` (for example "lunette").
//!
//! The secret comes from build/code_secret.rs, which build.ps1 generates outside the repo.

use hmac::{Hmac, Mac};
use sha2::Sha256;

// Defines MASKED and MASK: the secret is MASKED xor MASK, so it never sits in the exe as one string.
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../build/code_secret.rs"));

const ALPHABET: &[u8] = b"0123456789ABCDEFGHJKMNPQRSTVWXYZ";

/// What a code unlocks.
#[derive(Clone, Copy, PartialEq)]
pub enum Grant {
    All,
    Pet(&'static str),
}

/// The 20 values of a code, ignoring dashes and spaces and accepting common misreadings
/// (O for 0, I or L for 1). None unless there are exactly 20.
fn values(code: &str) -> Option<Vec<u8>> {
    let mut values = Vec::with_capacity(20);
    for raw in code.to_ascii_uppercase().bytes() {
        let ch = match raw {
            b'O' => b'0',
            b'I' | b'L' => b'1',
            c => c,
        };
        if let Some(v) = ALPHABET.iter().position(|&a| a == ch) {
            values.push(v as u8);
        }
    }
    (values.len() == 20).then_some(values)
}

/// The code written the standard way (XXXXX-XXXXX-XXXXX-XXXXX), for storing and comparing.
pub fn normalized(code: &str) -> Option<String> {
    let v = values(code)?;
    let text: String = v.iter().map(|&i| ALPHABET[i as usize] as char).collect();
    Some([&text[0..5], &text[5..10], &text[10..15], &text[15..20]].join("-"))
}

/// What the code unlocks, or None if it isn't a real code. `pet_ids` are the premium pets in this
/// version of the app; a code for a pet it doesn't have yet unlocks nothing.
pub fn check(code: &str, pet_ids: &[&'static str]) -> Option<Grant> {
    let values = values(code)?;
    let bits: Vec<bool> = values.iter().flat_map(|&v| (0..5).map(move |i| v & (16 >> i) != 0)).collect();
    let serial: Vec<u8> = (0..5).map(|b| (0..8).fold(0u8, |acc, i| (acc << 1) | bits[b * 8 + i] as u8)).collect();
    let secret: Vec<u8> = MASKED.iter().zip(MASK.iter()).map(|(a, m)| a ^ m).collect();

    let signs = |kind: u8, id: &str| {
        let Ok(mut mac) = Hmac::<Sha256>::new_from_slice(&secret) else { return false };
        mac.update(&[kind]);
        mac.update(&serial);
        mac.update(id.as_bytes());
        let tag = mac.finalize().into_bytes();
        (0..60).all(|i| bits[40 + i] == (tag[i / 8] & (0x80 >> (i % 8)) != 0))
    };
    if signs(1, "") {
        return Some(Grant::All);
    }
    pet_ids.iter().find(|id| signs(2, id)).map(|id| Grant::Pet(id))
}

/// Everything the saved codes unlock.
#[derive(Default)]
pub struct Owned {
    pub all: bool,
    pub pets: Vec<&'static str>,
}

impl Owned {
    pub fn from_codes(codes: &[String], pet_ids: &[&'static str]) -> Owned {
        let mut owned = Owned::default();
        for grant in codes.iter().filter_map(|c| check(c, pet_ids)) {
            owned.add(grant);
        }
        owned
    }

    pub fn add(&mut self, grant: Grant) {
        match grant {
            Grant::All => self.all = true,
            Grant::Pet(id) if !self.pets.contains(&id) => self.pets.push(id),
            Grant::Pet(_) => {}
        }
    }

    /// Whether a premium pet with this id is unlocked.
    pub fn has(&self, id: &str) -> bool {
        self.all || self.pets.contains(&id)
    }
}
