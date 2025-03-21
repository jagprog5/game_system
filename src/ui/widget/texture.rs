use std::{
    num::NonZeroU32,
    ops::Not,
    path::{Path, PathBuf},
};

use crate::{
    core::texture_area::TextureArea,
    ui::util::length::{
        AspectRatioPreferredDirection, MaxLen, MaxLenFailPolicy, MaxLenPolicy, MinLen,
        MinLenFailPolicy, MinLenPolicy, PreferredPortion,
    },
};

use super::{Widget, WidgetUpdateEvent};

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

pub struct Texture {
    /// path to this image. passed to core system interface
    texture_path: PathBuf,
    /// set when the texture path is set
    texture_size: (NonZeroU32, NonZeroU32),

    /// none means use the entire texture
    pub texture_src: Option<TextureArea>,

    /// how should the texture be stretched / sized if the aspect ratio is not
    /// respected
    pub aspect_ratio_fail_policy: AspectRatioFailPolicy,

    pub request_aspect_ratio: bool,

    pub min_w_fail_policy: MinLenFailPolicy,
    pub max_w_fail_policy: MaxLenFailPolicy,
    pub min_h_fail_policy: MinLenFailPolicy,
    pub max_h_fail_policy: MaxLenFailPolicy,
    pub min_w_policy: MinLenPolicy,
    pub max_w_policy: MaxLenPolicy,
    pub min_h_policy: MinLenPolicy,
    pub max_h_policy: MaxLenPolicy,
    pub pref_w: PreferredPortion,
    pub pref_h: PreferredPortion,
    pub preferred_link_allowed_exceed_portion: bool,

    /// state stored for draw from update
    draw_pos: crate::ui::util::rect::FRect,
}

impl Texture {
    pub fn new<'a, T: crate::core::System<'a>>(
        texture_path: &Path,
        sys_interface: &mut T,
    ) -> Result<Texture, String> {
        let mut ret = Texture {
            texture_path: "".to_owned().into(), // set below
            texture_size: unsafe { (NonZeroU32::new_unchecked(1), NonZeroU32::new_unchecked(1)) },
            texture_src: Default::default(),
            aspect_ratio_fail_policy: Default::default(),
            request_aspect_ratio: true,
            min_w_fail_policy: Default::default(),
            max_w_fail_policy: Default::default(),
            min_h_fail_policy: Default::default(),
            max_h_fail_policy: Default::default(),
            min_w_policy: Default::default(),
            max_w_policy: Default::default(),
            min_h_policy: Default::default(),
            max_h_policy: Default::default(),
            pref_w: Default::default(),
            pref_h: Default::default(),
            preferred_link_allowed_exceed_portion: Default::default(),
            draw_pos: Default::default(),
        };
        ret.set_texture_path(texture_path, sys_interface)?;
        Ok(ret)
    }

    pub fn texture_path(&self) -> &Path {
        &self.texture_path
    }

    pub fn set_texture_path<'a, T: crate::core::System<'a>>(
        &mut self,
        texture_path: &Path,
        sys_interface: &mut T,
    ) -> Result<(), String> {
        let txt = sys_interface.texture(texture_path)?;
        let query = crate::core::Texture::size(&txt)?;
        self.texture_path = texture_path.to_path_buf();
        self.texture_size = query;
        Ok(())
    }
}

impl<'a, T: crate::core::System<'a>> Widget<'a, T> for Texture {
    fn preferred_link_allowed_exceed_portion(&self) -> bool {
        self.preferred_link_allowed_exceed_portion
    }

    fn min(&mut self) -> Result<(MinLen, MinLen), String> {
        if let MinLenPolicy::Literal(w) = self.min_w_policy {
            if let MinLenPolicy::Literal(h) = self.min_h_policy {
                return Ok((w, h)); // no need to query texture
            }
        }
        let query = self.texture_size;
        Ok((
            match self.min_w_policy {
                MinLenPolicy::Children => MinLen(query.0.get() as f32),
                MinLenPolicy::Literal(min_len) => min_len,
            },
            match self.min_h_policy {
                MinLenPolicy::Children => MinLen(query.1.get() as f32),
                MinLenPolicy::Literal(min_len) => min_len,
            },
        ))
    }

    fn min_w_fail_policy(&self) -> MinLenFailPolicy {
        self.min_w_fail_policy
    }

    fn min_h_fail_policy(&self) -> MinLenFailPolicy {
        self.min_h_fail_policy
    }

    fn max(&mut self) -> Result<(MaxLen, MaxLen), String> {
        if let MaxLenPolicy::Literal(w) = self.max_w_policy {
            if let MaxLenPolicy::Literal(h) = self.max_h_policy {
                return Ok((w, h)); // no need to query texture
            }
        }
        let query = self.texture_size;
        Ok((
            match self.max_w_policy {
                MaxLenPolicy::Children => MaxLen(query.0.get() as f32),
                MaxLenPolicy::Literal(max_len) => max_len,
            },
            match self.max_h_policy {
                MaxLenPolicy::Children => MaxLen(query.1.get() as f32),
                MaxLenPolicy::Literal(max_len) => max_len,
            },
        ))
    }

    fn max_w_fail_policy(&self) -> MaxLenFailPolicy {
        self.max_w_fail_policy
    }

    fn max_h_fail_policy(&self) -> MaxLenFailPolicy {
        self.max_h_fail_policy
    }

    fn preferred_portion(&self) -> (PreferredPortion, PreferredPortion) {
        (self.pref_w, self.pref_h)
    }

    fn preferred_width_from_height(&mut self, pref_h: f32) -> Option<Result<f32, String>> {
        if self.request_aspect_ratio.not() {
            return None;
        }

        let ratio = self.texture_size.0.get() as f32 / self.texture_size.1.get() as f32;
        Some(Ok(AspectRatioPreferredDirection::width_from_height(
            ratio, pref_h,
        )))
    }

    fn preferred_height_from_width(&mut self, pref_w: f32) -> Option<Result<f32, String>> {
        if self.request_aspect_ratio.not() {
            return None;
        }

        let ratio = self.texture_size.0.get() as f32 / self.texture_size.1.get() as f32;
        Some(Ok(AspectRatioPreferredDirection::height_from_width(
            ratio, pref_w,
        )))
    }

    fn update(&mut self, event: WidgetUpdateEvent) -> Result<(), String> {
        self.draw_pos = event.position;
        Ok(())
    }

    fn update_adjust_position(&mut self, pos_delta: (i32, i32)) {
        self.draw_pos.x += pos_delta.0 as f32;
        self.draw_pos.y += pos_delta.1 as f32;
    }

    fn draw(&mut self, sys_interface: &mut T) -> Result<(), String> {
        texture_draw(
            &self.texture_path,
            self.texture_size,
            &self.aspect_ratio_fail_policy,
            sys_interface,
            self.texture_src,
            self.draw_pos,
        )
    }
}

pub(crate) fn texture_draw<'a, T: crate::core::System<'a>>(
    texture_path: &Path,
    texture_size: (NonZeroU32, NonZeroU32),
    aspect_ratio_fail_policy: &AspectRatioFailPolicy,
    sys_interface: &mut T,
    src: Option<TextureArea>,
    dst: crate::ui::util::rect::FRect,
) -> Result<(), String> {
    // dst is kept as float form until just before canvas copy. needed or else
    // it is jumpy

    let mut texture = sys_interface.texture(&texture_path)?;

    let (src_x, src_y, src_w, src_h) = match src {
        None => {
            (0, 0, texture_size.0, texture_size.1)
        }
        Some(v) => (v.x, v.y, v.w, v.h),
    };

    match aspect_ratio_fail_policy {
        AspectRatioFailPolicy::Stretch => {
            let dst: TextureArea = match dst.into() {
                None => return Ok(()), // can't draw zero size
                Some(v) => v,
            };
            crate::core::Texture::copy(&mut texture, src, dst)
        }
        AspectRatioFailPolicy::ZoomOut((zoom_x, zoom_y)) => {
            let src_w = src_w.get() as f32;
            let src_h = src_h.get() as f32;
            let src_aspect_ratio = src_w / src_h;
            if dst.h == 0. {
                return Ok(()); // guard div + can't drawn zero area texture
            }
            let dst_aspect_ratio = dst.w / dst.h;

            if src_aspect_ratio > dst_aspect_ratio {
                // padding at the top and bottom; scale down the size of the
                // src so the width matches the destination
                let scale_down = dst.w / src_w;
                let dst_width = (src_w * scale_down).round() as u32;
                let dst_height = (src_h * scale_down).round() as u32;

                let dst_width: NonZeroU32 = match dst_width.try_into() {
                    Ok(v) => v,
                    Err(_) => return Ok(()), // zoomed out too much
                };

                let dst_height: NonZeroU32 = match dst_height.try_into() {
                    Ok(v) => v,
                    Err(_) => return Ok(()), // zoomed out too much
                };

                let dst_y_offset = ((dst.h - dst_height.get() as f32) * zoom_y).round() as i32;
                crate::core::Texture::copy(
                    &mut texture,
                    src,
                    TextureArea {
                        x: dst.x.round() as i32,
                        y: dst.y.round() as i32 + dst_y_offset,
                        w: dst_width,
                        h: dst_height,
                    },
                )
            } else {
                // padding at the left and right; scale down the size of the
                // src so the height matches the destination
                let scale_down = dst.h / src_h;
                let dst_width = (src_w * scale_down).round() as u32;
                let dst_height = (src_h * scale_down).round() as u32;

                let dst_width: NonZeroU32 = match dst_width.try_into() {
                    Ok(v) => v,
                    Err(_) => return Ok(()), // zoomed out too much
                };

                let dst_height: NonZeroU32 = match dst_height.try_into() {
                    Ok(v) => v,
                    Err(_) => return Ok(()), // zoomed out too much
                };

                let dst_x_offset = ((dst.w - dst_width.get() as f32) * zoom_x) as i32;
                crate::core::Texture::copy(
                    &mut texture,
                    src,
                    TextureArea {
                        x: dst.x.round() as i32 + dst_x_offset,
                        y: dst.y.round() as i32,
                        w: dst_width,
                        h: dst_height,
                    },
                )
            }
        }
        AspectRatioFailPolicy::ZoomIn((zoom_x, zoom_y)) => {
            let dst_txt_area: TextureArea = match dst.into() {
                None => return Ok(()), // can't draw zero size
                Some(v) => v,
            };
            let src_w_f = src_w.get() as f32;
            let src_h_f = src_h.get() as f32;

            let src_aspect_ratio = src_w_f / src_h_f;
            let dst_aspect_ratio = dst.w / dst.h; // guarded above by dst_sdl2 into

            if src_aspect_ratio > dst_aspect_ratio {
                let width = (dst_aspect_ratio * src_h_f).round() as u32;
                let width: NonZeroU32 = match width.try_into() {
                    Ok(v) => v,
                    Err(_) => return Ok(()) // too extreme of a ratio
                };

                let x = ((src_w_f - width.get() as f32) * zoom_x) as i32;
                crate::core::Texture::copy(
                    &mut texture,
                    TextureArea {
                        x: src_x + x,
                        y: src_y,
                        w: width,
                        h: src_h,
                    },
                    dst_txt_area,
                )
            } else {
                //                     V guarded above by dst_sdl2 into
                let height = ((src_w_f / dst.w) * dst.h).round() as u32;
                let height: NonZeroU32 = match height.try_into() {
                    Ok(v) => v,
                    Err(_) => return Ok(()) // too extreme of a ratio
                };
                let y = ((src_h_f - height.get() as f32) * zoom_y) as i32;
                crate::core::Texture::copy(
                    &mut texture,
                    TextureArea {
                        x: src_x,
                        y: src_y + y,
                        w: src_w,
                        h: height,
                    },
                    dst_txt_area,
                )
            }
        }
    }
}
