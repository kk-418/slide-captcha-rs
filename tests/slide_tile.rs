use std::path::PathBuf;

use image::DynamicImage;
use slide_captcha_rs::{
    Builder, Captcha, GraphImage, validate, with_backgrounds, with_enable_graph_vertical_random,
    with_gen_graph_number, with_graph_images,
};

fn fixture(name: &str) -> DynamicImage {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(name);
    image::open(&path).unwrap_or_else(|e| panic!("load {}: {e}", path.display()))
}

fn graph_images() -> Vec<GraphImage> {
    vec![
        GraphImage::new(
            fixture("tile-1.png"),
            fixture("tile-shadow-1.png"),
            fixture("tile-mask-1.png"),
        ),
        GraphImage::new(
            fixture("tile-2.png"),
            fixture("tile-shadow-2.png"),
            fixture("tile-mask-2.png"),
        ),
    ]
}

fn backgrounds() -> Vec<DynamicImage> {
    vec![fixture("bg.jpg"), fixture("bg1.jpg")]
}

fn with_go_slide_resources(builder: Builder) -> Builder {
    builder.set_resources([
        with_graph_images(graph_images()),
        with_backgrounds(backgrounds()),
    ])
}

fn make_slide_tile() -> Captcha {
    with_go_slide_resources(Builder::new()).make()
}

/// Port of go-captcha `TestSlideTileCaptcha` (`v2/tests/slide_tile_test.go`).
#[test]
fn slide_tile_captcha() {
    let data = make_slide_tile().generate().expect("generate");
    let block = data.block();

    let json = serde_json::to_string(block).expect("serialize block");
    assert!(!json.is_empty());

    let master_b64 = data.master_image().to_base64().expect("master base64");
    assert!(master_b64.starts_with("data:image/jpeg;base64,"));
    assert!(master_b64.len() > "data:image/jpeg;base64,".len());

    let tile_b64 = data.tile_image().to_base64().expect("tile base64");
    assert!(tile_b64.starts_with("data:image/png;base64,"));
    assert!(tile_b64.len() > "data:image/png;base64,".len());

    let out_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("temp")
        .join("slide_tile");
    std::fs::create_dir_all(&out_dir).expect("create temp dir");
    let master_path = out_dir.join("master.jpg");
    let tile_path = out_dir.join("thumb.png");
    data.master_image().save(&master_path).expect("save master");
    data.tile_image().save(&tile_path).expect("save tile");
    assert!(master_path.is_file());
    assert!(tile_path.is_file());
    assert!(std::fs::metadata(&master_path).expect("master meta").len() > 0);
    assert!(std::fs::metadata(&tile_path).expect("tile meta").len() > 0);

    let master_jpeg = data.master_image().to_bytes().expect("master jpeg");
    let master = image::load_from_memory(&master_jpeg).expect("decode master");
    assert_eq!(master.width(), 300);
    assert_eq!(master.height(), 220);

    let tile_png = data.tile_image().to_bytes().expect("tile png");
    let tile = image::load_from_memory(&tile_png).expect("decode tile");
    assert_eq!(tile.width(), block.width);
    assert_eq!(tile.height(), block.height);

    assert!(validate(block.x, block.y, block.x, block.y, 0));
    assert_eq!(block.dy, block.y);
}

/// Port of the commented Go `MakeWithRegion` / `MakeDragDrop` path.
#[test]
fn slide_tile_drag_drop() {
    let data = with_go_slide_resources(Builder::new())
        .make_drag_drop()
        .generate()
        .expect("drag-drop generate");
    let block = data.block();

    assert!(block.dx >= 0);
    assert!(block.dy >= 0);
    assert!((block.dx as u32) < 300);
    assert!((block.dy as u32) < 220);
    assert!(!data.master_image().to_bytes().unwrap().is_empty());
    assert!(!data.tile_image().to_bytes().unwrap().is_empty());
}

/// Port of the commented Go `WithGenGraphNumber(2)` + `WithEnableGraphVerticalRandom(true)`.
#[test]
fn slide_tile_gen_graph_number_and_vertical_random() {
    let data = with_go_slide_resources(Builder::new())
        .set_options([
            with_gen_graph_number(2),
            with_enable_graph_vertical_random(true),
        ])
        .make_drag_drop()
        .generate()
        .expect("gen-graph-number generate");
    let block = data.block();
    assert!(block.width > 0);
    assert!(block.height > 0);
    assert!(!data.master_image().to_bytes().unwrap().is_empty());
    assert!(!data.tile_image().to_bytes().unwrap().is_empty());
}
