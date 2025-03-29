use std::{num::NonZeroU32, path::PathBuf};

use crate::{
    core::{
        texture_area::{TextureArea, TextureSource},
        Texture,
    },
    ui::util::{
        length::{MaxLen, MaxLenFailPolicy, MinLen, MinLenFailPolicy, PreferredPortion},
        rect::FRect,
    },
};

use super::{
    sizing::NestedContentSizing,
    Widget, WidgetUpdateEvent,
};

/// contains:
///  - optional background texture
///  - content
pub struct Background<'font_data, 'b, T: crate::core::System<'font_data> + 'b> {
    /// background texture and source area - it will be tiled to fill the
    /// available space
    pub background: Option<(PathBuf, TextureSource)>,

    // background scaling will NOT be supported due to poor backend handling:
    //
    // pub scale: NonZeroU32,
    //
    // - fractional source coordinate not supported in sdl2
    //   https://github.com/libsdl-org/SDL/pull/7384
    // - a workaround would be to draw the texture big and use a clipping
    //   rectangle but they fucked up basic functionality in some renderers
    //   https://github.com/libsdl-org/SDL/issues/12658
    //
    // in lieu of a scale, just scale up the underlying background texture
    pub contained: Box<dyn Widget<'font_data, T> + 'b>,
    pub sizing: NestedContentSizing,

    /// state stored from update for draw
    background_draw_pos: FRect,
}

impl<'font_data, 'b, T: crate::core::System<'font_data> + 'b> Background<'font_data, 'b, T> {
    pub fn new(
        background: Option<(PathBuf, TextureSource)>,
        contained: Box<dyn Widget<'font_data, T> + 'b>,
    ) -> Self {
        Self {
            background,
            contained,
            sizing: Default::default(),
            background_draw_pos: Default::default(),
        }
    }
}

impl<'font_data, 'b, T: crate::core::System<'font_data> + 'b> Widget<'font_data, T>
    for Background<'font_data, 'b, T>
{
    fn update(&mut self, mut event: WidgetUpdateEvent, sys_interface: &mut T) -> Result<(), String> {
        self.background_draw_pos = event.position;
        self.sizing.update_contained(self.contained.as_mut(), &mut event, sys_interface)
    }

    fn draw(&self, sys_interface: &mut T) -> Result<(), String> {
        if let Some((txt_path, maybe_txt_src)) = &self.background {
            let pos: Option<TextureArea> = self.background_draw_pos.into();
            if let Some(pos) = pos {
                let pos_width = pos.w.get() as i32;
                let pos_height = pos.h.get() as i32;

                let mut txt = sys_interface.texture(&txt_path)?;

                let txt_size_safe = match maybe_txt_src {
                    TextureSource::WholeTexture => txt.size()?,
                    TextureSource::Area(texture_area) => (texture_area.w, texture_area.h),
                };

                let txt_size = (txt_size_safe.0.get() as i32, txt_size_safe.1.get() as i32);

                let txt_position = match maybe_txt_src {
                    TextureSource::WholeTexture => (0, 0),
                    TextureSource::Area(texture_area) => (texture_area.x, texture_area.y),
                };

                // loop through and draw 2d grid of the background texture to
                // fill the available space, cutting off at the right and
                // bottom. it can probably be simplified:
                let mut y_start = 0;
                loop {
                    let height_left = pos_height - y_start;

                    if height_left >= txt_size.1 {
                        // enough space for whole height
                        let mut x_start = 0;
                        loop {
                            let width_left = pos_width - x_start;
                            if width_left >= txt_size.0 {
                                // enough space for whole tile
                                txt.copy(
                                    TextureArea {
                                        x: txt_position.0,
                                        y: txt_position.1,
                                        w: txt_size_safe.0,
                                        h: txt_size_safe.1,
                                    },
                                    TextureArea {
                                        x: x_start + pos.x,
                                        y: y_start + pos.y,
                                        w: txt_size_safe.0,
                                        h: txt_size_safe.1,
                                    },
                                )?;
                            } else {
                                // not enough space for whole width
                                if let Some(width_left_safe) = NonZeroU32::new(width_left as u32) {
                                    txt.copy(
                                        TextureArea {
                                            x: txt_position.0,
                                            y: txt_position.1,
                                            w: width_left_safe,
                                            h: txt_size_safe.1,
                                        },
                                        TextureArea {
                                            x: x_start + pos.x,
                                            y: y_start + pos.y,
                                            w: width_left_safe,
                                            h: txt_size_safe.1,
                                        },
                                    )?;
                                };
                                break;
                            }
                            x_start += txt_size.0;
                        }
                    } else {
                        if let Some(height_left_safe) = NonZeroU32::new(height_left as u32) {
                            // not enough space for whole height
                            let mut x_start = 0;
                            loop {
                                let width_left = pos_width - x_start;
                                if width_left >= txt_size.0 {
                                    // enough for width
                                    txt.copy(
                                        TextureArea {
                                            x: txt_position.0,
                                            y: txt_position.1,
                                            w: txt_size_safe.0,
                                            h: height_left_safe,
                                        },
                                        TextureArea {
                                            x: x_start + pos.x,
                                            y: y_start + pos.y,
                                            w: txt_size_safe.0,
                                            h: height_left_safe,
                                        },
                                    )?;
                                } else {
                                    // not enough space for whole width
                                    if let Some(width_left_safe) =
                                        NonZeroU32::new(width_left as u32)
                                    {
                                        txt.copy(
                                            TextureArea {
                                                x: txt_position.0,
                                                y: txt_position.1,
                                                w: width_left_safe,
                                                h: height_left_safe,
                                            },
                                            TextureArea {
                                                x: x_start + pos.x,
                                                y: y_start + pos.y,
                                                w: width_left_safe,
                                                h: height_left_safe,
                                            },
                                        )?;
                                    };
                                    break;
                                }
                                x_start += txt_size.0;
                            }
                        }
                        break;
                    }
                    y_start += txt_size.1;
                }
            }
        }
        self.contained.draw(sys_interface)
    }

    fn min(&self, sys_interface: &mut T) -> Result<(MinLen, MinLen), String> {
        self.sizing.min(self.contained.as_ref(), sys_interface)
    }

    fn min_w_fail_policy(&self) -> MinLenFailPolicy {
        self.sizing.min_w_fail_policy(self.contained.as_ref())
    }

    fn min_h_fail_policy(&self) -> MinLenFailPolicy {
        self.sizing.min_h_fail_policy(self.contained.as_ref())
    }

    fn max(&self, sys_interface: &mut T) -> Result<(MaxLen, MaxLen), String> {
        self.sizing.max(self.contained.as_ref(), sys_interface)
    }

    fn max_w_fail_policy(&self) -> MaxLenFailPolicy {
        self.sizing.max_w_fail_policy(self.contained.as_ref())
    }

    fn max_h_fail_policy(&self) -> MaxLenFailPolicy {
        self.sizing.max_h_fail_policy(self.contained.as_ref())
    }

    fn preferred_portion(&self) -> (PreferredPortion, PreferredPortion) {
        self.sizing.preferred_portion(self.contained.as_ref())
    }

    fn preferred_width_from_height(
        &self,
        pref_h: f32,
        sys_interface: &mut T,
    ) -> Option<Result<f32, String>> {
        self.sizing
            .preferred_width_from_height(self.contained.as_ref(), pref_h, sys_interface)
    }

    fn preferred_height_from_width(
        &self,
        pref_w: f32,
        sys_interface: &mut T,
    ) -> Option<Result<f32, String>> {
        self.sizing
            .preferred_height_from_width(self.contained.as_ref(), pref_w, sys_interface)
    }

    fn preferred_link_allowed_exceed_portion(&self) -> bool {
        self.sizing
            .preferred_link_allowed_exceed_portion(self.contained.as_ref())
    }
}
