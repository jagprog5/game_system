use std::num::NonZeroU32;

use typed_floats::{NonNaNFinite, StrictlyPositiveFinite};

use super::color::Color;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
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

    pub fn size(&self) -> (NonZeroU32, NonZeroU32) {
        (self.w, self.h)
    }
}

#[derive(Debug, Copy, Clone)]
pub struct TextureAreaF {
    pub x: NonNaNFinite<f32>,
    pub y: NonNaNFinite<f32>,
    pub w: StrictlyPositiveFinite<f32>,
    pub h: StrictlyPositiveFinite<f32>,
}

impl From<TextureArea> for TextureAreaF {
    fn from(value: TextureArea) -> Self {
        unsafe {
            TextureAreaF {
                x: NonNaNFinite::<f32>::new_unchecked(value.x as f32),
                y: NonNaNFinite::<f32>::new_unchecked(value.y as f32),
                w: StrictlyPositiveFinite::<f32>::new_unchecked(value.w.get() as f32),
                h: StrictlyPositiveFinite::<f32>::new_unchecked(value.h.get() as f32),
            }
        }
    }
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

/// representing this as an explicit enum, as otherwise there's confusion with
/// Option<TextureArea> (may or may not produce a texture area)
#[derive(Default, Clone, Copy)]
pub enum TextureSource {
    #[default]
    WholeTexture,
    Area(TextureArea),
}

impl From<TextureArea> for TextureSource {
    fn from(value: TextureArea) -> Self {
        Self::Area(value)
    }
}

#[derive(Default, Clone, Copy)]
pub enum TextureSourceF {
    #[default]
    WholeTexture,
    Area(TextureAreaF),
}

impl From<TextureAreaF> for TextureSourceF {
    fn from(value: TextureAreaF) -> Self {
        Self::Area(value)
    }
}

pub struct TextureDestination(pub TextureArea, pub Option<TextureRotation>, pub Color);

pub struct TextureDestinationF(pub TextureAreaF, pub Option<TextureRotationF>, pub Color);

impl From<TextureArea> for TextureDestination {
    fn from(area: TextureArea) -> Self {
        TextureDestination(
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

impl From<TextureAreaF> for TextureDestinationF {
    fn from(area: TextureAreaF) -> Self {
        TextureDestinationF(
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
