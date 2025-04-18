use std::{ops::Not, path::PathBuf};

use crate::{
    core::{
        color::Color,
        texture_rect::{TextureDestinationF, TextureRect, TextureRectF, TextureSource},
        TextureHandle,
    },
    ui::util::{
        aspect_ratio::AspectRatioFailPolicy,
        length::{
            AspectRatioPreferredDirection, MaxLen, MaxLenFailPolicy, MaxLenPolicy, MinLen,
            MinLenFailPolicy, MinLenPolicy, PreferredPortion,
        },
        rect::FRect,
    },
};

use super::{Widget, WidgetUpdateEvent};

#[derive(Clone)]
pub struct Texture {
    pub texture_path: PathBuf,
    pub texture_src: TextureSource,
    pub color_mod: Color,

    /// applicable when request_aspect_ratio is false
    pub aspect_ratio_fail_policy: AspectRatioFailPolicy,

    /// default: true
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
    pub preferred_ratio_exceed_parent: bool,

    /// state stored for draw from update
    draw_pos: crate::ui::util::rect::FRect,
}

impl Texture {
    pub fn new(texture_path: PathBuf) -> Self {
        Texture {
            texture_path: texture_path.to_path_buf(),
            texture_src: Default::default(),
            color_mod: Color {
                r: 0xFF,
                g: 0xFF,
                b: 0xFF,
                a: 0xFF,
            },
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
            preferred_ratio_exceed_parent: Default::default(),
            draw_pos: Default::default(),
        }
    }
}

impl<'a, T: crate::core::System<'a>> Widget<'a, T> for Texture {
    fn preferred_ratio_exceed_parent(&self) -> bool {
        self.preferred_ratio_exceed_parent
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

        let src = match self.texture_src {
            TextureSource::WholeTexture => {
                let texture_size = texture.size()?;
                TextureRect {
                    x: 0,
                    y: 0,
                    w: texture_size.0,
                    h: texture_size.1,
                }
            }
            TextureSource::Area(texture_rect) => texture_rect,
        };

        let maybe_src_dst = self.aspect_ratio_fail_policy.get(src.into(), self.draw_pos);
        if let Some((src, dst)) = maybe_src_dst {
            // snap dst to grid
            let dst: FRect = dst.into();
            let maybe_dst: Option<TextureRect> = dst.into();
            if let Some(dst) = maybe_dst {
                let dst: TextureRectF = dst.into();
                let dst = TextureDestinationF(dst, None, self.color_mod);
                texture.copy_f(src, dst)?;
            }
        }

        Ok(())
    }
}
