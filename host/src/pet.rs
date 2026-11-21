//! One pet on screen: its module, and drawing what it renders.
//! The pen (the pet's window) is `width`×`height` points; the pet moves inside it.
//! Coordinates are points with y pointing up from the floor, as on the Mac.

use crate::canvas::{Canvas, Font};
use crate::module::PetModule;
use petkit::art::{prop_palette, sys_color, Color, Frame, Palette, PET_SIZE, PIXEL};
use petkit::engine::{About, Command, Input, PictureFrame, Render};
use petkit::look::Pt;

pub struct PetView {
    module: PetModule,
    pub about: About,
    /// What the pet last rendered: where it is and what to draw.
    pub r: Render,
    /// Set when the pet's module failed (it panicked, ran too long or sent something unreadable);
    /// the app then swaps in another pet.
    pub failed: Option<String>,
}

impl PetView {
    pub fn new(bytes: &[u8], now: f64, seed: u64) -> Result<PetView, String> {
        let mut module = PetModule::load(bytes, now, seed)?;
        let (about, r) = (module.about()?, module.render()?);
        Ok(PetView { module, about, r, failed: None })
    }

    /// Something happened; returns true for a right-click the pet used for a trick.
    pub fn command(&mut self, c: Command) -> bool {
        if self.failed.is_some() {
            return false;
        }
        let used = self.module.command(c).unwrap_or_else(|e| {
            self.failed = Some(e);
            false
        });
        self.refresh();
        used
    }

    pub fn tick(&mut self, input: Input) {
        if self.failed.is_some() {
            return;
        }
        if let Err(e) = self.module.tick(input) {
            self.failed = Some(e);
        }
        self.refresh();
    }

    fn refresh(&mut self) {
        if self.failed.is_none() {
            match self.module.render() {
                Ok(r) => self.r = r,
                Err(e) => self.failed = Some(e),
            }
        }
    }

    pub fn say(&mut self, text: &str, seconds: f64) {
        self.command(Command::Say(text.to_string(), seconds));
    }

    // MARK: Drawing

    pub fn draw(&self, c: &mut Canvas) {
        let r = &self.r;
        if r.show_pen {
            self.draw_pen(c);
        }
        let look = &r.look;
        if let Some((origin, angle)) = look.beam {
            self.draw_beam(c, origin, angle);
        }

        let mut colors = prop_palette();
        colors.set('w', r.paw);
        colors.set('k', look.palette.get(b'k'));

        for f in &look.back {
            let (x, y) = (c.snap(f.x), c.snap(f.y));
            draw_grid(c, &f.rows, if f.own_colors { &look.palette } else { &colors }, x, 0.0, y, f.px, false);
        }
        if let Some(tw) = &look.twin {
            draw_sprite(c, &tw.frame, &look.palette, tw.center_x, tw.scale, tw.y_offset, tw.facing_left);
        }
        let shift = look.x_offset * PIXEL * if r.facing_left { -1.0 } else { 1.0 };
        let center = r.pet_x + PET_SIZE / 2.0 + shift;
        match &r.picture {
            Some(pic) => draw_picture(c, pic, &look.palette, center, look.scale, look.y_offset, r.facing_left),
            None => draw_sprite(c, &look.frame, &look.palette, center, look.scale, look.y_offset, r.facing_left),
        }
        let px0 = c.snap(r.pet_x + shift);
        for p in &look.props {
            draw_grid(c, &p.rows, &colors, px0, p.col + r.prop_shift, p.lift * PIXEL, PIXEL, r.facing_left);
        }
        for f in &look.free {
            let (x, y) = (c.snap(f.x), c.snap(f.y));
            draw_grid(c, &f.rows, if f.own_colors { &look.palette } else { &colors }, x, 0.0, y, f.px, false);
        }

        for p in &r.particles {
            let alpha = (p.life / p.max_life * 1.5).clamp(0.0, 1.0);
            c.text(p.text, Font::Symbol, p.size, p.color.with_alpha(alpha), p.x, p.y);
        }

        if let Some((text, icon)) = &r.bubble {
            self.draw_bubble(c, text, *icon);
        }
    }

    fn draw_beam(&self, c: &mut Canvas, p: Pt, angle: f64) {
        let facing_left = self.r.facing_left;
        let dir = if facing_left { -1.0 } else { 1.0 };
        let (length, spread) = (96.0, 0.34);
        let a = c.p(p.x, p.y);
        let b = c.p(p.x + dir * length * (angle - spread).cos(), p.y + length * (angle - spread).sin());
        let d = c.p(p.x + dir * length * (angle + spread).cos(), p.y + length * (angle + spread).sin());
        let light = Color::new(255, 237, 140, 128);
        c.fill_gradient_triangle(a, b, d, light, light.with_alpha(0.0), facing_left);
    }

    fn draw_pen(&self, c: &mut Canvas) {
        let r = &self.r;
        let (x, y) = (r.min_pet_x, 1.0);
        let (w, h) = (r.max_pet_x - r.min_pet_x + PET_SIZE, PET_SIZE - 4.0);
        c.fill_round_rect(x, y, w, h, 10.0, sys_color::TEAL.with_alpha(0.12));
        c.stroke_round_rect(x, y, w, h, 10.0, sys_color::TEAL.with_alpha(0.8), 2.0, Some((6.0, 4.0)));

        let label = if r.range == 0.0 { "stays put".to_string() } else { format!("roams ±{} px", r.range as i32) };
        let (lw, _) = c.measure(&label, Font::Ui, 10.0);
        c.text(&label, Font::Ui, 10.0, sys_color::TEAL, x + w / 2.0 - lw / 2.0, y + h + 2.0);
    }

    fn draw_bubble(&self, c: &mut Canvas, text: &str, icon: Option<&'static [&'static str]>) {
        let r = &self.r;
        let icon_px = 2.0;
        let (tw, th) = c.measure(text, Font::Ui, 12.0);
        let (iw, ih) = match icon {
            Some(rows) => (rows.iter().map(|r| r.len()).max().unwrap_or(0) as f64 * icon_px, rows.len() as f64 * icon_px),
            None => (0.0, 0.0),
        };
        let content_w = tw + if icon.is_some() { 5.0 + iw } else { 0.0 };
        let content_h = th.max(ih);
        let cx = r.pet_x + PET_SIZE / 2.0;

        let (bw, bh) = (content_w + 16.0, content_h + 8.0);
        let by = r.head * r.look.scale + 4.0; // a grown pet gets its speech bubble above its head
        let bx = (cx - content_w / 2.0 - 8.0).max(2.0).min(r.width - bw - 2.0);

        let white = Color::new(255, 255, 255, 255);
        c.fill_round_rect(bx, by, bw, bh, 8.0, white);
        let (a, b, d) = (c.p(cx - 5.0, by + 1.0), c.p(cx, by - 6.0), c.p(cx + 5.0, by + 1.0));
        c.fill_triangle(a, b, d, white);
        c.stroke_round_rect(bx, by, bw, bh, 8.0, Color::new(51, 51, 51, 255), 1.5, None);
        c.text(text, Font::Ui, 12.0, Color::new(0, 0, 0, 255), bx + 8.0, by + 4.0 + (content_h - th) / 2.0);
        if let Some(rows) = icon {
            let ix = c.snap(bx + 8.0 + tw + 5.0);
            let iy = c.snap(by + 4.0 + (content_h - ih) / 2.0);
            draw_grid(c, rows, &prop_palette(), ix, 0.0, iy, icon_px, false);
        }
    }
}

/// Draws the 16×16 pet sprite scaled around its centre, standing on the ground.
/// Art row 13 (feet) sits on the ground line, so the sprite starts 2 art pixels below it.
fn draw_sprite(c: &mut Canvas, rows: &Frame, palette: &Palette, center_x: f64, scale: f64, y_offset: f64, mirrored: bool) {
    let px = (PIXEL * scale * 2.0).round().max(1.0) / 2.0;
    let x = c.snap(center_x - 8.0 * px);
    let y = c.snap(-2.0 * px + y_offset * PIXEL);
    draw_grid(c, &rows[..], palette, x, 0.0, y, px, mirrored);
}

/// Draws a picture pet's frame scaled around its centre, its bottom row standing on the ground.
fn draw_picture(c: &mut Canvas, pic: &PictureFrame, palette: &Palette, center_x: f64, scale: f64, y_offset: f64, mirrored: bool) {
    let rows = &pic.rows;
    let px = (pic.px * scale * 2.0).round().max(1.0) / 2.0;
    let w = rows[0].len();
    let anchor = if mirrored { w as f64 - pic.anchor } else { pic.anchor };
    let x = c.snap(center_x - anchor * px);
    let bottom = c.snap(y_offset * PIXEL);
    let n = rows.len();
    for (r, row) in rows.iter().enumerate() {
        let y = bottom + (n - 1 - r) as f64 * px;
        for (i, &ch) in row.iter().enumerate() {
            let Some(color) = palette.get(ch) else {
                continue;
            };
            let col = if mirrored { w - 1 - i } else { i } as f64;
            c.fill_edges(x + col * px, y, x + (col + 1.0) * px, y + px, color);
        }
    }
}

/// Draws pixel rows whose left edge is sprite column `col` and whose bottom row sits at `bottom_y`.
/// Mirrored grids flip around the 16-column sprite, so props stay in front of a pet facing left.
#[allow(clippy::too_many_arguments)]
pub fn draw_grid<R: AsRef<[u8]>>(c: &mut Canvas, rows: &[R], palette: &Palette, x: f64, col: f64, bottom_y: f64, px: f64, mirrored: bool) {
    let n = rows.len();
    for (r, row) in rows.iter().enumerate() {
        let y = bottom_y + (n - 1 - r) as f64 * px;
        for (i, &ch) in row.as_ref().iter().enumerate() {
            let Some(color) = palette.get(ch) else {
                continue;
            };
            let sprite_col = col + i as f64;
            let draw_col = if mirrored { 15.0 - sprite_col } else { sprite_col };
            c.fill_edges(x + draw_col * px, y, x + (draw_col + 1.0) * px, y + px, color);
        }
    }
}
