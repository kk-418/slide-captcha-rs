use std::iter::Once;

use image::DynamicImage;

/// Custom puzzle-piece images: overlay (rim/texture), shadow (hole), and mask.
#[derive(Clone)]
pub struct GraphImage {
    /// Piece overlay drawn on the tile (border / texture).
    pub overlay: DynamicImage,
    /// Shadow drawn onto the master image as the hole.
    pub shadow: DynamicImage,
    /// Alpha mask used to cut the tile from the background.
    pub mask: DynamicImage,
}

impl GraphImage {
    /// Creates a graph image from overlay, shadow, and mask.
    #[must_use]
    pub fn new(overlay: DynamicImage, shadow: DynamicImage, mask: DynamicImage) -> Self {
        Self {
            overlay,
            shadow,
            mask,
        }
    }

    pub(crate) fn is_valid(&self) -> bool {
        self.overlay.width() > 0
            && self.overlay.height() > 0
            && self.shadow.width() > 0
            && self.shadow.height() > 0
            && self.mask.width() > 0
            && self.mask.height() > 0
    }
}

#[derive(Clone, Default)]
pub(crate) struct Resources {
    pub backgrounds: Vec<DynamicImage>,
    pub graphs: Vec<GraphImage>,
}

/// Resource setter applied by [`crate::Builder::set_resources`].
pub struct ResourceFn {
    inner: Box<dyn FnOnce(&mut Resources) + Send>,
}

impl ResourceFn {
    fn new(f: impl FnOnce(&mut Resources) + Send + 'static) -> Self {
        Self { inner: Box::new(f) }
    }

    pub(crate) fn apply(self, resources: &mut Resources) {
        (self.inner)(resources);
    }
}

impl IntoIterator for ResourceFn {
    type Item = ResourceFn;
    type IntoIter = Once<ResourceFn>;

    fn into_iter(self) -> Self::IntoIter {
        std::iter::once(self)
    }
}

/// Sets background images. At least one is required.
#[must_use]
pub fn with_backgrounds(images: Vec<DynamicImage>) -> ResourceFn {
    ResourceFn::new(move |resources| {
        resources.backgrounds = images;
    })
}

/// Sets custom graph images. When empty, a procedural jigsaw is generated.
#[must_use]
pub fn with_graph_images(images: Vec<GraphImage>) -> ResourceFn {
    ResourceFn::new(move |resources| {
        resources.graphs = images;
    })
}
