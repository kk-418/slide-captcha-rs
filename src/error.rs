/// Errors produced by CAPTCHA generation and image encoding.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// No background image was provided.
    #[error("no background image")]
    EmptyBackground,
    /// A custom graph image is missing overlay, shadow, or mask data.
    #[error("graph image is invalid")]
    InvalidGraphImage,
    /// Block placement or image composition failed.
    #[error("data generation failed")]
    GenerateFailed,
    /// JPEG or PNG encoding failed.
    #[error("image encode failed: {0}")]
    EncodeFailed(String),
    /// Filesystem I/O failed.
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}
