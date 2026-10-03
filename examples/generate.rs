use std::fs;

use image::{DynamicImage, Rgb, RgbImage};
use slide_captcha_rs::{Builder, Size, with_backgrounds, with_image_size};

fn colorful_bg(width: u32, height: u32) -> DynamicImage {
    let mut img = RgbImage::new(width, height);
    for y in 0..height {
        for x in 0..width {
            let r = (x * 255 / width.max(1)) as u8;
            let g = (y * 255 / height.max(1)) as u8;
            let b = (((x + y) * 180) / (width + height).max(1)) as u8;
            img.put_pixel(x, y, Rgb([r, g, b.saturating_add(40)]));
        }
    }
    DynamicImage::ImageRgb8(img)
}

fn main() {
    let captcha = Builder::new()
        .set_options(with_image_size(Size {
            width: 300,
            height: 220,
        }))
        .set_resources(with_backgrounds(vec![colorful_bg(400, 300)]))
        .make();

    let data = captcha.generate().unwrap();
    fs::create_dir_all("temp").unwrap();
    data.master_image().save("temp/master.jpg").unwrap();
    data.tile_image().save("temp/tile.png").unwrap();

    let block = data.block();
    println!("saved temp/master.jpg and temp/tile.png");
    println!(
        "block x={} y={} w={} h={} dx={} dy={}",
        block.x, block.y, block.width, block.height, block.dx, block.dy
    );
}
