use std::num::NonZeroU32;

pub use typed_floats::{NonNaNFinite, StrictlyPositiveFinite};

use crate::ui::util::rect::FRect;

use super::color::Color;

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

pub struct TextureDestination(pub TextureRect, pub Option<TextureRotation>, pub Color);

pub struct TextureDestinationF(pub TextureRectF, pub Option<TextureRotationF>, pub Color);

/// ergonomic cast - sets default fields
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

/// ergonomic cast - sets default fields
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

// =============================================================================

/// how should an image's aspect ratio be treated if the available space does
/// not have the same ratio
pub enum AspectRatioFailPolicy {
    /// simply stretch the image to fit the available space, ignoring the aspect
    /// ratio
    Stretch,

    /// zoom out, adding blank space.
    ///
    /// contains two floats from 0-1 (inclusive), where 0 aligns the image in
    /// the negative direction (x, y respectively), and 1 aligns the image in
    /// the positive direction.
    ///
    /// a sane default is (0.5, 0.5)
    ZoomOut((f32, f32)),

    /// zoom in, cutting off excess length
    ///
    /// contains two floats from 0-1 (inclusive) where 0 aligns the image in the
    /// negative direction (x, y respectively), and 1 aligns the image in the
    /// positive direction.
    ///
    /// a sane default is (0.5, 0.5)
    ZoomIn((f32, f32)),
}

impl Default for AspectRatioFailPolicy {
    fn default() -> Self {
        AspectRatioFailPolicy::ZoomOut((0.5, 0.5))
    }
}

impl AspectRatioFailPolicy {
    /// return the src and dst to use, respectively
    pub fn get(
        &self,
        src: TextureRectF,
        dst: crate::ui::util::rect::FRect,
    ) -> Option<(TextureRectF, TextureRectF)> {
        match self {
            AspectRatioFailPolicy::Stretch => {
                let dst: Option<TextureRectF> = dst.into();
                match dst {
                    Some(dst) => Some((src.into(), dst)),
                    None => None,
                }
            }
            AspectRatioFailPolicy::ZoomOut((zoom_x, zoom_y)) => {
                let src_aspect_ratio = src.w.get() / src.h.get();
                if dst.h == 0. {
                    return None; // guard div + can't drawn zero area texture
                }
                let dst_aspect_ratio = dst.w / dst.h;

                let maybe_dst: Option<TextureRectF> = if src_aspect_ratio > dst_aspect_ratio {
                    // padding at the top and bottom; scale down the size of the
                    // src so the width matches the destination
                    let scale_down = dst.w / src.w.get();
                    let dst_width = src.w.get() * scale_down;
                    let dst_height = src.h.get() * scale_down;
                    let dst_y_offset = (dst.h - dst_height) * zoom_y;
                    let maybe_dst = crate::ui::util::rect::FRect {
                        x: dst.x,
                        y: dst.y + dst_y_offset,
                        w: dst_width,
                        h: dst_height,
                    };

                    maybe_dst.into()
                } else {
                    // padding at the left and right; scale down the size of the
                    // src so the height matches the destination
                    let scale_down = dst.h / src.h.get();
                    let dst_width = src.w.get() * scale_down;
                    let dst_height = src.h.get() * scale_down;
                    let dst_x_offset = (dst.w - dst_width) * zoom_x;

                    let maybe_dst = crate::ui::util::rect::FRect {
                        x: dst.x + dst_x_offset,
                        y: dst.y,
                        w: dst_width,
                        h: dst_height,
                    };

                    maybe_dst.into()
                };
                match maybe_dst {
                    Some(dst) => Some((src.into(), dst)),
                    None => None,
                }
            }
            AspectRatioFailPolicy::ZoomIn((zoom_x, zoom_y)) => {
                let src_aspect_ratio = src.w.get() / src.h.get();
                if dst.h == 0. || dst.w == 0. {
                    return None; // guard div + can't drawn zero area texture
                }
                let dst_aspect_ratio = dst.w / dst.h;

                let maybe_src: Option<TextureRectF> = if src_aspect_ratio > dst_aspect_ratio {
                    let width = dst_aspect_ratio * src.h.get();
                    let x = (src.w.get() - width) * zoom_x;
                    FRect {
                        x: src.x.get() + x,
                        y: src.y.get(),
                        w: width,
                        h: src.h.get(),
                    }
                    .into()
                } else {
                    let height = (src.w.get() / dst.w) * dst.h;
                    let y = (src.h.get() - height) * zoom_y;
                    FRect {
                        x: src.x.get(),
                        y: src.y.get() + y,
                        w: src.w.get(),
                        h: height,
                    }
                    .into()
                };

                let maybe_dst: Option<TextureRectF> = dst.into();
                match maybe_dst {
                    Some(dst) => match maybe_src {
                        Some(src) => Some((src, dst)),
                        None => None,
                    },
                    None => None,
                }
            }
        }
    }
}
