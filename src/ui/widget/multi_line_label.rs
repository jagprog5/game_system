use std::num::NonZeroU16;

use crate::{
    core::{
        color::Color,
        texture_area::{TextureRect, TextureSource},
        NonEmptyStr, TextureHandle,
    },
    ui::util::{
        length::{MaxLenFailPolicy, MinLenFailPolicy, PreferredPortion},
        rect::rect_len_round,
        rust::CellRefOrCell,
    },
};

use super::{Widget, WidgetUpdateEvent};

/// a multiline label's sizing is flexible - it can be any size. if the
/// width is too small, then it will wrap text. however, if the height is
/// too large, what should happen?
pub enum MultiLineMinHeightFailPolicy {
    /// cut off the text, to ensure it does not expand over the parent. contains
    /// a value from 0 to 1 inclusively, indicating if the text should be cut
    /// off from the negative or positive direction, respectively
    CutOff(f32),
    /// allow the text to be drawn past the parent's boundary in a direction.
    /// indicate the direction
    AllowRunOff(MinLenFailPolicy),
    /// do not cut off the height. request an appropriate height, deduced from
    /// the width and text
    None(MinLenFailPolicy, MaxLenFailPolicy),
}

impl Default for MultiLineMinHeightFailPolicy {
    fn default() -> Self {
        MultiLineMinHeightFailPolicy::AllowRunOff(MinLenFailPolicy::POSITIVE)
    }
}

/// a widget that contains multiline text
///
/// this widget defines a height_from_width but not a width_from_height. because
/// of this, the text will be compressed vertically when all of the following
/// conditions are true:
///  - it's in a width from height (e.g. vertical layout) context
///  - MultiLineMinHeightFailPolicy::None
///  - lots of text: the wrapped text has a smaller aspect ratio than the parent
///    is giving it
///
/// there's a few ways of fixing this:
///  - change the context. e.g. wrap the MultiLineLabel in a horizontal layout
///    or something which places it's contained widget (like a scroller with
///    NestedContentSizing::custom)
///  - change the other two conditions
pub struct MultiLineLabel<'state> {
    pub text: CellRefOrCell<'state, String>,
    /// a single line label infers an appropriate point size from the available
    /// height. this doesn't make sense for multiline text, so it's instead
    /// stated literally
    pub point_size: NonZeroU16,
    pub color: Color,

    pub max_h_policy: MaxLenFailPolicy,
    pub min_h_policy: MultiLineMinHeightFailPolicy,

    pub preferred_w: PreferredPortion,
    pub preferred_h: PreferredPortion,

    /// state stored for draw from update
    draw_pos: crate::ui::util::rect::FRect,
}

impl<'state> MultiLineLabel<'state> {
    pub fn new(text: CellRefOrCell<'state, String>, point_size: NonZeroU16, color: Color) -> Self {
        Self {
            text,
            point_size,
            color,
            preferred_w: Default::default(),
            preferred_h: Default::default(),
            min_h_policy: Default::default(),
            max_h_policy: Default::default(),
            draw_pos: Default::default(),
        }
    }
}

impl<'state, 'a, T: crate::core::System<'a>> Widget<'a, T> for MultiLineLabel<'state> {
    fn preferred_portion(&self) -> (PreferredPortion, PreferredPortion) {
        (self.preferred_w, self.preferred_h)
    }

    fn preferred_ratio_exceed_parent(&self) -> bool {
        match self.min_h_policy {
            MultiLineMinHeightFailPolicy::None(_, _) => true,
            _ => false,
        }
    }

    fn min_h_fail_policy(&self) -> MinLenFailPolicy {
        match self.min_h_policy {
            MultiLineMinHeightFailPolicy::None(min_len_fail_policy, _) => min_len_fail_policy,
            _ => Default::default(), // doesn't matter
        }
    }

    fn max_h_fail_policy(&self) -> MaxLenFailPolicy {
        match self.min_h_policy {
            MultiLineMinHeightFailPolicy::None(_, max_len_fail_policy) => max_len_fail_policy,
            _ => Default::default(), // doesn't matter
        }
    }

    fn preferred_height_from_width(
        &self,
        pref_w: f32,
        sys_interface: &mut T,
    ) -> Option<Result<f32, String>> {
        match self.min_h_policy {
            MultiLineMinHeightFailPolicy::None(_, _) => Some((|| {
                let wrap_width = match rect_len_round(pref_w) {
                    Some(v) => v,
                    None => return Ok(0.),
                };

                let text = self.text.scope_take();
                let text: NonEmptyStr = match text.as_str().try_into() {
                    Ok(v) => v,
                    Err(()) => return Ok(0.),
                };

                let texture = sys_interface.text(text, self.point_size, Some(wrap_width))?;
                let size = texture.size()?;
                Ok(size.1.get() as f32)
            })()),
            _ => None,
        }
    }

    fn update(&mut self, event: WidgetUpdateEvent, _sys_interface: &mut T) -> Result<bool, String> {
        self.draw_pos = event.position;
        Ok(false)
    }

    fn draw(&self, sys_interface: &mut T) -> Result<(), String> {
        let position: TextureRect = match self.draw_pos.into() {
            Some(v) => v,
            None => return Ok(()), // no input handling
        };

        let text = self.text.scope_take();
        let text: NonEmptyStr = match text.as_str().try_into() {
            Ok(v) => v,
            Err(()) => return Ok(()),
        };

        let mut texture = sys_interface.text(text, self.point_size, Some(position.w))?;
        let size = texture.size()?;

        if size.1 <= position.h {
            let excess = position.h.get() - size.1.get();
            let excess = excess as f32;
            let excess = excess * self.max_h_policy.0;
            let excess = excess.round() as i32;
            texture.copy(
                TextureSource::WholeTexture,
                TextureRect {
                    x: position.x,
                    y: position.y + excess,
                    w: size.0,
                    h: size.1,
                },
            )?;
        } else {
            let excess = size.1.get() - position.h.get();
            let excess = excess as f32;
            match self.min_h_policy {
                MultiLineMinHeightFailPolicy::CutOff(v) => {
                    let excess = excess * (1. - v);
                    let excess = excess.round() as i32;
                    texture.copy(
                        TextureRect {
                            x: 0,
                            y: excess,
                            w: size.0,
                            h: position.h,
                        },
                        TextureRect {
                            x: position.x,
                            y: position.y,
                            w: size.0,
                            h: position.h,
                        },
                    )?
                }
                MultiLineMinHeightFailPolicy::AllowRunOff(v) => {
                    let excess = excess * (v.0 - 1.);
                    let excess = excess.round() as i32;
                    texture.copy(
                        TextureSource::WholeTexture,
                        TextureRect {
                            x: position.x,
                            y: position.y + excess,
                            w: size.0,
                            h: size.1,
                        },
                    )?;
                }
                MultiLineMinHeightFailPolicy::None(_, _) => {
                    texture.copy(TextureSource::WholeTexture, position)?;
                }
            }
        }
        Ok(())
    }
}
