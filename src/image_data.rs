use std::path::Path;

use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use image::codecs::jpeg::JpegEncoder;
use image::codecs::png::PngEncoder;
use image::{DynamicImage, ExtendedColorType, ImageEncoder};

use crate::{Error, Result};

/// Default JPEG quality (image crate `JpegEncoder`).
const JPEG_QUALITY: u8 = 80;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum EncodedFormat {
    Jpeg,
    Png,
}

/// Encoded master (JPEG) or tile (PNG) image.
#[derive(Clone, Debug)]
pub struct EncodedImage {
    image: DynamicImage,
    format: EncodedFormat,
}

impl EncodedImage {
    pub(crate) fn jpeg(image: DynamicImage) -> Self {
        Self {
            image,
            format: EncodedFormat::Jpeg,
        }
    }

    pub(crate) fn png(image: DynamicImage) -> Self {
        Self {
            image,
            format: EncodedFormat::Png,
        }
    }

    /// Underlying decoded image.
    #[must_use]
    pub fn inner(&self) -> &DynamicImage {
        &self.image
    }

    /// Encodes to JPEG (master) or PNG (tile) bytes.
    pub fn to_bytes(&self) -> Result<Vec<u8>> {
        match self.format {
            EncodedFormat::Jpeg => encode_jpeg(&self.image, JPEG_QUALITY),
            EncodedFormat::Png => encode_png(&self.image),
        }
    }

    /// Encodes to a data-URI (`data:image/jpeg;base64,...` or PNG).
    pub fn to_base64(&self) -> Result<String> {
        let data = self.to_base64_data()?;
        let prefix = match self.format {
            EncodedFormat::Jpeg => "data:image/jpeg;base64,",
            EncodedFormat::Png => "data:image/png;base64,",
        };
        Ok(format!("{prefix}{data}"))
    }

    /// Encodes to raw Base64 without a data-URI prefix.
    pub fn to_base64_data(&self) -> Result<String> {
        let bytes = self.to_bytes()?;
        Ok(STANDARD.encode(bytes))
    }

    /// Writes encoded bytes to `path`.
    pub fn save(&self, path: impl AsRef<Path>) -> Result<()> {
        let bytes = self.to_bytes()?;
        std::fs::write(path, bytes)?;
        Ok(())
    }
}

fn encode_jpeg(image: &DynamicImage, quality: u8) -> Result<Vec<u8>> {
    if image.width() == 0 || image.height() == 0 {
        return Err(Error::EncodeFailed("empty image".to_string()));
    }
    let rgb = image.to_rgb8();
    let mut buf = Vec::new();
    {
        let mut encoder = JpegEncoder::new_with_quality(&mut buf, quality);
        encoder
            .encode(
                rgb.as_raw(),
                rgb.width(),
                rgb.height(),
                ExtendedColorType::Rgb8,
            )
            .map_err(|err| Error::EncodeFailed(err.to_string()))?;
    }
    Ok(buf)
}

fn encode_png(image: &DynamicImage) -> Result<Vec<u8>> {
    if image.width() == 0 || image.height() == 0 {
        return Err(Error::EncodeFailed("empty image".to_string()));
    }
    let rgba = image.to_rgba8();
    let mut buf = Vec::new();
    let encoder = PngEncoder::new(&mut buf);
    encoder
        .write_image(
            rgba.as_raw(),
            rgba.width(),
            rgba.height(),
            ExtendedColorType::Rgba8,
        )
        .map_err(|err| Error::EncodeFailed(err.to_string()))?;
    Ok(buf)
}
