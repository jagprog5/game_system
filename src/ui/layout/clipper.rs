use crate::{core::{clipping_rect::ClippingArea, texture_area::TextureArea}, ui::{util::rect::FRect, widget::Widget}};

/// contains something. when it is draw, a clipping rect is set to not allow
/// drawing to go past the widget's given position
pub struct Clipper<'a, T: crate::core::System<'a>> {
    pub contained: Box<dyn Widget<'a, T>>,
    /// calculated during update, stored for draw.
    ///
    /// this is the clipping rect that should be applied before drawing
    update_clip_rect: ClippingArea,
}

impl<'a, T: crate::core::System<'a>> Clipper<'a, T> {
    pub fn new(contained: Box<dyn Widget<'a, T>>) -> Self {
        Self {
            contained,
            update_clip_rect: ClippingArea::None, // doesn't matter here
        }
    }
}

impl<'a, T: crate::core::System<'a>> Widget<'a, T> for Clipper<'a, T> {
    fn update(
        &mut self,
        mut event: crate::ui::widget::WidgetUpdateEvent,
    ) -> Result<(), String> {
        let previous_clipping_rect = event.clipping_rect;
        // store for update step
        let maybe_texture_area: Option<TextureArea> = event.position.into();



        self.update_clip_rect = previous_clipping_rect
        // set clipping rect in dup as to not affect any widgets that might come
        // after this one
        let mut event_dup = event.dup();
        event_dup.clipping_rect = self.update_clip_rect;
        self.contained.update(event.dup())
    }

    fn update_adjust_position(&mut self, pos_delta: (i32, i32)) {
        if let ClippingArea::Some(rect) = &mut self.update_clip_rect {
            rect.x += pos_delta.0;
            rect.y += pos_delta.1;
        }
        self.contained.update_adjust_position(pos_delta);
    }

    fn draw(
        &mut self,
        sys_interface: &mut T,
    ) -> Result<(), String> {
        let previous_clipping_rect = sys_interface.get_clip();
        sys_interface.clip(self.update_clip_rect);
        let ret = self.contained.draw(sys_interface);
        // reset clipping rect for following elements that will be drawn after
        sys_interface.clip(previous_clipping_rect);
        ret
    }

    fn min(
        &mut self
    ) -> Result<(crate::ui::util::length::MinLen, crate::ui::util::length::MinLen), String> {
        self.contained.min()
    }

    fn min_w_fail_policy(&self) -> crate::ui::util::length::MinLenFailPolicy {
        self.contained.min_w_fail_policy()
    }

    fn min_h_fail_policy(&self) -> crate::ui::util::length::MinLenFailPolicy {
        self.contained.min_h_fail_policy()
    }

    fn max(
        &mut self
    ) -> Result<(crate::ui::util::length::MaxLen, crate::ui::util::length::MaxLen), String> {
        self.contained.max()
    }

    fn max_w_fail_policy(&self) -> crate::ui::util::length::MaxLenFailPolicy {
        self.contained.max_w_fail_policy()
    }

    fn max_h_fail_policy(&self) -> crate::ui::util::length::MaxLenFailPolicy {
        self.contained.max_h_fail_policy()
    }

    fn preferred_portion(
        &self,
    ) -> (
        crate::ui::util::length::PreferredPortion,
        crate::ui::util::length::PreferredPortion,
    ) {
        self.contained.preferred_portion()
    }

    fn preferred_width_from_height(&mut self, pref_h: f32) -> Option<Result<f32, String>> {
        self.contained.preferred_width_from_height(pref_h)
    }

    fn preferred_height_from_width(&mut self, pref_w: f32) -> Option<Result<f32, String>> {
        self.contained.preferred_height_from_width(pref_w)
    }

    fn preferred_link_allowed_exceed_portion(&self) -> bool {
        self.contained.preferred_link_allowed_exceed_portion()
    }
}
