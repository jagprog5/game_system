use std::num::NonZeroU32;

pub use typed_floats::{NonNaNFinite, StrictlyPositiveFinite};

/// has a positive area
///
/// in cases where there might be zero area, a Option<TextureRect> is used.
/// inspired from rust-sdl2
///
/// the thinking is as follows:
///  - better for calculating aspect ratios. can't div by 0!
///  - better backend support. some backends just don't handle 0 correctly, so
///    it's best to just pass values that work. and drawing a zero area rect is
///    like not calling the backend at all!
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct TextureRect {
    pub x: i32,
    pub y: i32,
    pub w: NonZeroU32,
    pub h: NonZeroU32,
}

impl TextureRect {
    /// checked ctor - w and h non zero
    pub fn new(x: i32, y: i32, w: u32, h: u32) -> Option<Self> {
        let w = match NonZeroU32::new(w) {
            Some(v) => v,
            None => return None,
        };
        let h = match NonZeroU32::new(h) {
            Some(v) => v,
            None => return None,
        };
        Some(Self { x, y, w, h })
    }

    pub unsafe fn new_unchecked(x: i32, y: i32, w: u32, h: u32) -> Self {
        Self {
            x,
            y,
            w: NonZeroU32::new_unchecked(w),
            h: NonZeroU32::new_unchecked(h),
        }
    }

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

impl TextureRectF {
    /// checked ctor
    pub fn new(x: f32, y: f32, w: f32, h: f32) -> Option<Self> {
        let x = match NonNaNFinite::<f32>::new(x) {
            Ok(v) => v,
            Err(_) => return None,
        };

        let y = match NonNaNFinite::<f32>::new(y) {
            Ok(v) => v,
            Err(_) => return None,
        };

        let w = match StrictlyPositiveFinite::<f32>::new(w) {
            Ok(v) => v,
            Err(_) => return None,
        };

        let h = match StrictlyPositiveFinite::<f32>::new(h) {
            Ok(v) => v,
            Err(_) => return None,
        };
        Some(Self { x, y, w, h })
    }

    pub unsafe fn new_unchecked(x: f32, y: f32, w: f32, h: f32) -> Self {
        Self {
            x: NonNaNFinite::<f32>::new_unchecked(x),
            y: NonNaNFinite::<f32>::new_unchecked(y),
            w: StrictlyPositiveFinite::<f32>::new_unchecked(w),
            h: StrictlyPositiveFinite::<f32>::new_unchecked(h),
        }
    }
}

impl From<TextureRect> for TextureRectF {
    fn from(value: TextureRect) -> Self {
        TextureRectF {
            x: value.x.into(),
            y: value.y.into(),
            w: value.w.into(),
            h: value.h.into(),
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

/// ergonomic cast for enum init
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

/// ergonomic cast for enum init
impl From<TextureRectF> for TextureSourceF {
    fn from(value: TextureRectF) -> Self {
        Self::Area(value)
    }
}

pub struct TextureDestination(pub TextureRect, pub Option<TextureRotation>);

pub struct TextureDestinationF(pub TextureRectF, pub Option<TextureRotationF>);

/// ergonomic cast - sets default fields
impl From<TextureRect> for TextureDestination {
    fn from(area: TextureRect) -> Self {
        TextureDestination(area, None)
    }
}

/// ergonomic cast - sets default fields
impl From<TextureRectF> for TextureDestinationF {
    fn from(area: TextureRectF) -> Self {
        TextureDestinationF(area, None)
    }
}
