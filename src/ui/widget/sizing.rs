use crate::ui::util::{
    length::{
        AspectRatioPreferredDirection, MaxLen, MaxLenFailPolicy, MinLen, MinLenFailPolicy,
        PreferredPortion,
    },
    rect::FRect,
};

use super::{place, Widget, WidgetUpdateEvent};

#[derive(Debug, Clone, Copy, Default)]
pub struct CustomSizing {
    pub min_w: MinLen,
    pub min_h: MinLen,
    pub max_w: MaxLen,
    pub max_h: MaxLen,
    pub preferred_w: PreferredPortion,
    pub preferred_h: PreferredPortion,
    pub preferred_aspect_ratio: Option<f32>,
    pub preferred_link_allowed_exceed_portion: bool,

    pub min_w_fail_policy: MinLenFailPolicy,
    pub max_w_fail_policy: MaxLenFailPolicy,
    pub min_h_fail_policy: MinLenFailPolicy,
    pub max_h_fail_policy: MaxLenFailPolicy,
}

impl CustomSizing {
    pub fn preferred_width_from_height(&self, pref_h: f32) -> Option<f32> {
        let ratio = match &self.preferred_aspect_ratio {
            None => return None,
            Some(v) => v,
        };

        Some(AspectRatioPreferredDirection::width_from_height(
            *ratio, pref_h,
        ))
    }

    pub fn preferred_height_from_width(&self, pref_w: f32) -> Option<f32> {
        let ratio = match &self.preferred_aspect_ratio {
            None => return None,
            Some(v) => v,
        };

        Some(AspectRatioPreferredDirection::height_from_width(
            *ratio, pref_w,
        ))
    }
}

#[derive(Default, Clone, Copy)]
pub enum NestedContentSizing {
    /// the parent inherits the sizing info from the contained thing
    #[default]
    Inherit,
    /// the parent's size is stated literally, ignoring the contained thing
    ///
    /// if a widget is contained, it may place the contained within the draw
    /// bounds of the parent widget
    ///
    /// if something else is contained, then it's up to that widget to decide
    /// how best to handle it
    Custom(CustomSizing),
}

impl NestedContentSizing {
    /// get the position that would be used to update the contained if
    /// update_contained were to be called
    pub fn position_for_contained<'font_data, T: crate::core::System<'font_data>>(
        &self,
        contained: &dyn Widget<'font_data, T>,
        event: &WidgetUpdateEvent,
        sys_interface: &mut T,
    ) -> Result<FRect, String> {
        match &self {
            NestedContentSizing::Inherit => {
                // exactly passes sizing information to parent in this
                // case, no need to place again
                Ok(event.position)
            }
            NestedContentSizing::Custom(_) => {
                // whatever the sizing of the parent, properly place the
                // contained within it
                place(
                    contained,
                    event.position,
                    AspectRatioPreferredDirection::default(),
                    sys_interface,
                )
            }
        }
    }

    pub fn update_contained<'font_data, T: crate::core::System<'font_data>>(
        &self,
        contained: &mut dyn Widget<'font_data, T>,
        event: &mut WidgetUpdateEvent,
        sys_interface: &mut T,
    ) -> Result<bool, String> {
        let position_for_contained =
            self.position_for_contained(contained, event, sys_interface)?;
        let event_for_contained = event.sub_event(position_for_contained);
        contained.update(event_for_contained, sys_interface)
    }

    pub fn min<'font_data, T: crate::core::System<'font_data>>(
        &self,
        contained: &dyn Widget<'font_data, T>,
        sys_interface: &mut T,
    ) -> Result<(MinLen, MinLen), String> {
        match self {
            NestedContentSizing::Inherit => contained.min(sys_interface),
            NestedContentSizing::Custom(custom) => Ok((custom.min_w, custom.min_h)),
        }
    }

    pub fn max<'font_data, T: crate::core::System<'font_data>>(
        &self,
        contained: &dyn Widget<'font_data, T>,
        sys_interface: &mut T,
    ) -> Result<(MaxLen, MaxLen), String> {
        match self {
            NestedContentSizing::Inherit => contained.max(sys_interface),
            NestedContentSizing::Custom(custom) => Ok((custom.max_w, custom.max_h)),
        }
    }

    pub fn preferred_portion<'font_data, T: crate::core::System<'font_data>>(
        &self,
        contained: &dyn Widget<'font_data, T>,
    ) -> (PreferredPortion, PreferredPortion) {
        match self {
            NestedContentSizing::Inherit => contained.preferred_portion(),
            NestedContentSizing::Custom(custom) => (custom.preferred_w, custom.preferred_h),
        }
    }

    pub fn preferred_width_from_height<'font_data, T: crate::core::System<'font_data>>(
        &self,
        contained: &dyn Widget<'font_data, T>,
        pref_h: f32,
        sys_interface: &mut T,
    ) -> Option<Result<f32, String>> {
        match &self {
            NestedContentSizing::Inherit => {
                contained.preferred_width_from_height(pref_h, sys_interface)
            }
            NestedContentSizing::Custom(custom) => {
                custom.preferred_width_from_height(pref_h).map(|a| Ok(a))
            }
        }
    }

    pub fn preferred_height_from_width<'font_data, T: crate::core::System<'font_data>>(
        &self,
        contained: &dyn Widget<'font_data, T>,
        pref_w: f32,
        sys_interface: &mut T,
    ) -> Option<Result<f32, String>> {
        match &self {
            NestedContentSizing::Inherit => {
                contained.preferred_height_from_width(pref_w, sys_interface)
            }
            NestedContentSizing::Custom(custom) => {
                custom.preferred_height_from_width(pref_w).map(|a| Ok(a))
            }
        }
    }

    pub fn preferred_link_allowed_exceed_portion<'font_data, T: crate::core::System<'font_data>>(
        &self,
        contained: &dyn Widget<'font_data, T>,
    ) -> bool {
        match self {
            NestedContentSizing::Inherit => contained.preferred_ratio_exceed_parent(),
            NestedContentSizing::Custom(s) => s.preferred_link_allowed_exceed_portion,
        }
    }

    pub fn min_w_fail_policy<'font_data, T: crate::core::System<'font_data>>(
        &self,
        contained: &dyn Widget<'font_data, T>,
    ) -> MinLenFailPolicy {
        match &self {
            NestedContentSizing::Inherit => contained.min_w_fail_policy(),
            NestedContentSizing::Custom(custom) => custom.min_w_fail_policy,
        }
    }

    pub fn max_w_fail_policy<'font_data, T: crate::core::System<'font_data>>(
        &self,
        contained: &dyn Widget<'font_data, T>,
    ) -> MaxLenFailPolicy {
        match &self {
            NestedContentSizing::Inherit => contained.max_w_fail_policy(),
            NestedContentSizing::Custom(custom) => custom.max_w_fail_policy,
        }
    }

    pub fn min_h_fail_policy<'font_data, T: crate::core::System<'font_data>>(
        &self,
        contained: &dyn Widget<'font_data, T>,
    ) -> MinLenFailPolicy {
        match &self {
            NestedContentSizing::Inherit => contained.min_h_fail_policy(),
            NestedContentSizing::Custom(custom) => custom.min_h_fail_policy,
        }
    }

    pub fn max_h_fail_policy<'font_data, T: crate::core::System<'font_data>>(
        &self,
        contained: &dyn Widget<'font_data, T>,
    ) -> MaxLenFailPolicy {
        match &self {
            NestedContentSizing::Inherit => contained.max_h_fail_policy(),
            NestedContentSizing::Custom(custom) => custom.max_h_fail_policy,
        }
    }
}
