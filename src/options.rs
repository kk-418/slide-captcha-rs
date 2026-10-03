use std::iter::Once;

use crate::types::{DeadZoneDirection, RangeVal, Size};

/// Internal generation options.
#[derive(Debug, Clone)]
pub(crate) struct Options {
    pub image_size: Size,
    pub image_alpha: f32,
    pub range_graph_size: RangeVal,
    pub range_graph_angle_pos: Vec<RangeVal>,
    pub gen_graph_number: u32,
    pub enable_graph_vertical_random: bool,
    pub range_dead_zone_directions: Vec<DeadZoneDirection>,
}

impl Options {
    pub(crate) fn defaults() -> Self {
        Self {
            image_size: Size {
                width: 300,
                height: 220,
            },
            image_alpha: 1.0,
            range_graph_size: RangeVal { min: 60, max: 70 },
            range_graph_angle_pos: vec![RangeVal { min: 0, max: 0 }],
            gen_graph_number: 1,
            enable_graph_vertical_random: false,
            range_dead_zone_directions: vec![
                DeadZoneDirection::Left,
                DeadZoneDirection::Right,
                DeadZoneDirection::Bottom,
                DeadZoneDirection::Top,
            ],
        }
    }
}

/// Option setter applied by [`crate::Builder::set_options`].
pub struct OptionFn {
    inner: Box<dyn FnOnce(&mut Options) + Send>,
}

impl OptionFn {
    fn new(f: impl FnOnce(&mut Options) + Send + 'static) -> Self {
        Self { inner: Box::new(f) }
    }

    pub(crate) fn apply(self, opts: &mut Options) {
        (self.inner)(opts);
    }
}

impl IntoIterator for OptionFn {
    type Item = OptionFn;
    type IntoIter = Once<OptionFn>;

    fn into_iter(self) -> Self::IntoIter {
        std::iter::once(self)
    }
}

/// Sets the master image size. Default: 300×220.
#[must_use]
pub fn with_image_size(size: Size) -> OptionFn {
    OptionFn::new(move |opts| {
        opts.image_size = size;
    })
}

/// Sets master/tile image alpha in `0.0..=1.0`. Default: `1.0`.
#[must_use]
pub fn with_image_alpha(alpha: f32) -> OptionFn {
    OptionFn::new(move |opts| {
        opts.image_alpha = alpha.clamp(0.0, 1.0);
    })
}

/// Sets the inclusive puzzle-piece size range. Default: 60..=70.
#[must_use]
pub fn with_range_graph_size(val: RangeVal) -> OptionFn {
    OptionFn::new(move |opts| {
        opts.range_graph_size = val;
    })
}

/// Sets candidate angle ranges. Default: `[0..=0]` (no rotation).
#[must_use]
pub fn with_range_graph_angle_pos(vals: Vec<RangeVal>) -> OptionFn {
    OptionFn::new(move |opts| {
        opts.range_graph_angle_pos = vals;
    })
}

/// Sets how many hole shadows to draw. Values below 1 are clamped to 1.
#[must_use]
pub fn with_gen_graph_number(val: u32) -> OptionFn {
    OptionFn::new(move |opts| {
        opts.gen_graph_number = val.max(1);
    })
}

/// Enables a new random Y for each extra graph block.
#[must_use]
pub fn with_enable_graph_vertical_random(val: bool) -> OptionFn {
    OptionFn::new(move |opts| {
        opts.enable_graph_vertical_random = val;
    })
}

/// Sets dead-zone directions used when placing the hole and tile.
#[must_use]
pub fn with_range_dead_zone_directions(val: Vec<DeadZoneDirection>) -> OptionFn {
    OptionFn::new(move |opts| {
        opts.range_dead_zone_directions = val;
    })
}
