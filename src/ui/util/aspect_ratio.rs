use crate::core::texture_rect::TextureRectF;

use super::rect::FRect;

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

impl AspectRatioFailPolicy {
    /// return the src and dst to use, respectively
    pub fn get(
        &self,
        src: TextureRectF,
        dst: crate::ui::util::rect::FRect,
    ) -> Option<(TextureRectF, TextureRectF)> {
        match self {
            AspectRatioFailPolicy::Stretch => {
                let dst: Option<TextureRectF> = dst.into();
                match dst {
                    Some(dst) => Some((src.into(), dst)),
                    None => None,
                }
            }
            AspectRatioFailPolicy::ZoomOut((zoom_x, zoom_y)) => {
                let src_aspect_ratio = src.w.get() / src.h.get();
                if dst.h == 0. {
                    return None; // guard div + can't drawn zero area texture
                }
                let dst_aspect_ratio = dst.w / dst.h;

                let maybe_dst: Option<TextureRectF> = if src_aspect_ratio > dst_aspect_ratio {
                    // padding at the top and bottom; scale down the size of the
                    // src so the width matches the destination
                    let scale_down = dst.w / src.w.get();
                    let dst_width = src.w.get() * scale_down;
                    let dst_height = src.h.get() * scale_down;
                    let dst_y_offset = (dst.h - dst_height) * zoom_y;
                    let maybe_dst = crate::ui::util::rect::FRect {
                        x: dst.x,
                        y: dst.y + dst_y_offset,
                        w: dst_width,
                        h: dst_height,
                    };

                    maybe_dst.into()
                } else {
                    // padding at the left and right; scale down the size of the
                    // src so the height matches the destination
                    let scale_down = dst.h / src.h.get();
                    let dst_width = src.w.get() * scale_down;
                    let dst_height = src.h.get() * scale_down;
                    let dst_x_offset = (dst.w - dst_width) * zoom_x;

                    let maybe_dst = crate::ui::util::rect::FRect {
                        x: dst.x + dst_x_offset,
                        y: dst.y,
                        w: dst_width,
                        h: dst_height,
                    };

                    maybe_dst.into()
                };
                match maybe_dst {
                    Some(dst) => Some((src.into(), dst)),
                    None => None,
                }
            }
            AspectRatioFailPolicy::ZoomIn((zoom_x, zoom_y)) => {
                let src_aspect_ratio = src.w.get() / src.h.get();
                if dst.h == 0. || dst.w == 0. {
                    return None; // guard div + can't drawn zero area texture
                }
                let dst_aspect_ratio = dst.w / dst.h;

                let maybe_src: Option<TextureRectF> = if src_aspect_ratio > dst_aspect_ratio {
                    let width = dst_aspect_ratio * src.h.get();
                    let x = (src.w.get() - width) * zoom_x;
                    FRect {
                        x: src.x.get() + x,
                        y: src.y.get(),
                        w: width,
                        h: src.h.get(),
                    }
                    .into()
                } else {
                    let height = (src.w.get() / dst.w) * dst.h;
                    let y = (src.h.get() - height) * zoom_y;
                    FRect {
                        x: src.x.get(),
                        y: src.y.get() + y,
                        w: src.w.get(),
                        h: height,
                    }
                    .into()
                };

                let maybe_dst: Option<TextureRectF> = dst.into();
                match maybe_dst {
                    Some(dst) => match maybe_src {
                        Some(src) => Some((src, dst)),
                        None => None,
                    },
                    None => None,
                }
            }
        }
    }
}
