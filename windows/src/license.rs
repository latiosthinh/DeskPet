//! Ember, Pip, Puddle and Sprout are unlocked with a 20-character code (XXXXX-XXXXX-XXXXX-XXXXX):
//! a 40-bit serial plus 60 bits of HMAC-SHA256 over it, made by Scripts/make-codes.ps1 (or .swift).
//! The secret comes from build/code_secret.rs, which build.ps1 generates outside the repo. Same check as main.swift.

use hmac::{Hmac, Mac};
use sha2::Sha256;

// Defines MASKED and MASK: the secret is MASKED xor MASK, so it never sits in the exe as one string.
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../build/code_secret.rs"));

const ALPHABET: &[u8] = b"0123456789ABCDEFGHJKMNPQRSTVWXYZ";

pub fn is_valid(code: &str) -> bool {
    // Ignore dashes and spaces, and accept common misreadings (O for 0, I or L for 1).
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
    if values.len() != 20 {
        return false;
    }
    let bits: Vec<bool> = values.iter().flat_map(|&v| (0..5).map(move |i| v & (16 >> i) != 0)).collect();

    let mut message = [0u8; 6];
    message[0] = 1;
    for b in 0..5 {
        message[1 + b] = (0..8).fold(0u8, |acc, i| (acc << 1) | bits[b * 8 + i] as u8);
    }

    let secret: Vec<u8> = MASKED.iter().zip(MASK.iter()).map(|(a, m)| a ^ m).collect();
    let Ok(mut mac) = Hmac::<Sha256>::new_from_slice(&secret) else { return false };
    mac.update(&message);
    let tag = mac.finalize().into_bytes();
    (0..60).all(|i| bits[40 + i] == (tag[i / 8] & (0x80 >> (i % 8)) != 0))
}
