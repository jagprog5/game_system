use std::{num::NonZeroU32, ops::Not, path::PathBuf};

use typed_floats::{NonNaNFinite, StrictlyPositiveFinite};

use crate::{
    core::{
        color::Color,
        texture_area::{
            TextureAreaF, TextureDestination, TextureDestinationF, TextureRect, TextureSource,
        },
    },
    ui::util::{
        length::{
            AspectRatioPreferredDirection, MaxLen, MaxLenFailPolicy, MaxLenPolicy, MinLen,
            MinLenFailPolicy, MinLenPolicy, PreferredPortion,
        },
        rect::rect_position_round,
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
    pub texture_path: PathBuf,
    pub texture_src: TextureSource,

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
    pub fn new(texture_path: PathBuf) -> Self {
        Texture {
            texture_path: texture_path.to_path_buf(),
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
        }
    }
}

impl<'a, T: crate::core::System<'a>> Widget<'a, T> for Texture {
    fn preferred_ratio_exceed_parent(&self) -> bool {
        self.preferred_link_allowed_exceed_portion
    }

    fn min(&self, sys_interface: &mut T) -> Result<(MinLen, MinLen), String> {
        if let MinLenPolicy::Literal(w) = self.min_w_policy {
            if let MinLenPolicy::Literal(h) = self.min_h_policy {
                return Ok((w, h)); // no need to query texture
            }
        }

        let size = match self.texture_src {
            TextureSource::WholeTexture => {
                let texture = sys_interface.texture(&self.texture_path)?;
                crate::core::TextureHandle::size(&texture)?
            }
            TextureSource::Area(texture_area) => texture_area.size(),
        };
        Ok((
            match self.min_w_policy {
                MinLenPolicy::Children => MinLen(size.0.get() as f32),
                MinLenPolicy::Literal(min_len) => min_len,
            },
            match self.min_h_policy {
                MinLenPolicy::Children => MinLen(size.1.get() as f32),
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

    fn max(&self, sys_interface: &mut T) -> Result<(MaxLen, MaxLen), String> {
        if let MaxLenPolicy::Literal(w) = self.max_w_policy {
            if let MaxLenPolicy::Literal(h) = self.max_h_policy {
                return Ok((w, h)); // no need to query texture
            }
        }
        let size = match self.texture_src {
            TextureSource::WholeTexture => {
                let texture = sys_interface.texture(&self.texture_path)?;
                crate::core::TextureHandle::size(&texture)?
            }
            TextureSource::Area(texture_area) => texture_area.size(),
        };
        Ok((
            match self.max_w_policy {
                MaxLenPolicy::Children => MaxLen(size.0.get() as f32),
                MaxLenPolicy::Literal(max_len) => max_len,
            },
            match self.max_h_policy {
                MaxLenPolicy::Children => MaxLen(size.1.get() as f32),
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

    fn preferred_width_from_height(
        &self,
        pref_h: f32,
        sys_interface: &mut T,
    ) -> Option<Result<f32, String>> {
        if self.request_aspect_ratio.not() {
            return None;
        }

        Some(|| -> Result<f32, String> {
            let size = match self.texture_src {
                TextureSource::WholeTexture => {
                    let texture = sys_interface.texture(&self.texture_path)?;
                    crate::core::TextureHandle::size(&texture)?
                }
                TextureSource::Area(texture_area) => texture_area.size(),
            };

            let ratio = size.0.get() as f32 / size.1.get() as f32;
            Ok(AspectRatioPreferredDirection::width_from_height(
                ratio, pref_h,
            ))
        }())
    }

    fn preferred_height_from_width(
        &self,
        pref_w: f32,
        sys_interface: &mut T,
    ) -> Option<Result<f32, String>> {
        if self.request_aspect_ratio.not() {
            return None;
        }

        Some(|| -> Result<f32, String> {
            let size = match self.texture_src {
                TextureSource::WholeTexture => {
                    let texture = sys_interface.texture(&self.texture_path)?;
                    crate::core::TextureHandle::size(&texture)?
                }
                TextureSource::Area(texture_area) => texture_area.size(),
            };

            let ratio = size.0.get() as f32 / size.1.get() as f32;
            Ok(AspectRatioPreferredDirection::height_from_width(
                ratio, pref_w,
            ))
        }())
    }

    fn update(&mut self, event: WidgetUpdateEvent, _sys_interface: &mut T) -> Result<bool, String> {
        self.draw_pos = event.position;
        Ok(false)
    }

    fn draw(&self, sys_interface: &mut T) -> Result<(), String> {
        let mut texture = sys_interface.texture(&self.texture_path)?;
        texture_draw(
            &mut texture,
            Color {
                r: 0xFF,
                g: 0xFF,
                b: 0xFF,
                a: 0xFF,
            },
            &self.aspect_ratio_fail_policy,
            self.texture_src,
            self.draw_pos,
        )
    }
}

pub(crate) fn texture_draw<'a>(
    texture: &mut impl crate::core::TextureHandle<'a>,
    color_mod: Color,
    aspect_ratio_fail_policy: &AspectRatioFailPolicy,
    src: TextureSource,
    dst: crate::ui::util::rect::FRect,
) -> Result<(), String> {
    // dst is kept as float form until just before canvas copy. needed or else
    // it is jumpy

    let texture_size = crate::core::TextureHandle::size(texture)?;

    let (src_x, src_y, src_w, src_h) = match src {
        TextureSource::WholeTexture => (0, 0, texture_size.0, texture_size.1),
        TextureSource::Area(v) => (v.x, v.y, v.w, v.h),
    };

    match aspect_ratio_fail_policy {
        AspectRatioFailPolicy::Stretch => {
            let dst: TextureRect = match dst.into() {
                None => return Ok(()), // can't draw zero size
                Some(v) => v,
            };
            crate::core::TextureHandle::copy(texture, src, TextureDestination(dst, None, color_mod))
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
                crate::core::TextureHandle::copy(
                    texture,
                    src,
                    TextureDestination(
                        TextureRect {
                            x: rect_position_round(dst.x),
                            y: rect_position_round(dst.y) + dst_y_offset,
                            w: dst_width,
                            h: dst_height,
                        },
                        None,
                        color_mod,
                    ),
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
                crate::core::TextureHandle::copy(
                    texture,
                    src,
                    TextureDestination(
                        TextureRect {
                            x: rect_position_round(dst.x) + dst_x_offset,
                            y: rect_position_round(dst.y),
                            w: dst_width,
                            h: dst_height,
                        },
                        None,
                        color_mod,
                    ),
                )
            }
        }
        AspectRatioFailPolicy::ZoomIn((zoom_x, zoom_y)) => {
            // ensure that destination is ok in terms of INF, NAN, etc. this
            // instance is used for division below because there's no div by 0
            let dst_safe: TextureAreaF = match dst.into() {
                None => return Ok(()), // can't draw zero size
                Some(v) => v,
            };

            // this is where the texture is actually copied to
            let dst_actual: TextureRect = match dst.into() {
                None => return Ok(()), // can't draw zero size
                Some(v) => v,
            };
            // the src is using floating point prevision, but the destination
            // still snaps to grid
            let dst_actual: TextureAreaF = dst_actual.into();
            let dst = ();
            let _dst = dst; // don't use this one any more

            let src_w_f = src_w.get() as f32;
            let src_h_f = src_h.get() as f32;

            // div guarded by type
            let src_aspect_ratio = src_w_f / src_h_f;
            let dst_aspect_ratio = dst_safe.w.get() / dst_safe.h.get();

            // copy_f required here or else if it jumpy for small textures
            if src_aspect_ratio > dst_aspect_ratio {
                let width = dst_aspect_ratio * src_h_f;
                let x = (src_w_f - width) * zoom_x;

                // requires check because zoom_x could be weird
                let x_arg =
                    NonNaNFinite::<f32>::new(src_x as f32 + x).map_err(|e| e.to_string())?;
                // safe because i32 always maps to ok f32
                let y_arg = unsafe { NonNaNFinite::<f32>::new_unchecked(src_y as f32) };
                let width_arg = match StrictlyPositiveFinite::<f32>::new(width) {
                    Ok(v) => Ok(v),
                    Err(e) => {
                        if let typed_floats::InvalidNumber::Zero = e {
                            return Ok(()); // too extreme of a ratio
                        } else {
                            Err(e)
                        }
                    }
                }
                .map_err(|e| e.to_string())?;

                // safe because always ok from NonZeroU32
                let height_arg = unsafe { StrictlyPositiveFinite::<f32>::new_unchecked(src_h_f) };

                crate::core::TextureHandle::copy_f(
                    texture,
                    TextureAreaF {
                        x: x_arg,
                        y: y_arg,
                        w: width_arg,
                        h: height_arg,
                    },
                    TextureDestinationF(dst_actual, None, color_mod),
                )
            } else {
                //                     V guarded above by dst_sdl2 into
                let height = (src_w_f / dst_safe.w.get()) * dst_safe.h.get();
                let y = (src_h_f - height) * zoom_y;

                let height_arg = match StrictlyPositiveFinite::<f32>::new(height) {
                    Ok(v) => Ok(v),
                    Err(e) => {
                        if let typed_floats::InvalidNumber::Zero = e {
                            return Ok(()); // too extreme of a ratio
                        } else {
                            Err(e)
                        }
                    }
                }
                .map_err(|e| e.to_string())?;
                let y_arg =
                    NonNaNFinite::<f32>::new(src_y as f32 + y).map_err(|e| e.to_string())?;

                let x_arg = unsafe { NonNaNFinite::<f32>::new_unchecked(src_x as f32) };

                let width_arg = unsafe { StrictlyPositiveFinite::<f32>::new_unchecked(src_w_f) };

                crate::core::TextureHandle::copy_f(
                    texture,
                    TextureAreaF {
                        x: x_arg,
                        y: y_arg,
                        w: width_arg,
                        h: height_arg,
                    },
                    TextureDestinationF(dst_actual, None, color_mod),
                )
            }
        }
    }
}
