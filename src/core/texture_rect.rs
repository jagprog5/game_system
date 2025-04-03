use std::num::NonZeroU32;

use typed_floats::{NonNaNFinite, StrictlyPositiveFinite};

use super::color::Color;

/// has a positive area
///
/// in cases where there might be zero area, a Option<TextureRect> is used.
/// inspired from rust-sdl2
///
/// the thinking is as follows:
///  - better for calculating aspect ratios. can't div by 0!
///  - better backend support. some backends just don't handle 0 correctly, so
///    it's best to just pass values that work
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct TextureRect {
    pub x: i32,
    pub y: i32,
    pub w: NonZeroU32,
    pub h: NonZeroU32,
}

impl TextureRect {
    pub fn contains_point<P>(&self, point: P) -> bool
    where
        P: Into<(i32, i32)>,
    {
        let (x, y) = point.into();
        let inside_x = x >= self.x && x < self.x + self.w.get() as i32;
        inside_x && (y >= self.y && y < self.y + self.h.get() as i32)
    }

    pub fn intersection(&self, other: Self) -> Option<TextureRect> {
        let x1 = self.x.max(other.x);
        let y1 = self.y.max(other.y);
        let x2 = (self.x + self.w.get() as i32).min(other.x + other.w.get() as i32);
        let y2 = (self.y + self.h.get() as i32).min(other.y + other.h.get() as i32);
        if x1 < x2 && y1 < y2 {
            Some(TextureRect {
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
pub struct TextureRectF {
    pub x: NonNaNFinite<f32>,
    pub y: NonNaNFinite<f32>,
    pub w: StrictlyPositiveFinite<f32>,
    pub h: StrictlyPositiveFinite<f32>,
}

impl From<TextureRect> for TextureRectF {
    fn from(value: TextureRect) -> Self {
        unsafe {
            TextureRectF {
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
    Area(TextureRect),
}

impl From<TextureRect> for TextureSource {
    fn from(value: TextureRect) -> Self {
        Self::Area(value)
    }
}

#[derive(Default, Clone, Copy)]
pub enum TextureSourceF {
    #[default]
    WholeTexture,
    Area(TextureRectF),
}

impl From<TextureRectF> for TextureSourceF {
    fn from(value: TextureRectF) -> Self {
        Self::Area(value)
    }
}

pub struct TextureDestination(pub TextureRect, pub Option<TextureRotation>, pub Color);

pub struct TextureDestinationF(pub TextureRectF, pub Option<TextureRotationF>, pub Color);

impl From<TextureRect> for TextureDestination {
    fn from(area: TextureRect) -> Self {
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

impl From<TextureRectF> for TextureDestinationF {
    fn from(area: TextureRectF) -> Self {
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
