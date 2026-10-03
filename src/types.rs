/// Pixel size of an image or puzzle block.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Size {
    /// Width in pixels.
    pub width: u32,
    /// Height in pixels.
    pub height: u32,
}

/// Inclusive integer range.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct RangeVal {
    /// Lower bound (inclusive).
    pub min: i32,
    /// Upper bound (inclusive).
    pub max: i32,
}

/// Integer point.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Point {
    /// X coordinate.
    pub x: i32,
    /// Y coordinate.
    pub y: i32,
}

/// CAPTCHA interaction mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    /// Horizontal slider. Tile Y matches the hole Y.
    Basic,
    /// Tile starts on a dead-zone edge and can be dragged freely.
    DragDrop,
}

/// Edge used as a dead zone when placing the puzzle hole and tile.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeadZoneDirection {
    /// Left edge.
    Left,
    /// Right edge.
    Right,
    /// Top edge.
    Top,
    /// Bottom edge.
    Bottom,
}

/// Puzzle block metadata returned with a generated CAPTCHA.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Block {
    /// Hole X on the master image.
    pub x: i32,
    /// Hole Y on the master image.
    pub y: i32,
    /// Block width in pixels.
    pub width: u32,
    /// Block height in pixels.
    pub height: u32,
    /// Rotation angle in degrees (0 in v0.1).
    pub angle: i32,
    /// Tile display X.
    pub dx: i32,
    /// Tile display Y.
    pub dy: i32,
}

impl Size {
    /// Creates a new size.
    #[must_use]
    pub const fn new(width: u32, height: u32) -> Self {
        Self { width, height }
    }
}

impl RangeVal {
    /// Creates a new inclusive range.
    #[must_use]
    pub const fn new(min: i32, max: i32) -> Self {
        Self { min, max }
    }
}
