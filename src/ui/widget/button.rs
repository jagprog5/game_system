use std::cell::Cell;

use crate::{
    core::texture_rect::TextureRect,
    ui::util::{
        length::{MaxLen, MaxLenFailPolicy, MinLen, MinLenFailPolicy, PreferredPortion},
        rust::CellRefOrCell,
    },
};

use super::{sizing::NestedContentSizing, Widget, WidgetUpdateEvent};

#[derive(Default, Clone, Copy)]
enum ButtonState {
    #[default]
    Idle,
    Hovered,
    Pressed,
}

/// an internal state for a button. generally this should persist between frames
/// but it's not necessary for most button content
#[derive(Default, Clone, Copy)]
pub struct ButtonPrivateState {
    s: ButtonState,
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

pub struct Button<'b, 'state, T: crate::core::System + 'b> {
    pub idle: Box<dyn Widget<T> + 'b>,
    pub hovered: Box<dyn Widget<T> + 'b>,
    pub pressed: Box<dyn Widget<T> + 'b>,

    pub sizing: NestedContentSizing,
    pub sizing_inherit_choice: ButtonInheritSizing,

    /// use this to implement functionality
    pub released: &'state Cell<bool>,

    /// a button which can be used to press the button
    pub hotkey: Option<u8>,

    /// state stored for draw from update. under some circumstances this needs
    /// to persist between frames. for example, if the contained button content
    /// has an animation. but otherwise, the state is set appropriately when
    /// events are received each frame and so persisting it isn't necessary
    pub state: CellRefOrCell<'state, ButtonPrivateState>,
}

impl<'b, 'state, T: crate::core::System + 'b> Button<'b, 'state, T> {
    pub fn new(
        idle: Box<dyn Widget<T> + 'b>,
        hovered: Box<dyn Widget<T> + 'b>,
        pressed: Box<dyn Widget<T> + 'b>,
        released: &'state Cell<bool>,
    ) -> Self {
        Button {
            idle,
            hovered,
            pressed,
            released,
            hotkey: None,
            state: CellRefOrCell::Cell(Cell::new(Default::default())),
            sizing: Default::default(),
            sizing_inherit_choice: Default::default(),
        }
    }

    fn current_widget(&self) -> &dyn Widget<T> {
        match self.state.get().s {
            ButtonState::Idle => self.idle.as_ref(),
            ButtonState::Hovered => self.hovered.as_ref(),
            ButtonState::Pressed => self.pressed.as_ref(),
        }
    }

    fn current_widget_mut(&mut self) -> &mut dyn Widget<T> {
        match self.state.get().s {
            ButtonState::Idle => self.idle.as_mut(),
            ButtonState::Hovered => self.hovered.as_mut(),
            ButtonState::Pressed => self.pressed.as_mut(),
        }
    }

    fn inherit_sizing_widget(&self) -> &dyn Widget<T> {
        match self.sizing_inherit_choice {
            ButtonInheritSizing::Idle => self.idle.as_ref(),
            ButtonInheritSizing::Hovered => self.hovered.as_ref(),
            ButtonInheritSizing::Pressed => self.pressed.as_ref(),
            ButtonInheritSizing::Current => self.current_widget(),
        }
    }
}

impl<'b, 'state, T: crate::core::System + 'b> Widget<T> for Button<'b, 'state, T> {
    fn preferred_ratio_exceed_parent(&self) -> bool {
        self.sizing
            .preferred_ratio_exceed_parent(self.inherit_sizing_widget())
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
                crate::core::event::Event::Key(key_event) => {
                    if let Some(hotkey) = self.hotkey {
                        if key_event.key == hotkey {
                            e.set_consumed();
                            if key_event.down {
                                self.state.set(ButtonPrivateState {
                                    s: ButtonState::Pressed,
                                });
                            } else {
                                // rising edge
                                self.released.set(true);
                                self.state.set(ButtonPrivateState {
                                    s: ButtonState::Idle,
                                });
                            }
                        }
                    }
                }
                crate::core::event::Event::Mouse(mouse) => {
                    if non_zero_area.contains_point((mouse.x, mouse.y))
                        && event.clipping_rect.contains_point((mouse.x, mouse.y))
                    {
                        if mouse.changed {
                            e.set_consumed();
                        }
                        if !mouse.down {
                            if mouse.changed {
                                // rising edge
                                self.released.set(true);
                            }
                            self.state.set(ButtonPrivateState {
                                s: ButtonState::Hovered,
                            });
                        } else {
                            self.state.set(ButtonPrivateState {
                                s: ButtonState::Pressed,
                            });
                        }
                    } else {
                        self.state.set(ButtonPrivateState {
                            s: ButtonState::Idle,
                        });
                    }
                }
                _ => {}
            }
        }

        let sizing = self.sizing;
        sizing.update_contained(self.current_widget_mut(), &mut event, sys_interface)
    }

    fn draw(&self, sys_interface: &mut T) -> Result<(), String> {
        self.current_widget().draw(sys_interface)?;
        Ok(())
    }
}
