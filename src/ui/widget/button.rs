use std::cell::Cell;

use crate::{
    core::texture_rect::TextureRect,
    ui::util::length::{MaxLen, MaxLenFailPolicy, MinLen, MinLenFailPolicy, PreferredPortion},
};

use super::{sizing::NestedContentSizing, Widget, WidgetUpdateEvent};

enum ButtonState {
    Idle,
    Hovered,
    Pressed,
}

/// if NestedContentSizing::Inherit, which contained widget should be used for
/// sizing
#[derive(Debug, Clone, Copy, Default)]
pub enum ButtonInheritSizing {
    Idle,
    Hovered,
    Pressed,
    #[default]
    Current,
}

pub struct Button<'font_data, 'b, 'state, T: crate::core::System<'font_data> + 'b> {
    pub idle: Box<dyn Widget<'font_data, T> + 'b>,
    pub hovered: Box<dyn Widget<'font_data, T> + 'b>,
    pub pressed: Box<dyn Widget<'font_data, T> + 'b>,

    pub sizing: NestedContentSizing,
    pub sizing_inherit_choice: ButtonInheritSizing,

    /// use this to implement functionality
    pub released: &'state Cell<bool>,

    /// state stored for draw from update
    state: ButtonState,
}

impl<'font_data, 'b, 'state, T: crate::core::System<'font_data> + 'b>
    Button<'font_data, 'b, 'state, T>
{
    pub fn new(
        idle: Box<dyn Widget<'font_data, T> + 'b>,
        hovered: Box<dyn Widget<'font_data, T> + 'b>,
        pressed: Box<dyn Widget<'font_data, T> + 'b>,
        released: &'state Cell<bool>,
    ) -> Self {
        Button {
            idle,
            hovered,
            pressed,
            released,
            state: ButtonState::Idle,
            sizing: Default::default(),
            sizing_inherit_choice: Default::default(),
        }
    }

    fn current_widget(&self) -> &dyn Widget<'font_data, T> {
        match self.state {
            ButtonState::Idle => self.idle.as_ref(),
            ButtonState::Hovered => self.hovered.as_ref(),
            ButtonState::Pressed => self.pressed.as_ref(),
        }
    }

    fn current_widget_mut(&mut self) -> &mut dyn Widget<'font_data, T> {
        match self.state {
            ButtonState::Idle => self.idle.as_mut(),
            ButtonState::Hovered => self.hovered.as_mut(),
            ButtonState::Pressed => self.pressed.as_mut(),
        }
    }

    fn inherit_sizing_widget(&self) -> &dyn Widget<'font_data, T> {
        match self.sizing_inherit_choice {
            ButtonInheritSizing::Idle => self.idle.as_ref(),
            ButtonInheritSizing::Hovered => self.hovered.as_ref(),
            ButtonInheritSizing::Pressed => self.pressed.as_ref(),
            ButtonInheritSizing::Current => self.current_widget(),
        }
    }
}

impl<'font_data, 'b, 'state, T: crate::core::System<'font_data> + 'b> Widget<'font_data, T>
    for Button<'font_data, 'b, 'state, T>
{
    fn preferred_ratio_exceed_parent(&self) -> bool {
        self.sizing
            .preferred_link_allowed_exceed_portion(self.inherit_sizing_widget())
        // self.preferred_link_allowed_exceed_portion
    }

    fn min(&self, sys_interface: &mut T) -> Result<(MinLen, MinLen), String> {
        self.sizing.min(self.inherit_sizing_widget(), sys_interface)
    }

    fn min_w_fail_policy(&self) -> MinLenFailPolicy {
        self.sizing.min_w_fail_policy(self.inherit_sizing_widget())
    }

    fn min_h_fail_policy(&self) -> MinLenFailPolicy {
        self.sizing.min_h_fail_policy(self.inherit_sizing_widget())
    }

    fn max(&self, sys_interface: &mut T) -> Result<(MaxLen, MaxLen), String> {
        self.sizing.max(self.inherit_sizing_widget(), sys_interface)
    }

    fn max_w_fail_policy(&self) -> MaxLenFailPolicy {
        self.sizing.max_w_fail_policy(self.inherit_sizing_widget())
    }

    fn max_h_fail_policy(&self) -> MaxLenFailPolicy {
        self.sizing.max_h_fail_policy(self.inherit_sizing_widget())
    }

    fn preferred_portion(&self) -> (PreferredPortion, PreferredPortion) {
        self.sizing.preferred_portion(self.inherit_sizing_widget())
    }

    fn preferred_width_from_height(
        &self,
        pref_h: f32,
        sys_interface: &mut T,
    ) -> Option<Result<f32, String>> {
        self.sizing
            .preferred_width_from_height(self.inherit_sizing_widget(), pref_h, sys_interface)
    }

    fn preferred_height_from_width(
        &self,
        pref_w: f32,
        sys_interface: &mut T,
    ) -> Option<Result<f32, String>> {
        self.sizing
            .preferred_height_from_width(self.inherit_sizing_widget(), pref_w, sys_interface)
    }

    fn update(
        &mut self,
        mut event: WidgetUpdateEvent,
        sys_interface: &mut T,
    ) -> Result<bool, String> {
        self.released.set(false);
        let non_zero_area: TextureRect = match event.position.into() {
            Some(v) => v,
            None => return Ok(false), // can't click or hover with zero area
        };
        for e in event.events.iter_mut().filter(|e| e.available()) {
            match e.e {
                crate::core::event::Event::Mouse(mouse) => {
                    if non_zero_area.contains_point((mouse.x, mouse.y))
                        && event.clipping_rect.contains_point((mouse.x, mouse.y))
                    {
                        if !mouse.down {
                            if mouse.changed {
                                // on falling edge
                                e.set_consumed();
                                self.released.set(true);
                            }
                            self.state = ButtonState::Hovered;
                        } else {
                            e.set_consumed();
                            self.state = ButtonState::Pressed;
                        }
                    } else {
                        self.state = ButtonState::Idle;
                    }
                }
                _ => {}
            }
        }

        let sizing = self.sizing;
        sizing.update_contained(self.current_widget_mut(), &mut event, sys_interface)?;
        Ok(false)
    }

    fn draw(&self, sys_interface: &mut T) -> Result<(), String> {
        self.current_widget().draw(sys_interface)?;
        Ok(())
    }
}
