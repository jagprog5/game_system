use std::{cell::Cell, path::PathBuf};

use crate::{
    core::{texture_rect::TextureRect, PathLike, TextureHandle},
    ui::util::{
        length::{MaxLen, MinLen},
        rect::FRect,
    },
};

use super::Widget;

#[derive(Clone)]
pub struct CheckBox<'state> {
    pub texture_path: PathBuf,
    /// square
    pub min: MinLen,
    /// square
    pub max: MaxLen,

    pub check: TextureRect,
    pub check_faded: TextureRect,
    pub uncheck: TextureRect,
    pub uncheck_faded: TextureRect,

    pub checked: &'state Cell<bool>,
    pub changed: &'state Cell<bool>,

    /// a button which can be used to toggle this checkbox
    pub hotkey: Option<u8>,

    /// state stored for draw from update
    draw_pos: FRect,
    hovered: bool,
}

impl<'state> CheckBox<'state> {
    pub fn new<'a, P: Into<PathLike<'a>>>(
        texture_path: P,
        min: MinLen,
        max: MaxLen,
        checked: &'state Cell<bool>,
        changed: &'state Cell<bool>,
        check: TextureRect,
        check_faded: TextureRect,
        uncheck: TextureRect,
        uncheck_faded: TextureRect,
    ) -> Self {
        let texture_path: PathLike = texture_path.into();
        let texture_path: PathBuf = texture_path.into();
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
            hotkey: None,
            draw_pos: Default::default(),
            hovered: false,
        }
    }
}

impl<'state, T: crate::core::System> Widget<T> for CheckBox<'state> {
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
        _sys_interface: &mut T,
    ) -> Result<bool, String> {
        self.changed.set(false);
        self.draw_pos = event.position;

        let non_zero_area: TextureRect = match self.draw_pos.into() {
            Some(v) => v,
            None => return Ok(false), // can't click or hover with zero area
        };
        for e in event.events.iter_mut().filter(|e| e.available()) {
            match e.e {
                crate::core::event::Event::Key(key_event) => {
                    if let Some(hotkey) = self.hotkey {
                        if key_event.key == hotkey {
                            e.set_consumed();
                            if !key_event.down {
                                // rising edge
                                self.checked.set(!self.checked.get());
                                self.changed.set(true);
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
                        self.hovered = true;
                        if !mouse.down && mouse.changed {
                            // rising edge
                            self.checked.set(!self.checked.get());
                            self.changed.set(true);
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
        let pos: Option<crate::core::texture_rect::TextureRect> = self.draw_pos.into();
        let pos = match pos {
            Some(v) => v,
            None => return Ok(()),
        };

        let mut txt = sys_interface.image(&self.texture_path)?;

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
