//! Random numbers (xorshift; nothing here needs more). Seeded by the app.

use std::cell::Cell;

thread_local! {
    static SEED: Cell<u64> = const { Cell::new(0x2545_f491_4f6c_dd1d) };
}

/// Seeds the generator (the app calls this once with the time).
pub fn seed(s: u64) {
    SEED.with(|c| c.set(s | 1));
}

pub fn rand01() -> f64 {
    SEED.with(|s| {
        let mut x = s.get();
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        s.set(x);
        (x >> 11) as f64 / (1u64 << 53) as f64
    })
}

pub fn rand(lo: f64, hi: f64) -> f64 {
    lo + rand01() * (hi - lo)
}

pub fn rand_index(n: usize) -> usize {
    ((rand01() * n as f64) as usize).min(n.saturating_sub(1))
}
