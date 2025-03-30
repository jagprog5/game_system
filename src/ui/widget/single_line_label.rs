use std::num::NonZeroU16;

use crate::core::color::Color;
use crate::core::texture_area::{TextureRect, TextureSource};
use crate::core::NonEmptyStr;
use crate::ui::util::length::{
    AspectRatioPreferredDirection, MaxLen, MaxLenFailPolicy, MaxLenPolicy, MinLen,
    MinLenFailPolicy, MinLenPolicy, PreferredPortion,
};
use crate::ui::util::rect::FRect;

use crate::ui::util::rust::CellRefOrCell;

use super::texture::{texture_draw, AspectRatioFailPolicy};
use super::{Widget, WidgetUpdateEvent};

pub(crate) const RATIO_POINT_SIZE: NonZeroU16 = unsafe { NonZeroU16::new_unchecked(16) };

/// a widget that contains a single line of text.
/// the font object and rendered font is cached - rendering only occurs when the
/// text / style or dimensions change
pub struct SingleLineLabel<'state> {
    pub text: CellRefOrCell<'state, String>,
    pub color: Color,

    pub aspect_ratio_fail_policy: AspectRatioFailPolicy,
    pub request_aspect_ratio: bool,

    pub min_w_fail_policy: MinLenFailPolicy,
    pub max_w_fail_policy: MaxLenFailPolicy,
    pub min_h_fail_policy: MinLenFailPolicy,
    pub max_h_fail_policy: MaxLenFailPolicy,

    // a label does it's sizing by receiving a height, and deriving what the
    // corresponding width would be for that height
    pub min_h: MinLen,
    pub max_h: MaxLen,
    pub min_w_policy: MinLenPolicy,
    pub max_w_policy: MaxLenPolicy,
    pub preferred_w: PreferredPortion,
    pub preferred_h: PreferredPortion,

    /// state stored for draw from update
    draw_pos: FRect,
}

impl<'state> SingleLineLabel<'state> {
    pub fn new<'a, T: crate::core::System<'a>>(text: CellRefOrCell<'state, String>) -> Self {
        Self {
            text,
            color: Color {
                r: 0xFF,
                g: 0xFF,
                b: 0xFF,
                a: 0xFF,
            },
            request_aspect_ratio: true,
            aspect_ratio_fail_policy: Default::default(),
            min_w_fail_policy: Default::default(),
            max_w_fail_policy: Default::default(),
            min_h_fail_policy: Default::default(),
            max_h_fail_policy: Default::default(),
            min_w_policy: Default::default(),
            max_w_policy: Default::default(),
            min_h: Default::default(),
            max_h: Default::default(),
            preferred_w: Default::default(),
            preferred_h: Default::default(),
            draw_pos: Default::default(),
        }
    }

    /// 0 on empty text
    fn ratio<'a, T: crate::core::System<'a>>(&self, sys_interface: &mut T) -> Result<f32, String> {
        Ok({
            let text = self.text.scope_take();
            let text: Result<NonEmptyStr, ()> = text.as_str().try_into();
            match text {
                Err(()) => 0.,
                Ok(v) => {
                    let size = crate::core::TextureHandle::size(&sys_interface.text(
                        v,
                        RATIO_POINT_SIZE,
                        None,
                    )?)?;
                    size.0.get() as f32 / size.1.get() as f32
                }
            }
        })
    }
}

impl<'state, 'a, T: crate::core::System<'a>> Widget<'a, T> for SingleLineLabel<'state> {
    fn min(&self, sys_interface: &mut T) -> Result<(MinLen, MinLen), String> {
        let min_w = AspectRatioPreferredDirection::width_from_height(
            self.ratio(sys_interface)?,
            self.min_h.0,
        );
        Ok((MinLen(min_w), self.min_h))
    }

    fn min_w_fail_policy(&self) -> MinLenFailPolicy {
        self.min_w_fail_policy
    }

    fn min_h_fail_policy(&self) -> MinLenFailPolicy {
        self.min_h_fail_policy
    }

    fn max(&self, sys_interface: &mut T) -> Result<(MaxLen, MaxLen), String> {
        let max_w = AspectRatioPreferredDirection::width_from_height(
            self.ratio(sys_interface)?,
            self.max_h.0,
        );
        Ok((MaxLen(max_w), self.max_h))
    }

    fn max_w_fail_policy(&self) -> MaxLenFailPolicy {
        self.max_w_fail_policy
    }

    fn max_h_fail_policy(&self) -> MaxLenFailPolicy {
        self.max_h_fail_policy
    }

    fn preferred_portion(&self) -> (PreferredPortion, PreferredPortion) {
        (self.preferred_w, self.preferred_h)
    }

    fn preferred_width_from_height(
        &self,
        pref_h: f32,
        sys_interface: &mut T,
    ) -> Option<Result<f32, String>> {
        if !self.request_aspect_ratio {
            return None;
        }
        Some((|| {
            Ok(AspectRatioPreferredDirection::width_from_height(
                self.ratio(sys_interface)?,
                pref_h,
            ))
        })())
    }

    fn preferred_height_from_width(
        &self,
        pref_w: f32,
        sys_interface: &mut T,
    ) -> Option<Result<f32, String>> {
        if !self.request_aspect_ratio {
            return None;
        }
        Some((|| {
            Ok(AspectRatioPreferredDirection::height_from_width(
                self.ratio(sys_interface)?,
                pref_w,
            ))
        })())
    }

    fn update(&mut self, event: WidgetUpdateEvent, _sys_interface: &mut T) -> Result<bool, String> {
        self.draw_pos = event.position;
        Ok(false)
    }

    fn draw(&self, sys_interface: &mut T) -> Result<(), String> {
        let position: TextureRect = match self.draw_pos.into() {
            Some(v) => v,
            None => return Ok(()),
        };

        let text = self.text.scope_take();
        let text: Result<NonEmptyStr, ()> = text.as_str().try_into();

        let point_size_to_use =
            unsafe { NonZeroU16::new_unchecked(position.h.get().min(u16::MAX.into()) as u16) };
        let mut texture = match text {
            Err(()) => return Ok(()),
            Ok(v) => sys_interface.text(v, point_size_to_use, None)?,
        };

        texture_draw(
            &mut texture,
            self.color,
            &self.aspect_ratio_fail_policy,
            TextureSource::WholeTexture,
            self.draw_pos,
        )
    }
}
