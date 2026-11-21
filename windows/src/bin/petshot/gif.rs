//! A small animated GIF writer for pixel art: one shared palette, each frame stored as just the
//! rectangle that changed (unchanged pixels inside it see-through), looping forever.

use std::collections::HashMap;

/// Frames of opaque 0xRRGGBB pixels, all `w`×`h`, each shown for `delay` hundredths of a second.
pub struct Gif {
    pub w: usize,
    pub h: usize,
    pub frames: Vec<(Vec<u32>, u16)>,
}

const SEE_THROUGH: u8 = 255;

impl Gif {
    pub fn encode(&self) -> Vec<u8> {
        let (palette, index) = palette(&self.frames);
        let mut out = Vec::new();
        out.extend_from_slice(b"GIF89a");
        le16(&mut out, self.w as u16);
        le16(&mut out, self.h as u16);
        out.extend_from_slice(&[0xF7, 0, 0]); // global colour table of 256 entries
        for i in 0..256 {
            let c = palette.get(i).copied().unwrap_or(0);
            out.extend_from_slice(&[(c >> 16) as u8, (c >> 8) as u8, c as u8]);
        }
        out.extend_from_slice(&[0x21, 0xFF, 11]);
        out.extend_from_slice(b"NETSCAPE2.0");
        out.extend_from_slice(&[3, 1, 0, 0, 0]); // loop forever

        let mut prev: Option<&Vec<u32>> = None;
        for (pixels, delay) in &self.frames {
            // The rectangle that changed since the last frame (the whole picture the first time).
            let (mut x0, mut y0, mut x1, mut y1) = (0, 0, self.w, self.h);
            if let Some(p) = prev {
                let changed = |i: usize| pixels[i] != p[i];
                let rows: Vec<usize> = (0..self.h).filter(|&y| (0..self.w).any(|x| changed(y * self.w + x))).collect();
                let cols: Vec<usize> = (0..self.w).filter(|&x| rows.iter().any(|&y| changed(y * self.w + x))).collect();
                match (rows.first(), rows.last(), cols.first(), cols.last()) {
                    (Some(&t), Some(&b), Some(&l), Some(&r)) => (x0, y0, x1, y1) = (l, t, r + 1, b + 1),
                    _ => (x0, y0, x1, y1) = (0, 0, 1, 1), // nothing changed: one see-through pixel
                }
            }
            let mut indices = Vec::with_capacity((x1 - x0) * (y1 - y0));
            for y in y0..y1 {
                for x in x0..x1 {
                    let i = y * self.w + x;
                    indices.push(if prev.is_some_and(|p| p[i] == pixels[i]) { SEE_THROUGH } else { index[&pixels[i]] });
                }
            }
            // Graphic control: keep the previous frame under this one, index 255 is see-through.
            out.extend_from_slice(&[0x21, 0xF9, 4, 0x05]);
            le16(&mut out, *delay);
            out.extend_from_slice(&[SEE_THROUGH, 0]);
            out.push(0x2C);
            for v in [x0, y0, x1 - x0, y1 - y0] {
                le16(&mut out, v as u16);
            }
            out.push(0);
            out.push(8); // minimum code size
            for chunk in lzw(&indices).chunks(255) {
                out.push(chunk.len() as u8);
                out.extend_from_slice(chunk);
            }
            out.push(0);
            prev = Some(pixels);
        }
        out.push(0x3B);
        out
    }
}

fn le16(out: &mut Vec<u8>, v: u16) {
    out.extend_from_slice(&v.to_le_bytes());
}

/// Up to 255 colours (index 255 is see-through) and each colour's index. Pixel art rarely has more;
/// if it does, the rarest colours are mapped to the nearest kept one.
fn palette(frames: &[(Vec<u32>, u16)]) -> (Vec<u32>, HashMap<u32, u8>) {
    let mut counts: HashMap<u32, usize> = HashMap::new();
    for (pixels, _) in frames {
        for &p in pixels {
            *counts.entry(p).or_default() += 1;
        }
    }
    let mut colors: Vec<(u32, usize)> = counts.into_iter().collect();
    colors.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
    let kept: Vec<u32> = colors.iter().take(255).map(|c| c.0).collect();
    let mut index: HashMap<u32, u8> = kept.iter().enumerate().map(|(i, &c)| (c, i as u8)).collect();
    let dist = |a: u32, b: u32| {
        let d = |s: u32| ((a >> s & 0xFF) as i32 - (b >> s & 0xFF) as i32).pow(2);
        d(16) + d(8) + d(0)
    };
    for &(c, _) in colors.iter().skip(255) {
        let nearest = (0..kept.len()).min_by_key(|&i| dist(c, kept[i])).unwrap();
        index.insert(c, nearest as u8);
    }
    (kept, index)
}

/// GIF's LZW compression of 8-bit indices.
fn lzw(data: &[u8]) -> Vec<u8> {
    const CLEAR: u16 = 256;
    const END: u16 = 257;
    let mut out = Vec::new();
    let (mut bits, mut nbits) = (0u32, 0u32);
    let mut emit = |code: u16, size: u32, out: &mut Vec<u8>| {
        bits |= (code as u32) << nbits;
        nbits += size;
        while nbits >= 8 {
            out.push(bits as u8);
            bits >>= 8;
            nbits -= 8;
        }
    };
    let mut table: HashMap<(u16, u8), u16> = HashMap::new();
    let mut next = 258u16;
    let mut size = 9u32;
    emit(CLEAR, size, &mut out);
    let mut current: Option<u16> = None;
    for &b in data {
        let Some(c) = current else {
            current = Some(b as u16);
            continue;
        };
        if let Some(&code) = table.get(&(c, b)) {
            current = Some(code);
            continue;
        }
        emit(c, size, &mut out);
        if next == 4096 {
            emit(CLEAR, size, &mut out);
            table.clear();
            next = 258;
            size = 9;
        } else {
            table.insert((c, b), next);
            if next == 1 << size && size < 12 {
                size += 1;
            }
            next += 1;
        }
        current = Some(b as u16);
    }
    if let Some(c) = current {
        emit(c, size, &mut out);
    }
    emit(END, size, &mut out);
    if nbits > 0 {
        out.push(bits as u8);
    }
    out
}
