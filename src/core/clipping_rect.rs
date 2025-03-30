use std::num::NonZeroU32;

use super::texture_area::TextureRect;

/// I added this in rust-sdl2. it's redefined here to be agnostic to a specific
/// backend
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ClippingRect {
    /// a non-zero area clipping rect
    Some(TextureRect),
    /// a clipping rect with zero area
    Zero,
    /// the absence of a clipping rect
    None,
}

impl ClippingRect {
    pub fn intersect_area(&self, position: Option<TextureRect>) -> ClippingRect {
        match position {
            Some(position) => {
                match self {
                    ClippingRect::Some(rect) => match rect.intersection(position) {
                        Some(v) => ClippingRect::Some(v),
                        None => ClippingRect::Zero,
                    },
                    ClippingRect::Zero => ClippingRect::Zero,
                    ClippingRect::None => {
                        // clipping rect has infinite area, so it's just whatever position is
                        ClippingRect::Some(position)
                    }
                }
            }
            None => {
                // position is zero area so intersection result is zero
                ClippingRect::Zero
            }
        }
    }

    pub fn intersection(&self, other: ClippingRect) -> ClippingRect {
        match self {
            ClippingRect::Zero => ClippingRect::Zero,
            ClippingRect::None => other,
            ClippingRect::Some(self_rect) => match other {
                ClippingRect::Zero => ClippingRect::Zero,
                ClippingRect::None => *self,
                ClippingRect::Some(rect) => {
                    let x1 = self_rect.x.max(rect.x);
                    let y1 = self_rect.y.max(rect.y);
                    let x2 =
                        (self_rect.x + self_rect.w.get() as i32).min(rect.x + rect.w.get() as i32);
                    let y2 =
                        (self_rect.y + self_rect.h.get() as i32).min(rect.y + rect.h.get() as i32);
                    if x2 > x1 && y2 > y1 {
                        ClippingRect::Some(TextureRect {
                            x: x1,
                            y: y1,
                            w: NonZeroU32::new((x2 - x1) as u32).unwrap(),
                            h: NonZeroU32::new((y2 - y1) as u32).unwrap(),
                        })
                    } else {
                        ClippingRect::Zero
                    }
                }
            },
        }
    }

    pub fn contains_point<P>(&self, point: P) -> bool
    where
        P: Into<(i32, i32)>,
    {
        match self {
            ClippingRect::Some(texture_area) => texture_area.contains_point(point),
            ClippingRect::Zero => false,
            ClippingRect::None => true,
        }
    }
}
