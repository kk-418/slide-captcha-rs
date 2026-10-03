use image::{DynamicImage, Rgb, RgbImage, Rgba, RgbaImage};
use slide_captcha_rs::{
    Builder, Error, GraphImage, Size, validate, with_backgrounds, with_graph_images,
    with_image_size,
};

fn gradient_bg(width: u32, height: u32) -> DynamicImage {
    let mut img = RgbImage::new(width, height);
    for y in 0..height {
        for x in 0..width {
            let r = (x * 255 / width.max(1)) as u8;
            let g = (y * 255 / height.max(1)) as u8;
            img.put_pixel(x, y, Rgb([r, g, 128]));
        }
    }
    DynamicImage::ImageRgb8(img)
}

fn solid_rgba(width: u32, height: u32, color: Rgba<u8>) -> DynamicImage {
    let mut img = RgbaImage::new(width, height);
    for pixel in img.pixels_mut() {
        *pixel = color;
    }
    DynamicImage::ImageRgba8(img)
}

#[test]
fn generate_basic_with_procedural_jigsaw() {
    let size = Size {
        width: 300,
        height: 220,
    };
    let captcha = Builder::new()
        .set_options(with_image_size(size))
        .set_resources(with_backgrounds(vec![gradient_bg(300, 220)]))
        .make();

    let data = captcha.generate().expect("generate");
    let block = data.block();

    let master_jpeg = data.master_image().to_bytes().expect("master jpeg");
    let tile_png = data.tile_image().to_bytes().expect("tile png");
    assert!(!master_jpeg.is_empty());
    assert!(!tile_png.is_empty());

    let master = image::load_from_memory(&master_jpeg).expect("decode master");
    assert_eq!(master.width(), size.width);
    assert_eq!(master.height(), size.height);

    let tile = image::load_from_memory(&tile_png).expect("decode tile");
    assert!(tile.color().has_alpha());
    assert_eq!(tile.width(), block.width);
    assert_eq!(tile.height(), block.height);

    assert_eq!(block.dy, block.y);
    assert!(block.x >= 0);
    assert!((block.x as u32) < size.width);
    assert!(block.y >= 0);
    assert!((block.y as u32) < size.height);

    assert!(validate(block.x, block.y, block.x, block.y, 0));

    let b64 = data.master_image().to_base64().expect("master b64");
    assert!(b64.starts_with("data:image/jpeg;base64,"));
    let raw = data
        .master_image()
        .to_base64_data()
        .expect("master b64 data");
    assert!(!raw.is_empty());
}

#[test]
fn missing_background_returns_error() {
    let err = Builder::new().make().generate().expect_err("empty bg");
    assert!(matches!(err, Error::EmptyBackground));
}

#[test]
fn generate_drag_drop_sets_dx_dy() {
    let size = Size {
        width: 300,
        height: 220,
    };
    let captcha = Builder::new()
        .set_options(with_image_size(size))
        .set_resources(with_backgrounds(vec![gradient_bg(400, 300)]))
        .make_drag_drop();

    let data = captcha.generate().expect("drag-drop generate");
    let block = data.block();
    assert!(block.dx >= 0);
    assert!(block.dy >= 0);
    assert!((block.dx as u32) < size.width);
    assert!((block.dy as u32) < size.height);
    assert!(!data.master_image().to_bytes().unwrap().is_empty());
    assert!(!data.tile_image().to_bytes().unwrap().is_empty());
}

#[test]
fn generate_with_custom_graph_images() {
    let overlay = solid_rgba(16, 16, Rgba([255, 255, 255, 180]));
    let shadow = solid_rgba(16, 16, Rgba([0, 0, 0, 160]));
    let mask = solid_rgba(16, 16, Rgba([255, 255, 255, 255]));
    let graph = GraphImage::new(overlay, shadow, mask);

    let captcha = Builder::new()
        .set_resources([
            with_backgrounds(vec![gradient_bg(300, 220)]),
            with_graph_images(vec![graph]),
        ])
        .make();

    let data = captcha.generate().expect("custom graph generate");
    assert!(!data.master_image().to_bytes().unwrap().is_empty());
    assert!(!data.tile_image().to_bytes().unwrap().is_empty());
}

#[test]
fn empty_background_vec_is_error() {
    let err = Builder::new()
        .set_resources(with_backgrounds(vec![]))
        .make()
        .generate()
        .expect_err("empty vec");
    assert!(matches!(err, Error::EmptyBackground));
}

#[test]
fn invalid_graph_image_is_error() {
    let empty = DynamicImage::new_rgba8(0, 0);
    let graph = GraphImage::new(empty.clone(), empty.clone(), empty);
    let err = Builder::new()
        .set_resources([
            with_backgrounds(vec![gradient_bg(300, 220)]),
            with_graph_images(vec![graph]),
        ])
        .make()
        .generate()
        .expect_err("invalid graph");
    assert!(matches!(err, Error::InvalidGraphImage));
}
