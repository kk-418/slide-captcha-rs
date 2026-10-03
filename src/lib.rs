//! Slider CAPTCHA library for Rust.
//!
//! Supports basic sliding and drag-drop modes. Puzzle pieces can be generated
//! procedurally or supplied as custom graph images.

mod captcha;
mod draw;
mod error;
mod image_data;
mod jigsaw;
mod options;
mod resource;
mod types;
mod validate;

pub use captcha::{Builder, Captcha, CaptchaData, EncodedImage};
pub use error::Error;
pub use options::{
    OptionFn, with_enable_graph_vertical_random, with_gen_graph_number, with_image_alpha,
    with_image_size, with_range_dead_zone_directions, with_range_graph_angle_pos,
    with_range_graph_size,
};
pub use resource::{GraphImage, ResourceFn, with_backgrounds, with_graph_images};
pub use types::{Block, DeadZoneDirection, Mode, Point, RangeVal, Size};
pub use validate::validate;

/// Result type for this crate.
pub type Result<T> = std::result::Result<T, Error>;
