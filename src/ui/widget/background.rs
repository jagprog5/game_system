use crate::ui::widget::FrameTransiency;

use super::{sizing::NestedContentSizing, Widget};

/// has a contained thing, and something which is drawn behind it (background)
/// or in front of it (foreground)
///
/// the sizing of this widget is entirely inherited from the contained widget.
/// the background widget's sizing is ignore, and is generally assumed to be lax in its sizing requirements
pub struct Background<'b, T: crate::core::System + 'b> {
    pub contained: Box<dyn Widget<T> + 'b>,

    /// default: true. drawn behind contained. false draws in front of as
    /// contained like an overlay
    pub is_background: bool,

    pub background: Box<dyn Widget<T> + 'b>,

    pub sizing: NestedContentSizing,
}

impl<'b, T: crate::core::System + 'b> Background<'b, T> {
    pub fn new(contained: Box<dyn Widget<T> + 'b>, background: Box<dyn Widget<T> + 'b>) -> Self {
        Self {
            contained,
            is_background: true,
            background,
            sizing: Default::default(),
        }
    }
}

impl<'b, T: crate::core::System + 'b> Widget<T> for Background<'b, T> {
    fn min(
        &self,
        sys_interface: &mut T,
    ) -> Result<
        (
            crate::ui::util::length::MinLen,
            crate::ui::util::length::MinLen,
        ),
        String,
    > {
        self.sizing.min(self.contained.as_ref(), sys_interface)
    }

    fn min_w_fail_policy(&self) -> crate::ui::util::length::MinLenFailPolicy {
        self.sizing.min_w_fail_policy(self.contained.as_ref())
    }

    fn min_h_fail_policy(&self) -> crate::ui::util::length::MinLenFailPolicy {
        self.sizing.min_h_fail_policy(self.contained.as_ref())
    }

    fn max(
        &self,
        sys_interface: &mut T,
    ) -> Result<
        (
            crate::ui::util::length::MaxLen,
            crate::ui::util::length::MaxLen,
        ),
        String,
    > {
        self.sizing.max(self.contained.as_ref(), sys_interface)
    }

    fn max_w_fail_policy(&self) -> crate::ui::util::length::MaxLenFailPolicy {
        self.sizing.max_w_fail_policy(self.contained.as_ref())
    }

    fn max_h_fail_policy(&self) -> crate::ui::util::length::MaxLenFailPolicy {
        self.sizing.max_h_fail_policy(self.contained.as_ref())
    }

    fn preferred_portion(
        &self,
    ) -> (
        crate::ui::util::length::PreferredPortion,
        crate::ui::util::length::PreferredPortion,
    ) {
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

    fn preferred_ratio_exceed_parent(&self) -> bool {
        self.sizing
            .preferred_ratio_exceed_parent(self.contained.as_ref())
    }

    fn update(
        &mut self,
        mut event: super::WidgetUpdateEvent,
        sys_interface: &mut T,
    ) -> Result<FrameTransiency, String> {
        Ok(self
            .sizing
            .update_contained(self.contained.as_mut(), &mut event, sys_interface)?
            | self
                .sizing
                .update_contained(self.background.as_mut(), &mut event, sys_interface)?)
    }

    fn draw(&self, sys_interface: &mut T) -> Result<(), String> {
        if self.is_background {
            self.background.draw(sys_interface)?;
        }

        self.contained.draw(sys_interface)?;

        if !self.is_background {
            self.background.draw(sys_interface)?;
        }
        Ok(())
    }
}
