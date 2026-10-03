use image::imageops::{self, FilterType};
use image::{DynamicImage, Rgba, RgbaImage};
use rand::Rng;

use crate::types::{Block, Size};
use crate::{Error, Result};

/// Random-crop if the background is larger; otherwise resize to `target`.
/// Returns `(master_with_holes, clean_background)`.
pub(crate) fn gen_master_image<R: Rng>(
    rng: &mut R,
    background: &DynamicImage,
    shadow: &DynamicImage,
    blocks: &[Block],
    target: Size,
    alpha: f32,
) -> Result<(DynamicImage, DynamicImage)> {
    if target.width == 0 || target.height == 0 {
        return Err(Error::GenerateFailed);
    }

    let clean = fit_background(rng, background, target);
    let mut master = clean.to_rgba8();
    for block in blocks {
        let shadow_rgba = scale_to(shadow, block.width, block.height);
        let shadow_rgba = apply_global_alpha(shadow_rgba, alpha);
        imageops::overlay(
            &mut master,
            &shadow_rgba,
            i64::from(block.x),
            i64::from(block.y),
        );
    }

    Ok((DynamicImage::ImageRgba8(master), clean))
}

/// Cuts the tile from `clean_bg` using `mask` alpha, then overlays `overlay`.
pub(crate) fn gen_tile_image(
    mask: &DynamicImage,
    clean_bg: &DynamicImage,
    overlay: &DynamicImage,
    block: &Block,
    alpha: f32,
) -> Result<DynamicImage> {
    if block.width == 0 || block.height == 0 {
        return Err(Error::GenerateFailed);
    }

    let mut tile = copy_region(
        &clean_bg.to_rgba8(),
        block.x,
        block.y,
        block.width,
        block.height,
    );
    let mask_rgba = scale_to(mask, block.width, block.height);
    apply_mask_alpha(&mut tile, &mask_rgba);

    let overlay_rgba = scale_to(overlay, block.width, block.height);
    let overlay_rgba = apply_global_alpha(overlay_rgba, alpha);
    imageops::overlay(&mut tile, &overlay_rgba, 0, 0);

    Ok(DynamicImage::ImageRgba8(tile))
}

fn fit_background<R: Rng>(rng: &mut R, bg: &DynamicImage, target: Size) -> DynamicImage {
    let (bw, bh) = (bg.width(), bg.height());
    let (tw, th) = (target.width, target.height);
    if bw == 0 || bh == 0 {
        return DynamicImage::new_rgba8(tw, th);
    }
    if bw >= tw && bh >= th {
        let x = rand_offset(rng, bw - tw);
        let y = rand_offset(rng, bh - th);
        return bg.crop_imm(x, y, tw, th);
    }
    bg.resize_exact(tw, th, FilterType::Triangle)
}

fn rand_offset<R: Rng>(rng: &mut R, max: u32) -> u32 {
    if max == 0 {
        0
    } else {
        rng.random_range(0..=max)
    }
}

fn scale_to(img: &DynamicImage, width: u32, height: u32) -> RgbaImage {
    if img.width() == width && img.height() == height {
        return img.to_rgba8();
    }
    img.resize_exact(width, height, FilterType::Triangle)
        .to_rgba8()
}

fn apply_global_alpha(mut img: RgbaImage, alpha: f32) -> RgbaImage {
    if alpha >= 1.0 {
        return img;
    }
    let factor = alpha.clamp(0.0, 1.0);
    for pixel in img.pixels_mut() {
        pixel.0[3] = (f32::from(pixel.0[3]) * factor).round() as u8;
    }
    img
}

fn apply_mask_alpha(tile: &mut RgbaImage, mask: &RgbaImage) {
    for (dest, src) in tile.pixels_mut().zip(mask.pixels()) {
        dest.0[3] = src.0[3];
    }
}

fn copy_region(src: &RgbaImage, x: i32, y: i32, width: u32, height: u32) -> RgbaImage {
    let mut out = RgbaImage::new(width, height);
    let sw = src.width();
    let sh = src.height();
    for dy in 0..height {
        for dx in 0..width {
            let sx = x.saturating_add_unsigned(dx);
            let sy = y.saturating_add_unsigned(dy);
            if sx < 0 || sy < 0 {
                continue;
            }
            let ux = sx as u32;
            let uy = sy as u32;
            if ux < sw && uy < sh {
                out.put_pixel(dx, dy, *src.get_pixel(ux, uy));
            } else {
                out.put_pixel(dx, dy, Rgba([0, 0, 0, 0]));
            }
        }
    }
    out
}
