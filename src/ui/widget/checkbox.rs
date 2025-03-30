use std::{cell::Cell, path::PathBuf};

use crate::{
    core::{texture_area::TextureRect, TextureHandle},
    ui::util::{
        length::{MaxLen, MinLen},
        rect::FRect,
    },
};

use super::Widget;

pub struct CheckBox<'state> {
    pub texture_path: PathBuf,
    /// square
    pub min: MinLen,
    /// square
    pub max: MaxLen,

    pub toggle_sound: Option<PathBuf>,

    pub check: TextureRect,
    pub check_faded: TextureRect,
    pub uncheck: TextureRect,
    pub uncheck_faded: TextureRect,

    pub checked: &'state Cell<bool>,
    pub changed: &'state Cell<bool>,

    /// state stored for draw from update
    draw_pos: FRect,
    hovered: bool,
}

impl<'state> CheckBox<'state> {
    pub fn new(
        texture_path: PathBuf,
        min: MinLen,
        max: MaxLen,
        checked: &'state Cell<bool>,
        changed: &'state Cell<bool>,
        check: TextureRect,
        check_faded: TextureRect,
        uncheck: TextureRect,
        uncheck_faded: TextureRect,
    ) -> Self {
        Self {
            texture_path,
            min,
            max,
            check,
            check_faded,
            uncheck,
            uncheck_faded,
            checked,
            changed,
            draw_pos: Default::default(),
            hovered: false,
            toggle_sound: None,
        }
    }
}

impl<'state, 'a, T: crate::core::System<'a>> Widget<'a, T> for CheckBox<'state> {
    fn min(&self, _sys_interface: &mut T) -> Result<(MinLen, MinLen), String> {
        Ok((self.min, self.min))
    }

    fn max(&self, _sys_interface: &mut T) -> Result<(MaxLen, MaxLen), String> {
        Ok((self.max, self.max))
    }

    fn preferred_ratio_exceed_parent(&self) -> bool {
        true // always be square
    }

    fn preferred_width_from_height(&self, pref_h: f32, _s: &mut T) -> Option<Result<f32, String>> {
        Some(Ok(pref_h))
    }

    fn preferred_height_from_width(&self, pref_w: f32, _s: &mut T) -> Option<Result<f32, String>> {
        Some(Ok(pref_w))
    }

    fn update(
        &mut self,
        event: super::WidgetUpdateEvent,
        sys_interface: &mut T,
    ) -> Result<bool, String> {
        self.changed.set(false);
        self.draw_pos = event.position;

        let non_zero_area: TextureRect = match self.draw_pos.into() {
            Some(v) => v,
            None => return Ok(false), // can't click or hover with zero area
        };
        for e in event.events.iter_mut().filter(|e| e.available()) {
            match e.e {
                crate::core::event::Event::Mouse(mouse) => {
                    if non_zero_area.contains_point((mouse.x, mouse.y))
                        && event.clipping_rect.contains_point((mouse.x, mouse.y))
                    {
                        self.hovered = true;
                        if mouse.down && mouse.changed {
                            // on rising edge
                            e.set_consumed();
                            self.checked.set(!self.checked.get());
                            self.changed.set(true);
                            if let Some(toggle_sound) = &self.toggle_sound {
                                sys_interface.sound(toggle_sound, 0., 0.)?;
                            }
                        }
                    } else {
                        self.hovered = false;
                    }
                }
                _ => {}
            }
        }
        Ok(false)
    }

    fn draw(&self, sys_interface: &mut T) -> Result<(), String> {
        let pos: Option<crate::core::texture_area::TextureRect> = self.draw_pos.into();
        let pos = match pos {
            Some(v) => v,
            None => return Ok(()),
        };

        let mut txt = sys_interface.texture(&self.texture_path)?;

        let src = if self.checked.get() {
            if self.hovered {
                self.check_faded
            } else {
                self.check
            }
        } else {
            if self.hovered {
                self.uncheck_faded
            } else {
                self.uncheck
            }
        };

        txt.copy(src, pos)?;

        Ok(())
    }
}
