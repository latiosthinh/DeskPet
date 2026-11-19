//! Drawing into a 32 bpp premultiplied BGRA buffer, in the Mac version's coordinates:
//! points (DIPs) with y pointing up from the bottom of the pen. Everything is converted to device
//! pixels here; rectangle edges are rounded to whole pixels so art pixels never leave seams.
//! Shapes are antialiased in software. Text is rendered once by GDI into a coverage mask and cached.

use crate::art::Color;
use crate::sys;
use std::collections::HashMap;

pub struct Canvas<'a> {
    pub buf: &'a mut [u32],
    pub w: i32,
    pub h: i32,
    pub s: f64, // device pixels per point
    pub text: &'a mut TextCache,
}

#[inline]
fn premul(c: Color, cov: f64) -> (u32, u32, u32, u32) {
    let a = (c.a as f64 * cov).round().clamp(0.0, 255.0) as u32;
    let m = |v: u8| (v as u32 * a + 127) / 255;
    (m(c.r), m(c.g), m(c.b), a)
}

#[inline]
fn blend(dst: &mut u32, (r, g, b, a): (u32, u32, u32, u32)) {
    if a == 0 {
        return;
    }
    if a == 255 {
        *dst = 0xFF00_0000 | (r << 16) | (g << 8) | b;
        return;
    }
    let d = *dst;
    let inv = 255 - a;
    let ch = |shift: u32, src: u32| -> u32 { src + (((d >> shift) & 0xFF) * inv + 127) / 255 };
    *dst = (ch(24, a) << 24) | (ch(16, r) << 16) | (ch(8, g) << 8) | ch(0, b);
}

impl<'a> Canvas<'a> {
    pub fn clear(&mut self) {
        self.buf.fill(0);
    }

    pub fn snap(&self, v: f64) -> f64 {
        (v * self.s).round() / self.s
    }
    fn dx(&self, x: f64) -> i32 {
        (x * self.s).round() as i32
    }
    fn dy(&self, y: f64) -> i32 {
        self.h - (y * self.s).round() as i32
    }
    /// A point in device pixels (y down), not rounded.
    pub fn p(&self, x: f64, y: f64) -> (f64, f64) {
        (x * self.s, self.h as f64 - y * self.s)
    }

    #[inline]
    fn put(&mut self, x: i32, y: i32, px: (u32, u32, u32, u32)) {
        if x >= 0 && y >= 0 && x < self.w && y < self.h {
            blend(&mut self.buf[(y * self.w + x) as usize], px);
        }
    }

    /// Fills the rectangle between two corners, each edge rounded to the nearest device pixel.
    pub fn fill_edges(&mut self, x0: f64, y0: f64, x1: f64, y1: f64, c: Color) {
        let (l, r) = (self.dx(x0).max(0), self.dx(x1).min(self.w));
        let (t, b) = (self.dy(y1).max(0), self.dy(y0).min(self.h));
        if r <= l || b <= t {
            return;
        }
        let px = premul(c, 1.0);
        for y in t..b {
            let row = &mut self.buf[(y * self.w) as usize..((y + 1) * self.w) as usize];
            for d in &mut row[l as usize..r as usize] {
                blend(d, px);
            }
        }
    }

    pub fn fill_rect(&mut self, x: f64, y: f64, w: f64, h: f64, c: Color) {
        self.fill_edges(x, y, x + w, y + h, c);
    }

    /// Signed distance (device pixels) from a pixel centre to a rounded rectangle given in device pixels.
    fn round_rect_sdf(px: f64, py: f64, l: f64, t: f64, r: f64, b: f64, rad: f64) -> f64 {
        let (cx, cy) = ((l + r) / 2.0, (t + b) / 2.0);
        let (hx, hy) = ((r - l) / 2.0 - rad, (b - t) / 2.0 - rad);
        let qx = (px - cx).abs() - hx;
        let qy = (py - cy).abs() - hy;
        let outside = (qx.max(0.0).powi(2) + qy.max(0.0).powi(2)).sqrt();
        outside + qx.max(qy).min(0.0) - rad
    }

    /// The rounded rectangle (points, y up) as device-pixel edges (left, top, right, bottom).
    fn device_rect(&self, x: f64, y: f64, w: f64, h: f64) -> (f64, f64, f64, f64) {
        (x * self.s, self.h as f64 - (y + h) * self.s, (x + w) * self.s, self.h as f64 - y * self.s)
    }

    pub fn fill_round_rect(&mut self, x: f64, y: f64, w: f64, h: f64, radius: f64, c: Color) {
        let (l, t, r, b) = self.device_rect(x, y, w, h);
        let rad = (radius * self.s).min((r - l) / 2.0).min((b - t) / 2.0);
        for py in (t.floor() as i32).max(0)..(b.ceil() as i32).min(self.h) {
            for px in (l.floor() as i32).max(0)..(r.ceil() as i32).min(self.w) {
                let d = Self::round_rect_sdf(px as f64 + 0.5, py as f64 + 0.5, l, t, r, b, rad);
                let cov = (0.5 - d).clamp(0.0, 1.0);
                if cov > 0.0 {
                    self.put(px, py, premul(c, cov));
                }
            }
        }
    }

    /// Strokes a rounded rectangle; `dash` is (on, off) lengths in points.
    pub fn stroke_round_rect(&mut self, x: f64, y: f64, w: f64, h: f64, radius: f64, c: Color, width: f64, dash: Option<(f64, f64)>) {
        let (l, t, r, b) = self.device_rect(x, y, w, h);
        let rad = (radius * self.s).min((r - l) / 2.0).min((b - t) / 2.0);
        let half = width * self.s / 2.0;
        // Walk the outline clockwise from the start of the top edge, stamping antialiased discs.
        let straight_w = (r - l - 2.0 * rad).max(0.0);
        let straight_h = (b - t - 2.0 * rad).max(0.0);
        let arc = std::f64::consts::FRAC_PI_2 * rad;
        let total = 2.0 * (straight_w + straight_h) + 4.0 * arc;
        let point = |d: f64| -> (f64, f64) {
            let mut d = d;
            let corner = |cx: f64, cy: f64, start: f64, d: f64| {
                let a = start + if rad > 0.0 { d / rad } else { 0.0 };
                (cx + rad * a.cos(), cy + rad * a.sin())
            };
            if d < straight_w { return (l + rad + d, t); }
            d -= straight_w;
            if d < arc { return corner(r - rad, t + rad, -std::f64::consts::FRAC_PI_2, d); }
            d -= arc;
            if d < straight_h { return (r, t + rad + d); }
            d -= straight_h;
            if d < arc { return corner(r - rad, b - rad, 0.0, d); }
            d -= arc;
            if d < straight_w { return (r - rad - d, b); }
            d -= straight_w;
            if d < arc { return corner(l + rad, b - rad, std::f64::consts::FRAC_PI_2, d); }
            d -= arc;
            if d < straight_h { return (l, b - rad - d); }
            d -= straight_h;
            corner(l + rad, t + rad, std::f64::consts::PI, d)
        };

        let (bx0, by0) = ((l - half - 1.0).floor().max(0.0) as i32, (t - half - 1.0).floor().max(0.0) as i32);
        let (bx1, by1) = (((r + half + 2.0).ceil() as i32).min(self.w), ((b + half + 2.0).ceil() as i32).min(self.h));
        let (bw, bh) = (bx1 - bx0, by1 - by0);
        if bw <= 0 || bh <= 0 {
            return;
        }
        let mut cov = vec![0f32; (bw * bh) as usize];
        let step = 0.5;
        let mut d = 0.0;
        while d < total {
            let on = match dash {
                Some((a, gap)) => (d / self.s) % (a + gap) < a,
                None => true,
            };
            if on {
                let (sx, sy) = point(d);
                let reach = half + 1.0;
                for py in ((sy - reach).floor() as i32).max(by0)..((sy + reach).ceil() as i32).min(by1) {
                    for px in ((sx - reach).floor() as i32).max(bx0)..((sx + reach).ceil() as i32).min(bx1) {
                        let dist = ((px as f64 + 0.5 - sx).powi(2) + (py as f64 + 0.5 - sy).powi(2)).sqrt();
                        let v = (half + 0.5 - dist).clamp(0.0, 1.0) as f32;
                        let cell = &mut cov[((py - by0) * bw + (px - bx0)) as usize];
                        if v > *cell {
                            *cell = v;
                        }
                    }
                }
            }
            d += step;
        }
        for py in 0..bh {
            for px in 0..bw {
                let v = cov[(py * bw + px) as usize];
                if v > 0.0 {
                    self.put(px + bx0, py + by0, premul(c, v as f64));
                }
            }
        }
    }

    /// Fills a triangle (device pixels), 4×4 supersampled. `shade` gives the colour at a device x.
    fn fill_triangle_with(&mut self, a: (f64, f64), b: (f64, f64), c: (f64, f64), shade: impl Fn(f64) -> Color) {
        let edge = |p: (f64, f64), q: (f64, f64), x: f64, y: f64| (q.0 - p.0) * (y - p.1) - (q.1 - p.1) * (x - p.0);
        let area = edge(a, b, c.0, c.1);
        if area.abs() < 1e-9 {
            return;
        }
        let x0 = (a.0.min(b.0).min(c.0).floor() as i32).max(0);
        let x1 = (a.0.max(b.0).max(c.0).ceil() as i32).min(self.w);
        let y0 = (a.1.min(b.1).min(c.1).floor() as i32).max(0);
        let y1 = (a.1.max(b.1).max(c.1).ceil() as i32).min(self.h);
        for py in y0..y1 {
            for px in x0..x1 {
                let mut hits = 0;
                for sy in 0..4 {
                    for sx in 0..4 {
                        let (x, y) = (px as f64 + (sx as f64 + 0.5) / 4.0, py as f64 + (sy as f64 + 0.5) / 4.0);
                        let (e0, e1, e2) = (edge(a, b, x, y), edge(b, c, x, y), edge(c, a, x, y));
                        if (e0 >= 0.0 && e1 >= 0.0 && e2 >= 0.0) || (e0 <= 0.0 && e1 <= 0.0 && e2 <= 0.0) {
                            hits += 1;
                        }
                    }
                }
                if hits > 0 {
                    let col = shade(px as f64 + 0.5);
                    self.put(px, py, premul(col, hits as f64 / 16.0));
                }
            }
        }
    }

    pub fn fill_triangle(&mut self, a: (f64, f64), b: (f64, f64), c: (f64, f64), col: Color) {
        self.fill_triangle_with(a, b, c, |_| col);
    }

    /// Light fading out across the triangle, from `start` at the near side to `end` at the far side.
    pub fn fill_gradient_triangle(&mut self, a: (f64, f64), b: (f64, f64), c: (f64, f64), start: Color, end: Color, reversed: bool) {
        let lo = a.0.min(b.0).min(c.0) - 1.0;
        let hi = a.0.max(b.0).max(c.0) + 1.0;
        self.fill_triangle_with(a, b, c, |x| {
            let mut t = ((x - lo) / (hi - lo)).clamp(0.0, 1.0);
            if reversed {
                t = 1.0 - t;
            }
            let mix = |p: u8, q: u8| (p as f64 + (q as f64 - p as f64) * t).round() as u8;
            Color::new(mix(start.r, end.r), mix(start.g, end.g), mix(start.b, end.b), mix(start.a, end.a))
        });
    }

    /// Size of a line of text, in points.
    pub fn measure(&mut self, text: &str, font: Font, size_pt: f64) -> (f64, f64) {
        let m = self.text.mask(text, font, (size_pt * self.s).round() as i32);
        (m.w as f64 / self.s, m.h as f64 / self.s)
    }

    /// Draws text whose line box has its bottom-left corner at (x, y), like NSAttributedString.draw(at:).
    pub fn text(&mut self, text: &str, font: Font, size_pt: f64, c: Color, x: f64, y: f64) {
        let left = self.dx(x);
        let bottom = self.dy(y);
        let (cw, ch) = (self.w, self.h);
        let m = self.text.mask(text, font, (size_pt * self.s).round() as i32);
        let top = bottom - m.h;
        for my in 0..m.h {
            let y = top + my;
            if y < 0 || y >= ch {
                continue;
            }
            for mx in 0..m.w {
                let x = left + mx;
                let v = m.cov[(my * m.w + mx) as usize];
                if v > 0 && x >= 0 && x < cw {
                    blend(&mut self.buf[(y * cw + x) as usize], premul(c, v as f64 / 255.0));
                }
            }
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum Font {
    /// Segoe UI Symbol, bold: particle glyphs.
    Symbol,
    /// Segoe UI Semibold: speech bubbles and labels.
    Ui,
}

pub struct Mask {
    pub w: i32,
    pub h: i32,
    pub cov: Vec<u8>,
}

/// Renders text with GDI (white on black, greyscale antialiasing) and keeps the result as an
/// alpha mask, so each string is only rasterised once.
pub struct TextCache {
    masks: HashMap<(String, Font, i32), Mask>,
    fonts: HashMap<(Font, i32), sys::Font>,
    scratch: Option<sys::Dib>,
}

impl TextCache {
    pub fn new() -> TextCache {
        TextCache { masks: HashMap::new(), fonts: HashMap::new(), scratch: None }
    }

    pub fn mask(&mut self, text: &str, font: Font, px: i32) -> &Mask {
        let key = (text.to_string(), font, px);
        if !self.masks.contains_key(&key) {
            if self.masks.len() > 256 {
                self.masks.clear();
            }
            let m = self.render(text, font, px.max(1));
            self.masks.insert(key.clone(), m);
        }
        &self.masks[&key]
    }

    fn render(&mut self, text: &str, font: Font, px: i32) -> Mask {
        let f = self
            .fonts
            .entry((font, px))
            .or_insert_with(|| match font {
                Font::Symbol => sys::Font::new("Segoe UI Symbol", px, 700),
                Font::Ui => sys::Font::new("Segoe UI", px, 600),
            })
            .handle();
        let (w, h) = sys::text_extent(f, text);
        let (w, h) = (w.max(1), h.max(1));
        let need_new = match &self.scratch {
            Some(d) => d.w < w || d.h < h,
            None => true,
        };
        if need_new {
            self.scratch = Some(sys::Dib::new(w.max(64), h.max(32)));
        }
        let dib = self.scratch.as_mut().unwrap();
        let stride = dib.w;
        let pixels = dib.draw_text(f, text, w, h);
        let mut cov = vec![0u8; (w * h) as usize];
        for y in 0..h {
            for x in 0..w {
                let p = pixels[(y * stride + x) as usize];
                let (r, g, b) = ((p >> 16) & 0xFF, (p >> 8) & 0xFF, p & 0xFF);
                cov[(y * w + x) as usize] = r.max(g).max(b) as u8;
            }
        }
        Mask { w, h, cov }
    }
}
