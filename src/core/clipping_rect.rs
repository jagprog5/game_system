use std::num::NonZeroU32;

use super::texture_area::TextureArea;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ClippingArea {
    /// a non-zero area clipping rect
    Some(TextureArea),
    /// a clipping rect with zero area
    Zero,
    /// the absence of a clipping rect
    None,
}

impl ClippingArea {
    pub fn intersect_area(&self, position: Option<TextureArea>) -> ClippingArea {
        match position {
            Some(position) => {
                match self {
                    ClippingArea::Some(rect) => match rect.intersection(position) {
                        Some(v) => ClippingArea::Some(v),
                        None => ClippingArea::Zero,
                    },
                    ClippingArea::Zero => ClippingArea::Zero,
                    ClippingArea::None => {
                        // clipping rect has infinite area, so it's just whatever position is
                        ClippingArea::Some(position)
                    }
                }
            }
            None => {
                // position is zero area so intersection result is zero
                ClippingArea::Zero
            }
        }
    }

    pub fn intersection(&self, other: ClippingArea) -> ClippingArea {
        match self {
            ClippingArea::Zero => ClippingArea::Zero,
            ClippingArea::None => other,
            ClippingArea::Some(self_rect) => match other {
                ClippingArea::Zero => ClippingArea::Zero,
                ClippingArea::None => *self,
                ClippingArea::Some(rect) => {
                    let x1 = self_rect.x.max(rect.x);
                    let y1 = self_rect.y.max(rect.y);
                    let x2 =
                        (self_rect.x + self_rect.w.get() as i32).min(rect.x + rect.w.get() as i32);
                    let y2 =
                        (self_rect.y + self_rect.h.get() as i32).min(rect.y + rect.h.get() as i32);
                    if x2 > x1 && y2 > y1 {
                        ClippingArea::Some(TextureArea {
                            x: x1,
                            y: y1,
                            w: NonZeroU32::new((x2 - x1) as u32).unwrap(),
                            h: NonZeroU32::new((y2 - y1) as u32).unwrap(),
                        })
                    } else {
                        ClippingArea::Zero
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
            ClippingArea::Some(texture_area) => texture_area.contains_point(point),
            ClippingArea::Zero => false,
            ClippingArea::None => true,
        }
    }
}