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

    // scale not supported. for details, see
    // game_system::ui::widget::background::Background
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
        }
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
        let sub_amount = self.length_texture_src.h.get() * 2; // * 2 for each side
        let sub_amount = sub_amount as f32;
        // subtract border width from the pref input before passing to the
        // contained widget. then, add it back after getting the result
        let (amount_subtracted, pref_h) = if sub_amount >= pref_h {
            // atypical case (guard against subtract into negative range)
            (pref_h, 0.)
        } else {
            // typical case
            (sub_amount, pref_h - sub_amount)
        };
        self.contained
            .preferred_width_from_height(pref_h, sys_interface)
            .map(|some| some.map(|ok| ok + amount_subtracted))
    }

    fn preferred_height_from_width(
        &self,
        pref_w: f32,
        sys_interface: &mut T,
    ) -> Option<Result<f32, String>> {
        let sub_amount = self.length_texture_src.h.get() * 2; // * 2 for each side
        let sub_amount = sub_amount as f32;
        // subtract border width from the pref input before passing to the
        // contained widget. then, add it back after getting the result
        let (amount_subtracted, pref_w) = if sub_amount >= pref_w {
            // atypical case (guard against subtract into negative range)
            (pref_w, 0.)
        } else {
            // typical case
            (sub_amount, pref_w - sub_amount)
        };
        self.contained
            .preferred_height_from_width(pref_w, sys_interface)
            .map(|some| some.map(|ok| ok + amount_subtracted))
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
        let sub_amount = self.length_texture_src.h.get() * 2; // * 2 for each side
        let baseline = MinLen(sub_amount as f32);
        let m = self.contained.min(sys_interface)?;
        Ok((m.0.combined(baseline), m.1.combined(baseline)))
    }

    fn max(&self, sys_interface: &mut T) -> Result<(MaxLen, MaxLen), String> {
        let sub_amount = self.length_texture_src.h.get() * 2; // * 2 for each side
        let baseline = MaxLen(sub_amount as f32);
        let m = self.contained.max(sys_interface)?;
        Ok((m.0.combined(baseline), m.1.combined(baseline)))
    }

    fn update(
        &mut self,
        mut event: WidgetUpdateEvent,
        sys_interface: &mut T,
    ) -> Result<bool, String> {
        self.border_draw_pos = event.position;
        let style_width = (self.length_texture_src.h.get()) as f32;
        let position_for_child = crate::ui::util::rect::FRect {
            x: event.position.x + style_width,
            y: event.position.y + style_width,
            w: event.position.w - style_width * 2.,
            h: event.position.h - style_width * 2., // deliberately allow negative
        };
        self.contained
            .update(event.sub_event(position_for_child), sys_interface)
    }

    fn draw(&self, sys_interface: &mut T) -> Result<(), String> {
        self.contained.draw(sys_interface)?;

        let maybe_pos: Option<TextureRect> = self.border_draw_pos.into();

        if let Some(pos) = maybe_pos {
            // draw border if non empty position
            let mut txt = sys_interface.texture(&self.texture_path)?;
            {
                // top
                let mut x_offset = pos.x + self.length_texture_src.h.get() as i32;
                let mut top_amount_left = pos.w.get() - self.length_texture_src.h.get() * 2;
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

            {
                // bottom
                let mut x_offset = pos.x + self.length_texture_src.h.get() as i32;
                let mut bottom_amount_left = pos.w.get() - self.length_texture_src.h.get() * 2;
                loop {
                    if bottom_amount_left > self.length_texture_src.w.get() {
                        txt.copy(
                            self.length_texture_src,
                            TextureDestination(
                                TextureRect {
                                    x: x_offset,
                                    y: pos.y + pos.h.get() as i32
                                        - self.length_texture_src.h.get() as i32,
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
                                                - self.length_texture_src.h.get() as i32,
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

            {
                // right
                let mut y_offset = pos.y + self.length_texture_src.h.get() as i32;
                let mut right_amount_left = pos.h.get() - self.length_texture_src.h.get() * 2;
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

            {
                // left
                let mut y_offset = pos.y + self.length_texture_src.h.get() as i32;
                let mut left_amount_left = pos.h.get() - self.length_texture_src.h.get() * 2;
                loop {
                    if left_amount_left > self.length_texture_src.w.get() {
                        txt.copy(
                            self.length_texture_src,
                            TextureDestination(
                                TextureRect {
                                    x: pos.x + self.length_texture_src.h.get() as i32,
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
                                            x: pos.x + self.length_texture_src.h.get() as i32,
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
            txt.copy(
                self.corner_texture_src,
                TextureRect {
                    x: pos.x + pos.w.get() as i32 - self.length_texture_src.h.get() as i32,
                    y: pos.y,
                    w: self.corner_texture_src.w,
                    h: self.corner_texture_src.h,
                },
            )?;

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

            txt.copy(
                self.corner_texture_src,
                TextureDestination(
                    TextureRect {
                        x: pos.x + pos.w.get() as i32 - self.length_texture_src.h.get() as i32,
                        y: pos.y + pos.h.get() as i32 - self.length_texture_src.h.get() as i32,
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

            txt.copy(
                self.corner_texture_src,
                TextureDestination(
                    TextureRect {
                        x: pos.x,
                        y: pos.y + pos.h.get() as i32 - self.length_texture_src.h.get() as i32,
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
        Ok(())
    }
}
