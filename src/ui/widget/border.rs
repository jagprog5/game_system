use std::{num::NonZeroU32, path::PathBuf};

use crate::{
    core::{
        color::Color,
        texture_rect::{TextureDestination, TextureRect, TextureRotation},
        TextureHandle,
    },
    ui::util::{
        length::{MaxLen, MaxLenFailPolicy, MinLen, MinLenFailPolicy, PreferredPortion},
        rect::FRect,
    },
};

use super::{Widget, WidgetUpdateEvent};

/// contains some inner content inside a border
///
/// a border is drawn from two different image textures:
///
///  - a side length which is repeated until the length is complete
///  - a corner image
pub struct Border<'font_data, 'b, T: crate::core::System<'font_data> + 'b> {
    pub contained: Box<dyn Widget<'font_data, T> + 'b>,

    pub top: bool,
    pub left: bool,
    pub bottom: bool,
    pub right: bool,

    /// if None (default), the contained content will be placed so it does not
    /// overlap with the surrounding border
    ///
    /// if Some, the border is treated as being the specified width, which
    /// allows for overlap. the border is drawn over top of the contained
    ///
    /// if Some is chosen then the minimum width of the contained should be
    /// larger than the effective border width - otherwise borders on opposite
    /// sides can cross if its area is small (consider containing this border in
    /// a clipper)
    pub effective_border_width: Option<u32>,

    // scale not supported. for details, see
    // game_system::ui::widget::tiled_texture::TiledTexture
    //
    // pub scale: NonZeroU32,
    /// path to length texture
    pub texture_path: PathBuf,
    /// it must be oriented so the length extends left
    /// to right, and the top is the innermost part of the border
    pub length_texture_src: TextureRect,
    pub corner_texture_src: TextureRect,

    /// store state for draw from update
    border_draw_pos: FRect,
}

impl<'font_data, 'b, T: crate::core::System<'font_data> + 'b> Border<'font_data, 'b, T> {
    pub fn new(
        contained: Box<dyn Widget<'font_data, T> + 'b>,
        texture_path: PathBuf,
        length_texture_src: TextureRect,
        corner_texture_src: TextureRect,
    ) -> Self {
        Self {
            contained,
            texture_path,
            length_texture_src,
            corner_texture_src,
            border_draw_pos: Default::default(),
            effective_border_width: None,
            top: true,
            left: true,
            bottom: true,
            right: true,
        }
    }

    fn effective_border_width(&self) -> u32 {
        match self.effective_border_width {
            Some(v) => v,
            None => self.length_texture_src.h.get(),
        }
    }

    fn border_width(&self) -> u32 {
        self.length_texture_src.h.get()
    }

    /// used in sizing logic
    fn vertical_border_count(&self) -> u32 {
        let mut ret = 0;
        if self.bottom {
            ret += 1;
        }
        if self.top {
            ret += 1;
        }
        ret
    }

    /// used in sizing logic
    fn horizontal_border_count(&self) -> u32 {
        let mut ret = 0;
        if self.left {
            ret += 1;
        }
        if self.right {
            ret += 1;
        }
        ret
    }
}

impl<'font_data, 'b, T: crate::core::System<'font_data>> Widget<'font_data, T>
    for Border<'font_data, 'b, T>
{
    fn preferred_portion(&self) -> (PreferredPortion, PreferredPortion) {
        self.contained.preferred_portion()
    }

    fn preferred_width_from_height(
        &self,
        pref_h: f32,
        sys_interface: &mut T,
    ) -> Option<Result<f32, String>> {
        let vertical_subtract_amount = self.vertical_border_count() * self.effective_border_width();
        let vertical_subtract_amount = vertical_subtract_amount as f32;
        let horizontal_add_amount = self.horizontal_border_count() * self.effective_border_width();
        let horizontal_add_amount = horizontal_add_amount as f32;

        // subtract border width from the pref input before passing to the
        // contained widget. then, add it back after getting the result
        let pref_h = pref_h - vertical_subtract_amount;
        debug_assert!(pref_h >= 0.); // safe since min() and max() assure this
        self.contained
            .preferred_width_from_height(pref_h, sys_interface)
            .map(|some| some.map(|ok| ok + horizontal_add_amount))
    }

    fn preferred_height_from_width(
        &self,
        pref_w: f32,
        sys_interface: &mut T,
    ) -> Option<Result<f32, String>> {
        let horizontal_subtract_amount =
            self.horizontal_border_count() * self.effective_border_width();
        let horizontal_subtract_amount = horizontal_subtract_amount as f32;
        let vertical_add_amount = self.vertical_border_count() * self.effective_border_width();
        let vertical_add_amount = vertical_add_amount as f32;

        // subtract border width from the pref input before passing to the
        // contained widget. then, add it back after getting the result
        let pref_w = pref_w - horizontal_subtract_amount;
        debug_assert!(pref_w >= 0.);
        self.contained
            .preferred_height_from_width(pref_w, sys_interface)
            .map(|some| some.map(|ok| ok + vertical_add_amount))
    }

    fn preferred_ratio_exceed_parent(&self) -> bool {
        self.contained.preferred_ratio_exceed_parent()
    }

    fn min_w_fail_policy(&self) -> MinLenFailPolicy {
        self.contained.min_w_fail_policy()
    }

    fn min_h_fail_policy(&self) -> MinLenFailPolicy {
        self.contained.min_h_fail_policy()
    }

    fn max_w_fail_policy(&self) -> MaxLenFailPolicy {
        self.contained.max_w_fail_policy()
    }

    fn max_h_fail_policy(&self) -> MaxLenFailPolicy {
        self.contained.max_h_fail_policy()
    }

    fn min(&self, sys_interface: &mut T) -> Result<(MinLen, MinLen), String> {
        let horizontal_min_add = self.horizontal_border_count() * self.effective_border_width();
        let horizontal_min_add = MinLen(horizontal_min_add as f32);
        let vertical_min_add = self.vertical_border_count() * self.effective_border_width();
        let vertical_min_add = MinLen(vertical_min_add as f32);
        let m = self.contained.min(sys_interface)?;
        Ok((
            m.0.combined(horizontal_min_add),
            m.1.combined(vertical_min_add),
        ))
    }

    fn max(&self, sys_interface: &mut T) -> Result<(MaxLen, MaxLen), String> {
        let horizontal_max_add = self.horizontal_border_count() * self.effective_border_width();
        let horizontal_max_add = MaxLen(horizontal_max_add as f32);
        let vertical_max_add = self.vertical_border_count() * self.effective_border_width();
        let vertical_max_add = MaxLen(vertical_max_add as f32);
        let m = self.contained.max(sys_interface)?;
        Ok((
            m.0.combined(horizontal_max_add),
            m.1.combined(vertical_max_add),
        ))
    }

    fn update(
        &mut self,
        mut event: WidgetUpdateEvent,
        sys_interface: &mut T,
    ) -> Result<bool, String> {
        self.border_draw_pos = event.position;
        let style_width = self.effective_border_width() as f32;
        let position_for_child = crate::ui::util::rect::FRect {
            x: event.position.x + if self.left { style_width } else { 0. },
            y: event.position.y + if self.top { style_width } else { 0. },
            w: event.position.w - self.horizontal_border_count() as f32 * style_width,
            h: event.position.h - self.vertical_border_count() as f32 * style_width,
        };
        self.contained
            .update(event.sub_event(position_for_child), sys_interface)
    }

    fn draw(&self, sys_interface: &mut T) -> Result<(), String> {
        self.contained.draw(sys_interface)?;

        let maybe_pos: Option<TextureRect> = self.border_draw_pos.into();

        // draw border if non empty position (and snap to grid)
        if let Some(pos) = maybe_pos {
            let border_width = self.border_width() as i32;

            let l_border_width = border_width as i32 * if self.left { 1 } else { 0 };
            let r_border_width = border_width as i32 * if self.right { 1 } else { 0 };
            let t_border_width = border_width as i32 * if self.top { 1 } else { 0 };
            let b_border_width = border_width as i32 * if self.bottom { 1 } else { 0 };

            let mut txt = sys_interface.texture(&self.texture_path)?;
            if self.top {
                let mut x_offset = pos.x + l_border_width;
                let mut top_amount_left = pos
                    .w
                    .get()
                    .checked_sub(l_border_width as u32 + r_border_width as u32)
                    .unwrap_or(0);
                loop {
                    if top_amount_left > self.length_texture_src.w.get() {
                        txt.copy(
                            self.length_texture_src,
                            TextureRect {
                                x: x_offset,
                                y: pos.y,
                                w: self.length_texture_src.w,
                                h: self.length_texture_src.h,
                            },
                        )?;
                        x_offset += self.length_texture_src.w.get() as i32;
                        top_amount_left -= self.length_texture_src.w.get();
                    } else {
                        match NonZeroU32::new(top_amount_left) {
                            Some(top_amount_left) => {
                                txt.copy(
                                    TextureRect {
                                        x: self.length_texture_src.x,
                                        y: self.length_texture_src.y,
                                        w: top_amount_left,
                                        h: self.length_texture_src.h,
                                    },
                                    TextureRect {
                                        x: x_offset,
                                        y: pos.y,
                                        w: top_amount_left,
                                        h: self.length_texture_src.h,
                                    },
                                )?;
                            }
                            _ => {}
                        };
                        break;
                    }
                }
            }

            if self.bottom {
                let mut x_offset = pos.x + l_border_width;
                let mut bottom_amount_left = pos
                    .w
                    .get()
                    .checked_sub(l_border_width as u32 + r_border_width as u32)
                    .unwrap_or(0);
                loop {
                    if bottom_amount_left > self.length_texture_src.w.get() {
                        txt.copy(
                            self.length_texture_src,
                            TextureDestination(
                                TextureRect {
                                    x: x_offset,
                                    y: pos.y + pos.h.get() as i32 - border_width,
                                    w: self.length_texture_src.w,
                                    h: self.length_texture_src.h,
                                },
                                Some(TextureRotation {
                                    angle: 0.try_into().unwrap(),
                                    point: None,
                                    flip_horizontal: false,
                                    flip_vertical: true,
                                }),
                                Color {
                                    r: 0xFF,
                                    g: 0xFF,
                                    b: 0xFF,
                                    a: 0xFF,
                                },
                            ),
                        )?;
                        x_offset += self.length_texture_src.w.get() as i32;
                        bottom_amount_left -= self.length_texture_src.w.get();
                    } else {
                        match NonZeroU32::new(bottom_amount_left) {
                            Some(bottom_amount_left) => {
                                txt.copy(
                                    TextureRect {
                                        x: self.length_texture_src.x,
                                        y: self.length_texture_src.y,
                                        w: bottom_amount_left,
                                        h: self.length_texture_src.h,
                                    },
                                    TextureDestination(
                                        TextureRect {
                                            x: x_offset,
                                            y: pos.y + pos.h.get() as i32
                                                - self.border_width() as i32,
                                            w: bottom_amount_left,
                                            h: self.length_texture_src.h,
                                        },
                                        Some(TextureRotation {
                                            angle: 0.try_into().unwrap(),
                                            point: None,
                                            flip_horizontal: false,
                                            flip_vertical: true,
                                        }),
                                        Color {
                                            r: 0xFF,
                                            g: 0xFF,
                                            b: 0xFF,
                                            a: 0xFF,
                                        },
                                    ),
                                )?;
                            }
                            _ => {}
                        };
                        break;
                    }
                }
            }

            if self.right {
                let mut y_offset = pos.y + t_border_width;
                let mut right_amount_left = pos
                    .h
                    .get()
                    .checked_sub(t_border_width as u32 + b_border_width as u32)
                    .unwrap_or(0);
                loop {
                    if right_amount_left > self.length_texture_src.w.get() {
                        txt.copy(
                            self.length_texture_src,
                            TextureDestination(
                                TextureRect {
                                    x: pos.x + pos.w.get() as i32,
                                    y: y_offset,
                                    w: self.length_texture_src.w,
                                    h: self.length_texture_src.h,
                                },
                                Some(TextureRotation {
                                    angle: 90.try_into().unwrap(),
                                    point: Some((0, 0)),
                                    flip_horizontal: false,
                                    flip_vertical: false,
                                }),
                                Color {
                                    r: 0xFF,
                                    g: 0xFF,
                                    b: 0xFF,
                                    a: 0xFF,
                                },
                            ),
                        )?;
                        y_offset += self.length_texture_src.w.get() as i32;
                        right_amount_left -= self.length_texture_src.w.get();
                    } else {
                        match NonZeroU32::new(right_amount_left) {
                            Some(right_amount_left) => {
                                txt.copy(
                                    TextureRect {
                                        x: self.length_texture_src.x,
                                        y: self.length_texture_src.y,
                                        w: right_amount_left,
                                        h: self.length_texture_src.h,
                                    },
                                    TextureDestination(
                                        TextureRect {
                                            x: pos.x + pos.w.get() as i32,
                                            y: y_offset,
                                            w: right_amount_left,
                                            h: self.length_texture_src.h,
                                        },
                                        Some(TextureRotation {
                                            angle: 90.try_into().unwrap(),
                                            point: Some((0, 0)),
                                            flip_horizontal: false,
                                            flip_vertical: false,
                                        }),
                                        Color {
                                            r: 0xFF,
                                            g: 0xFF,
                                            b: 0xFF,
                                            a: 0xFF,
                                        },
                                    ),
                                )?;
                            }
                            _ => {}
                        };
                        break;
                    }
                }
            }

            if self.left {
                let mut y_offset = pos.y + t_border_width;
                let mut left_amount_left = pos
                    .h
                    .get()
                    .checked_sub(t_border_width as u32 + b_border_width as u32)
                    .unwrap_or(0);
                loop {
                    if left_amount_left > self.length_texture_src.w.get() {
                        txt.copy(
                            self.length_texture_src,
                            TextureDestination(
                                TextureRect {
                                    x: pos.x + self.border_width() as i32,
                                    y: y_offset,
                                    w: self.length_texture_src.w,
                                    h: self.length_texture_src.h,
                                },
                                Some(TextureRotation {
                                    angle: 90.try_into().unwrap(),
                                    point: Some((0, 0)),
                                    flip_horizontal: false,
                                    flip_vertical: true,
                                }),
                                Color {
                                    r: 0xFF,
                                    g: 0xFF,
                                    b: 0xFF,
                                    a: 0xFF,
                                },
                            ),
                        )?;
                        y_offset += self.length_texture_src.w.get() as i32;
                        left_amount_left -= self.length_texture_src.w.get();
                    } else {
                        match NonZeroU32::new(left_amount_left) {
                            Some(left_amount_left) => {
                                txt.copy(
                                    TextureRect {
                                        x: self.length_texture_src.x,
                                        y: self.length_texture_src.y,
                                        w: left_amount_left,
                                        h: self.length_texture_src.h,
                                    },
                                    TextureDestination(
                                        TextureRect {
                                            x: pos.x + self.border_width() as i32,
                                            y: y_offset,
                                            w: left_amount_left,
                                            h: self.length_texture_src.h,
                                        },
                                        Some(TextureRotation {
                                            angle: 90.try_into().unwrap(),
                                            point: Some((0, 0)),
                                            flip_horizontal: false,
                                            flip_vertical: true,
                                        }),
                                        Color {
                                            r: 0xFF,
                                            g: 0xFF,
                                            b: 0xFF,
                                            a: 0xFF,
                                        },
                                    ),
                                )?;
                            }
                            _ => {}
                        };
                        break;
                    }
                }
            }

            // corners
            if self.top && self.right {
                txt.copy(
                    self.corner_texture_src,
                    TextureRect {
                        x: pos.x + pos.w.get() as i32 - border_width,
                        y: pos.y,
                        w: self.corner_texture_src.w,
                        h: self.corner_texture_src.h,
                    },
                )?;
            }

            if self.top && self.left {
                txt.copy(
                    self.corner_texture_src,
                    TextureDestination(
                        TextureRect {
                            x: pos.x,
                            y: pos.y,
                            w: self.corner_texture_src.w,
                            h: self.corner_texture_src.h,
                        },
                        Some(TextureRotation {
                            angle: 0.try_into().unwrap(),
                            point: None,
                            flip_horizontal: true,
                            flip_vertical: false,
                        }),
                        Color {
                            r: 0xFF,
                            g: 0xFF,
                            b: 0xFF,
                            a: 0xFF,
                        },
                    ),
                )?;
            }

            if self.bottom && self.right {
                txt.copy(
                    self.corner_texture_src,
                    TextureDestination(
                        TextureRect {
                            x: pos.x + pos.w.get() as i32 - border_width,
                            y: pos.y + pos.h.get() as i32 - border_width,
                            w: self.corner_texture_src.w,
                            h: self.corner_texture_src.h,
                        },
                        Some(TextureRotation {
                            angle: 0.try_into().unwrap(),
                            point: None,
                            flip_horizontal: false,
                            flip_vertical: true,
                        }),
                        Color {
                            r: 0xFF,
                            g: 0xFF,
                            b: 0xFF,
                            a: 0xFF,
                        },
                    ),
                )?;
            }

            if self.bottom && self.left {
                txt.copy(
                    self.corner_texture_src,
                    TextureDestination(
                        TextureRect {
                            x: pos.x,
                            y: pos.y + pos.h.get() as i32 - border_width,
                            w: self.corner_texture_src.w,
                            h: self.corner_texture_src.h,
                        },
                        Some(TextureRotation {
                            angle: 0.try_into().unwrap(),
                            point: None,
                            flip_horizontal: true,
                            flip_vertical: true,
                        }),
                        Color {
                            r: 0xFF,
                            g: 0xFF,
                            b: 0xFF,
                            a: 0xFF,
                        },
                    ),
                )?;
            }
        }
        Ok(())
    }
}
