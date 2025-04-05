use typed_floats::{NonNaNFinite, StrictlyPositiveFinite};

/// a position in the UI
///
/// this type is deliberately flexible and the members can be any value! it's
/// used for layout calculations:
///
/// - otherwise there's a lot of casting to and from integer. best to keep it
///   as floating point until just before use
/// - started running into issues where a one pixel difference leads to a
///   visible jump. specifically, when a label font size changes in
///   horizontal layout (a one pixel in height leading to a larger difference
///   in width due to aspect ratio)
#[derive(Debug, Clone, Copy, Default)]
pub struct FRect {
    /// can be any value
    pub x: f32,
    /// can be any value
    pub y: f32,
    /// can be any value
    pub w: f32,
    /// can be any value
    pub h: f32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_tie_tests() {
        // whole numbers unaffected
        assert_eq!(rect_position_round(1.), 1);
        assert_eq!(rect_position_round(2.), 2);
        assert_eq!(rect_position_round(0.), 0);
        assert_eq!(rect_position_round(-1.), -1);
        assert_eq!(rect_position_round(-2.), -2);

        // typical rounding is fine

        // close to 0
        assert_eq!(rect_position_round(0.00001), 0);
        assert_eq!(rect_position_round(-0.00001), 0);

        // close
        assert_eq!(rect_position_round(1.0001), 1);
        assert_eq!(rect_position_round(0.9999), 1);

        // far
        assert_eq!(rect_position_round(1.4999), 1);
        assert_eq!(rect_position_round(0.5001), 1);

        // close (negative)
        assert_eq!(rect_position_round(-1.0001), -1);
        assert_eq!(rect_position_round(-0.9999), -1);

        // far negative
        assert_eq!(rect_position_round(-1.4999), -1);
        assert_eq!(rect_position_round(-0.5001), -1);

        // rounding away from 0 on positive side unaffected
        assert_eq!(rect_position_round(0.5), 1);
        assert_eq!(rect_position_round(1.5), 2);

        // checks special functionality (rounding up and not away from zero)
        assert_eq!(rect_position_round(-0.5), 0);
        assert_eq!(rect_position_round(-1.5), -1);
        assert_eq!(rect_position_round(-2.5), -2);
    }
}

/// round, but if exactly between numbers (even if negative!), always round up.
/// this is required or else a 1 pixel gap can appear
///
/// this should be used in contexts where it should match the conversion to
/// TextureArea from crate::util::rect::FRect
pub fn rect_position_round(i: f32) -> i32 {
    let i_whole = i.trunc();
    let i_frac = i - i_whole;
    if i_frac != -0.5 {
        i.round() as i32
    } else {
        i_whole as i32
    }
}

/// round, only giving positive output
///
/// this should be used in contexts where it should match the conversion to
/// TextureArea from crate::util::rect::FRect
pub fn rect_len_round(i: f32) -> Option<std::num::NonZeroU32> {
    let i = i.round();
    if i < 1. {
        // must be positive
        None
    } else {
        Some(unsafe { std::num::NonZeroU32::new_unchecked(i as u32) })
    }
}

impl From<crate::core::texture_rect::TextureRectF> for FRect {
    fn from(value: crate::core::texture_rect::TextureRectF) -> Self {
        Self {
            x: value.x.into(),
            y: value.y.into(),
            w: value.w.into(),
            h: value.h.into(),
        }
    }
}

/// convert to texture area for use by system in drawing a pixel at integer
/// coordinates
impl From<FRect> for Option<crate::core::texture_rect::TextureRect> {
    fn from(val: FRect) -> Self {
        let w = match rect_len_round(val.w) {
            Some(v) => v,
            None => return None,
        };
        let h = match rect_len_round(val.h) {
            Some(v) => v,
            None => return None,
        };
        let x = rect_position_round(val.x);
        let y = rect_position_round(val.y);
        Some(crate::core::texture_rect::TextureRect { x, y, w, h })
    }
}

/// convert to floating pt texture area for use by system
impl From<FRect> for Option<crate::core::texture_rect::TextureRectF> {
    fn from(val: FRect) -> Self {
        let x: NonNaNFinite<f32> = match val.x.try_into() {
            Ok(v) => v,
            Err(_) => return None,
        };

        let y: NonNaNFinite<f32> = match val.y.try_into() {
            Ok(v) => v,
            Err(_) => return None,
        };

        let w: StrictlyPositiveFinite<f32> = match val.w.try_into() {
            Ok(v) => v,
            Err(_) => return None,
        };

        let h: StrictlyPositiveFinite<f32> = match val.h.try_into() {
            Ok(v) => v,
            Err(_) => return None,
        };
        Some(crate::core::texture_rect::TextureRectF { x, y, w, h })
    }
}
