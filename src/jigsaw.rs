use image::{Rgba, RgbaImage};

/// Procedural slider jigsaw: overlay (rim), shadow (hole), and mask (alpha).
pub(crate) struct Jigsaw {
    pub overlay: RgbaImage,
    pub shadow: RgbaImage,
    pub mask: RgbaImage,
}

/// Generates overlay, shadow, and mask of size `width` × `height`.
///
/// Shape: rounded rectangle with a circular tab on the right, plus a 2px
/// transparent margin so edges are not clipped.
pub(crate) fn generate(width: u32, height: u32) -> Jigsaw {
    let mut overlay = RgbaImage::new(width, height);
    let mut shadow = RgbaImage::new(width, height);
    let mut mask = RgbaImage::new(width, height);
    if width == 0 || height == 0 {
        return Jigsaw {
            overlay,
            shadow,
            mask,
        };
    }

    let geom = ShapeGeom::new(width, height);
    for y in 0..height {
        for x in 0..width {
            let px = x as f32 + 0.5;
            let py = y as f32 + 0.5;
            let sdf = geom.sdf(px, py);
            let cover = (0.5 - sdf).clamp(0.0, 1.0);
            if cover <= 0.0 {
                continue;
            }

            let cover_u8 = (cover * 255.0).round() as u8;
            mask.put_pixel(x, y, Rgba([255, 255, 255, cover_u8]));
            shadow.put_pixel(x, y, Rgba([0, 0, 0, (cover * 160.0).round() as u8]));

            let dist_inside = -sdf;
            if dist_inside < 2.0 {
                let edge = cover * (200.0 / 255.0);
                overlay.put_pixel(x, y, Rgba([255, 255, 255, (edge * 255.0).round() as u8]));
            }
        }
    }

    Jigsaw {
        overlay,
        shadow,
        mask,
    }
}

struct ShapeGeom {
    body_left: f32,
    body_top: f32,
    body_right: f32,
    body_bottom: f32,
    corner_r: f32,
    tab_cx: f32,
    tab_cy: f32,
    tab_r: f32,
}

impl ShapeGeom {
    fn new(width: u32, height: u32) -> Self {
        let w = width as f32;
        let h = height as f32;
        let min_dim = w.min(h);
        let margin = 2.0_f32.min(min_dim / 8.0).max(0.0);
        let tab_r = (0.18 * min_dim).clamp(1.0, (min_dim - margin * 2.0) / 3.0);
        let body_left = margin;
        let body_top = margin;
        let body_right = (w - margin - tab_r).max(body_left + 1.0);
        let body_bottom = (h - margin).max(body_top + 1.0);
        let body_w = (body_right - body_left).max(1.0);
        let body_h = (body_bottom - body_top).max(1.0);
        let corner_r = (body_w.min(body_h) * 0.12).clamp(1.0, 8.0);
        Self {
            body_left,
            body_top,
            body_right,
            body_bottom,
            corner_r,
            tab_cx: body_right,
            tab_cy: h * 0.5,
            tab_r,
        }
    }

    fn sdf(&self, px: f32, py: f32) -> f32 {
        let rect = rounded_rect_sdf(
            px,
            py,
            self.body_left,
            self.body_top,
            self.body_right,
            self.body_bottom,
            self.corner_r,
        );
        let tab = circle_sdf(px, py, self.tab_cx, self.tab_cy, self.tab_r);
        rect.min(tab)
    }
}

fn circle_sdf(px: f32, py: f32, cx: f32, cy: f32, radius: f32) -> f32 {
    let dx = px - cx;
    let dy = py - cy;
    dx.hypot(dy) - radius
}

fn rounded_rect_sdf(
    px: f32,
    py: f32,
    left: f32,
    top: f32,
    right: f32,
    bottom: f32,
    radius: f32,
) -> f32 {
    let cx = (left + right) * 0.5;
    let cy = (top + bottom) * 0.5;
    let hx = ((right - left) * 0.5 - radius).max(0.0);
    let hy = ((bottom - top) * 0.5 - radius).max(0.0);
    let dx = (px - cx).abs() - hx;
    let dy = (py - cy).abs() - hy;
    let outside = dx.max(0.0).hypot(dy.max(0.0));
    let inside = dx.max(dy).min(0.0);
    outside + inside - radius
}

#[cfg(test)]
mod tests {
    use super::generate;

    #[test]
    fn jigsaw_has_opaque_and_transparent_mask() {
        let jigsaw = generate(64, 64);
        assert_eq!(jigsaw.mask.width(), 64);
        assert_eq!(jigsaw.mask.height(), 64);
        let mut opaque = 0u32;
        let mut transparent = 0u32;
        for p in jigsaw.mask.pixels() {
            if p.0[3] > 200 {
                opaque += 1;
            }
            if p.0[3] == 0 {
                transparent += 1;
            }
        }
        assert!(opaque > 100);
        assert!(transparent > 100);

        let mut rim = 0u32;
        for p in jigsaw.overlay.pixels() {
            if p.0[3] > 0 {
                rim += 1;
            }
        }
        assert!(rim > 50);
    }
}
