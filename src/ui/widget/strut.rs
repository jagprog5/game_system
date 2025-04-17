use crate::ui::util::length::{MaxLen, MinLen, PreferredPortion};

use super::Widget;

/// create horizontal or vertical spaces
#[derive(Debug, Clone)]
pub struct Strut {
    pub min_w: MinLen,
    pub min_h: MinLen,
    pub max_w: MaxLen,
    pub max_h: MaxLen,
    pub preferred_w: PreferredPortion,
    pub preferred_h: PreferredPortion,
}

impl Strut {
    pub fn fixed(w: f32, h: f32) -> Self {
        Strut {
            min_w: MinLen(w),
            min_h: MinLen(h),
            max_w: MaxLen(w),
            max_h: MaxLen(h),
            preferred_w: PreferredPortion(0.),
            preferred_h: PreferredPortion(0.),
        }
    }

    // prefers to be at its largest, but will shrink as needed
    pub fn new(min: (MinLen, MinLen), max: (MaxLen, MaxLen)) -> Self {
        Strut {
            min_w: min.0,
            min_h: min.1,
            max_w: max.0,
            max_h: max.1,
            preferred_w: PreferredPortion::FULL,
            preferred_h: PreferredPortion::FULL,
        }
    }
}

impl<'a, T: crate::core::System<'a>> Widget<'a, T> for Strut {
    fn draw(&self, _sys_interface: &mut T) -> Result<(), String> {
        Ok(())
    }

    fn max(&self, _sys_interface: &mut T) -> Result<(MaxLen, MaxLen), String> {
        Ok((self.max_w, self.max_h))
    }

    fn min(&self, _sys_interface: &mut T) -> Result<(MinLen, MinLen), String> {
        Ok((self.min_w, self.min_h))
    }

    fn preferred_portion(&self) -> (PreferredPortion, PreferredPortion) {
        (self.preferred_w, self.preferred_h)
    }
}
