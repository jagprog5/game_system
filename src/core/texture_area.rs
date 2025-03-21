use std::num::NonZeroU32;

use typed_floats::{NonNaNFinite, StrictlyPositiveFinite};

use super::color::Color;

#[derive(Debug, Copy, Clone)]
pub struct TextureArea {
    pub x: i32,
    pub y: i32,
    pub w: NonZeroU32,
    pub h: NonZeroU32,
}

impl TextureArea {
    pub fn contains_point<P>(&self, point: P) -> bool
    where
        P: Into<(i32, i32)>,
    {
        let (x, y) = point.into();
        let inside_x = x >= self.x && x < self.x + self.w.get() as i32;
        inside_x && (y >= self.y && y < self.y + self.h.get() as i32)
    }

    pub fn intersection(&self, other: Self) -> Option<TextureArea> {
        let x1 = self.x.max(other.x);
        let y1 = self.y.max(other.y);
        let x2 = (self.x + self.w.get() as i32).min(other.x + other.w.get() as i32);
        let y2 = (self.y + self.h.get() as i32).min(other.y + other.h.get() as i32);
        if x1 < x2 && y1 < y2 {
            Some(TextureArea {
                x: x1,
                y: y1,
                w: NonZeroU32::new((x2 - x1) as u32)?,
                h: NonZeroU32::new((y2 - y1) as u32)?,
            })
        } else {
            None
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct TextureAreaF {
    pub x: NonNaNFinite<f32>,
    pub y: NonNaNFinite<f32>,
    pub w: StrictlyPositiveFinite<f32>,
    pub h: StrictlyPositiveFinite<f32>,
}

#[derive(Debug, Copy, Clone)]
pub struct TextureRotation {
    /// degrees clockwise
    pub angle: NonNaNFinite<f32>,
    /// point in destination (x, y) in which the rotation is about. center if
    /// not stated
    pub point: Option<(i32, i32)>,
    pub flip_horizontal: bool,
    pub flip_vertical: bool,
}

#[derive(Debug, Copy, Clone)]
pub struct TextureRotationF {
    /// degrees clockwise
    pub angle: NonNaNFinite<f32>,
    /// point in destination (x, y) in which the rotation is about. center if
    /// not stated
    pub point: Option<(NonNaNFinite<f32>, NonNaNFinite<f32>)>,
    pub flip_horizontal: bool,
    pub flip_vertical: bool,
}

/// arg for render copy operations. where and how should the texture be placed
pub enum TextureDestination {
    Int(TextureArea, Option<TextureRotation>, Color),
    Float(TextureAreaF, Option<TextureRotationF>, Color),
}

impl From<TextureArea> for TextureDestination {
    fn from(area: TextureArea) -> Self {
        TextureDestination::Int(
            area,
            None,
            Color {
                r: 0xFF,
                g: 0xFF,
                b: 0xFF,
                a: 0xFF,
            },
        )
    }
}

impl From<TextureAreaF> for TextureDestination {
    fn from(area: TextureAreaF) -> Self {
        TextureDestination::Float(
            area,
            None,
            Color {
                r: 0xFF,
                g: 0xFF,
                b: 0xFF,
                a: 0xFF,
            },
        )
    }
}
