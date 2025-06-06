use crate::ui::{
    util::length::{MaxLen, MaxLenFailPolicy, MinLen, MinLenFailPolicy, PreferredPortion},
    widget::Widget,
};

/// might contain something and inherit the contained's sizing. if it does not,
/// then this widget has default flexible sizing
pub struct Slot<'b, T: crate::core::System + 'b> {
    pub contained: Option<Box<dyn Widget<T> + 'b>>,
}

impl<'b, T: crate::core::System> Default for Slot<'b, T> {
    fn default() -> Self {
        Self {
            contained: Default::default(),
        }
    }
}

impl<'b, T: crate::core::System + 'b> Widget<T> for Slot<'b, T> {
    fn min(&self, sys_interface: &mut T) -> Result<(MinLen, MinLen), String> {
        self.contained
            .as_ref()
            .map(|c| c.min(sys_interface))
            .unwrap_or(Ok(Default::default()))
    }

    fn min_w_fail_policy(&self) -> MinLenFailPolicy {
        self.contained
            .as_ref()
            .map(|c| c.min_w_fail_policy())
            .unwrap_or(Default::default())
    }

    fn min_h_fail_policy(&self) -> MinLenFailPolicy {
        self.contained
            .as_ref()
            .map(|c| c.min_h_fail_policy())
            .unwrap_or(Default::default())
    }

    fn max(&self, sys_interface: &mut T) -> Result<(MaxLen, MaxLen), String> {
        self.contained
            .as_ref()
            .map(|c| c.max(sys_interface))
            .unwrap_or(Ok(Default::default()))
    }

    fn max_w_fail_policy(&self) -> MaxLenFailPolicy {
        self.contained
            .as_ref()
            .map(|c| c.max_w_fail_policy())
            .unwrap_or(Default::default())
    }

    fn max_h_fail_policy(&self) -> MaxLenFailPolicy {
        self.contained
            .as_ref()
            .map(|c| c.max_h_fail_policy())
            .unwrap_or(Default::default())
    }

    fn preferred_portion(&self) -> (PreferredPortion, PreferredPortion) {
        self.contained
            .as_ref()
            .map(|c| c.preferred_portion())
            .unwrap_or(Default::default())
    }

    fn preferred_width_from_height(
        &self,
        pref_h: f32,
        sys_interface: &mut T,
    ) -> Option<Result<f32, String>> {
        self.contained
            .as_ref()
            .map(|c| c.preferred_width_from_height(pref_h, sys_interface))
            .unwrap_or(Default::default())
    }

    fn preferred_height_from_width(
        &self,
        pref_w: f32,
        sys_interface: &mut T,
    ) -> Option<Result<f32, String>> {
        self.contained
            .as_ref()
            .map(|c| c.preferred_height_from_width(pref_w, sys_interface))
            .unwrap_or(Default::default())
    }

    fn preferred_ratio_exceed_parent(&self) -> bool {
        self.contained
            .as_ref()
            .map(|c| c.preferred_ratio_exceed_parent())
            .unwrap_or(Default::default())
    }

    fn update(
        &mut self,
        event: super::WidgetUpdateEvent,
        sys_interface: &mut T,
    ) -> Result<bool, String> {
        self.contained
            .as_mut()
            .map(|c| c.update(event, sys_interface))
            .unwrap_or(Ok(false))
    }

    fn draw(&self, sys_interface: &mut T) -> Result<(), String> {
        self.contained
            .as_ref()
            .map(|c| c.draw(sys_interface))
            .unwrap_or(Ok(()))
    }
}
