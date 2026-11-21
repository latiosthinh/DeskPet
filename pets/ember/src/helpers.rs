//! Helpers shared by its tricks.

use crate::*;

pub fn firework(cx: &mut Stage, p: Pt) {
    for i in 0..16 {
        let a = i as f64 / 16.0 * 2.0 * PI;
        let speed = rand(34.0, 52.0);
        cx.particles.push(Particle {
            text: if i % 2 == 0 { "✦" } else { "•" },
            color: sys_color::SPARKLES[rand_index(6)],
            size: 11.0,
            x: p.x - 4.0,
            y: p.y,
            vx: a.cos() * speed,
            vy: a.sin() * speed,
            life: 1.3,
            max_life: 1.3,
            ay: -24.0,
        });
    }
}
