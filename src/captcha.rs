use image::DynamicImage;
use rand::Rng;

use crate::draw::{gen_master_image, gen_tile_image};
use crate::jigsaw;
use crate::options::{OptionFn, Options};
use crate::resource::{ResourceFn, Resources};
use crate::types::{Block, DeadZoneDirection, Mode, Point};
use crate::{Error, Result};

pub use crate::image_data::EncodedImage;

/// Builder for [`Captcha`].
///
/// Collects options and resources, then produces a basic or drag-drop CAPTCHA.
pub struct Builder {
    opts: Vec<OptionFn>,
    resources: Vec<ResourceFn>,
}

impl Default for Builder {
    fn default() -> Self {
        Self::new()
    }
}

impl Builder {
    /// Creates an empty builder. Defaults are applied in [`Self::make`].
    #[must_use]
    pub fn new() -> Self {
        Self {
            opts: Vec::new(),
            resources: Vec::new(),
        }
    }

    /// Appends option setters.
    ///
    /// Accepts a single [`OptionFn`] or any iterator of them.
    #[must_use]
    pub fn set_options(mut self, opts: impl IntoIterator<Item = OptionFn>) -> Self {
        self.opts.extend(opts);
        self
    }

    /// Appends resource setters.
    ///
    /// Accepts a single [`ResourceFn`] or any iterator of them.
    #[must_use]
    pub fn set_resources(mut self, resources: impl IntoIterator<Item = ResourceFn>) -> Self {
        self.resources.extend(resources);
        self
    }

    /// Clears collected options and resources.
    #[must_use]
    pub fn clear(mut self) -> Self {
        self.opts.clear();
        self.resources.clear();
        self
    }

    /// Builds a basic slider CAPTCHA.
    #[must_use]
    pub fn make(self) -> Captcha {
        self.build(Mode::Basic)
    }

    /// Builds a drag-drop CAPTCHA.
    #[must_use]
    pub fn make_drag_drop(self) -> Captcha {
        self.build(Mode::DragDrop)
    }

    fn build(self, mode: Mode) -> Captcha {
        let mut opts = Options::defaults();
        if mode == Mode::Basic {
            opts.range_dead_zone_directions = vec![DeadZoneDirection::Left];
            opts.enable_graph_vertical_random = false;
        }
        for opt in self.opts {
            opt.apply(&mut opts);
        }

        let mut resources = Resources::default();
        for resource in self.resources {
            resource.apply(&mut resources);
        }

        Captcha {
            opts,
            resources,
            mode,
        }
    }
}

/// Configured slide CAPTCHA generator.
pub struct Captcha {
    opts: Options,
    resources: Resources,
    mode: Mode,
}

impl Captcha {
    /// Generates one CAPTCHA: master JPEG, tile PNG, and block metadata.
    pub fn generate(&self) -> Result<CaptchaData> {
        self.check()?;

        let mut rng = rand::rng();
        let (blocks, tile_point) = self.gen_graph_blocks(&mut rng)?;
        if blocks.is_empty() {
            return Err(Error::GenerateFailed);
        }

        let block_index = if blocks.len() == 1 {
            0
        } else {
            rng.random_range(0..blocks.len())
        };
        let mut block = blocks[block_index];

        let (overlay, shadow, mask) = self.gen_graph(block.width, &mut rng)?;
        let background = choose(&self.resources.backgrounds, &mut rng)
            .cloned()
            .ok_or(Error::EmptyBackground)?;

        let (master, clean) = gen_master_image(
            &mut rng,
            &background,
            &shadow,
            &blocks,
            self.opts.image_size,
            self.opts.image_alpha,
        )?;

        let tile = gen_tile_image(&mask, &clean, &overlay, &block, self.opts.image_alpha)?;

        if self.mode == Mode::Basic {
            block.dy = block.y;
        } else {
            block.dy = tile_point.y;
        }
        block.dx = tile_point.x;

        Ok(CaptchaData {
            block,
            master_image: EncodedImage::jpeg(master),
            tile_image: EncodedImage::png(tile),
        })
    }

    fn check(&self) -> Result<()> {
        if self.resources.backgrounds.is_empty() {
            return Err(Error::EmptyBackground);
        }
        for graph in &self.resources.graphs {
            if !graph.is_valid() {
                return Err(Error::InvalidGraphImage);
            }
        }
        if self.opts.image_size.width == 0 || self.opts.image_size.height == 0 {
            return Err(Error::GenerateFailed);
        }
        Ok(())
    }

    fn gen_graph<R: Rng>(
        &self,
        size: u32,
        rng: &mut R,
    ) -> Result<(DynamicImage, DynamicImage, DynamicImage)> {
        if self.resources.graphs.is_empty() {
            let jigsaw = jigsaw::generate(size, size);
            return Ok((
                DynamicImage::ImageRgba8(jigsaw.overlay),
                DynamicImage::ImageRgba8(jigsaw.shadow),
                DynamicImage::ImageRgba8(jigsaw.mask),
            ));
        }
        let graph = choose(&self.resources.graphs, rng).ok_or(Error::InvalidGraphImage)?;
        Ok((
            graph.overlay.clone(),
            graph.shadow.clone(),
            graph.mask.clone(),
        ))
    }

    fn gen_graph_blocks<R: Rng>(&self, rng: &mut R) -> Result<(Vec<Block>, Point)> {
        let image_size = self.opts.image_size;
        let size = self.opts.range_graph_size;
        let length = self.opts.gen_graph_number.max(1);

        let width = i32::try_from(image_size.width).map_err(|_| Error::GenerateFailed)?;
        let height = i32::try_from(image_size.height).map_err(|_| Error::GenerateFailed)?;

        let rand_angle = self.rand_graph_angle(rng);
        let rand_size = rand_inclusive(rng, size.min, size.max);
        if rand_size <= 0 {
            return Err(Error::GenerateFailed);
        }
        let c_height = rand_size;
        let c_width = rand_size;
        if c_width >= width || c_height >= height {
            return Err(Error::GenerateFailed);
        }

        let dzd_type = self.rand_dead_zone_direction(rng);
        let dp = c_width / 2;
        let usable = width - c_width - 20;
        if usable <= 0 {
            return Err(Error::GenerateFailed);
        }
        let length_i = i32::try_from(length).map_err(|_| Error::GenerateFailed)?;
        let block_width = usable / length_i;

        let mut y = calc_y_with_dead_zone(rng, 5, height - c_height - 5, c_height, dzd_type);
        let mut blocks = Vec::with_capacity(length as usize);

        for i in 0..length_i {
            let (mut start, end) = calc_x_with_dead_zone(
                (i * block_width) + dp + 5,
                ((i + 1) * block_width) - dp,
                c_width,
                dzd_type,
            );
            start = start.max(dp + 5);
            let x = rand_inclusive(rng, start + 20, end + 20) - dp;

            if self.opts.enable_graph_vertical_random {
                y = calc_y_with_dead_zone(rng, 5, height - c_height - 5, c_height, dzd_type);
            }

            let bw = u32::try_from(c_width).map_err(|_| Error::GenerateFailed)?;
            let bh = u32::try_from(c_height).map_err(|_| Error::GenerateFailed)?;
            blocks.push(Block {
                x,
                y,
                width: bw,
                height: bh,
                angle: rand_angle,
                dx: 0,
                dy: 0,
            });
        }

        let point = if self.mode == Mode::Basic {
            Point {
                x: rand_inclusive(rng, 5, dp),
                y,
            }
        } else {
            drag_tile_point(rng, dzd_type, width, height, c_width, c_height)
        };
        Ok((blocks, point))
    }

    fn rand_dead_zone_direction<R: Rng>(&self, rng: &mut R) -> DeadZoneDirection {
        let dirs = &self.opts.range_dead_zone_directions;
        if dirs.is_empty() {
            return DeadZoneDirection::Left;
        }
        dirs[rng.random_range(0..dirs.len())]
    }

    fn rand_graph_angle<R: Rng>(&self, rng: &mut R) -> i32 {
        let angles = &self.opts.range_graph_angle_pos;
        if angles.is_empty() {
            return 0;
        }
        let angle = &angles[rng.random_range(0..angles.len())];
        rand_inclusive(rng, angle.min, angle.max)
    }
}

/// Generated CAPTCHA payload: block coordinates plus encoded images.
#[derive(Clone, Debug)]
pub struct CaptchaData {
    block: Block,
    master_image: EncodedImage,
    tile_image: EncodedImage,
}

impl CaptchaData {
    /// Puzzle block (hole position, size, and tile display offset).
    #[must_use]
    pub fn block(&self) -> &Block {
        &self.block
    }

    /// Master image (JPEG).
    #[must_use]
    pub fn master_image(&self) -> &EncodedImage {
        &self.master_image
    }

    /// Tile image (PNG with alpha).
    #[must_use]
    pub fn tile_image(&self) -> &EncodedImage {
        &self.tile_image
    }
}

fn drag_tile_point<R: Rng>(
    rng: &mut R,
    dzd_type: DeadZoneDirection,
    width: i32,
    height: i32,
    c_width: i32,
    c_height: i32,
) -> Point {
    match dzd_type {
        DeadZoneDirection::Top => Point {
            x: rand_inclusive(rng, 5, width - c_width - 5),
            y: 5,
        },
        DeadZoneDirection::Bottom => Point {
            x: rand_inclusive(rng, 5, width - c_width - 5),
            y: height - c_height - 5,
        },
        DeadZoneDirection::Left => Point {
            x: 5,
            y: rand_inclusive(rng, 5, height - c_height - 5),
        },
        DeadZoneDirection::Right => Point {
            x: width - c_width - 5,
            y: rand_inclusive(rng, 5, height - c_height - 5),
        },
    }
}

fn calc_x_with_dead_zone(
    mut start: i32,
    mut end: i32,
    value: i32,
    dzd_type: DeadZoneDirection,
) -> (i32, i32) {
    if dzd_type == DeadZoneDirection::Left {
        start += value;
        end += value;
    }
    (start, end)
}

fn calc_y_with_dead_zone<R: Rng>(
    rng: &mut R,
    mut start: i32,
    mut end: i32,
    value: i32,
    dzd_type: DeadZoneDirection,
) -> i32 {
    match dzd_type {
        DeadZoneDirection::Top => start += value,
        DeadZoneDirection::Bottom => end -= value,
        DeadZoneDirection::Left | DeadZoneDirection::Right => {}
    }
    rand_inclusive(rng, start, end)
}

fn rand_inclusive<R: Rng>(rng: &mut R, a: i32, b: i32) -> i32 {
    let (min, max) = if a <= b { (a, b) } else { (b, a) };
    rng.random_range(min..=max)
}

fn choose<'a, T, R: Rng>(items: &'a [T], rng: &mut R) -> Option<&'a T> {
    if items.is_empty() {
        return None;
    }
    items.get(rng.random_range(0..items.len()))
}
