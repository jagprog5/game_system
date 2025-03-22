use crate::{
    core::{event::Event, texture_area::TextureArea, Texture}, ui::util::{
        length::{
            AspectRatioPreferredDirection, MaxLen, MaxLenFailPolicy, MinLen, MinLenFailPolicy,
            PreferredPortion,
        },
        rect::FRect,
    }
};

use super::{Widget, WidgetUpdateEvent};

#[derive(Debug, Clone, Copy, Default)]
pub struct CustomSizing {
    pub min_w: MinLen,
    pub min_h: MinLen,
    pub max_w: MaxLen,
    pub max_h: MaxLen,
    pub preferred_w: PreferredPortion,
    pub preferred_h: PreferredPortion,
    pub aspect_ratio: Option<f32>,
    pub min_w_fail_policy: MinLenFailPolicy,
    pub max_w_fail_policy: MaxLenFailPolicy,
    pub min_h_fail_policy: MinLenFailPolicy,
    pub max_h_fail_policy: MaxLenFailPolicy,
    pub preferred_link_allowed_exceed_portion: bool,
}

/// super simple debug widget. draws a outline at its position. use for testing
/// purposes. brief flash when clicked
#[derive(Debug, Clone, Copy, Default)]
pub struct Debug {
    pub sizing: CustomSizing,

    /// internal state. set during update. used during draw
    clicked_this_frame: bool,
    /// state stored for draw from update
    draw_pos: FRect,
}

impl<'a, T: crate::core::System<'a>> Widget<'a, T> for Debug {
    fn preferred_link_allowed_exceed_portion(&self) -> bool {
        self.sizing.preferred_link_allowed_exceed_portion
    }

    fn min(&mut self) -> Result<(MinLen, MinLen), String> {
        Ok((self.sizing.min_w, self.sizing.min_h))
    }

    fn min_w_fail_policy(&self) -> MinLenFailPolicy {
        self.sizing.min_w_fail_policy
    }

    fn min_h_fail_policy(&self) -> MinLenFailPolicy {
        self.sizing.min_h_fail_policy
    }

    fn max(&mut self) -> Result<(MaxLen, MaxLen), String> {
        Ok((self.sizing.max_w, self.sizing.max_h))
    }

    fn max_w_fail_policy(&self) -> MaxLenFailPolicy {
        self.sizing.max_w_fail_policy
    }

    fn max_h_fail_policy(&self) -> MaxLenFailPolicy {
        self.sizing.max_h_fail_policy
    }

    fn preferred_portion(&self) -> (PreferredPortion, PreferredPortion) {
        (self.sizing.preferred_w, self.sizing.preferred_h)
    }

    fn preferred_width_from_height(&mut self, pref_h: f32) -> Option<Result<f32, String>> {
        let ratio = match &self.sizing.aspect_ratio {
            None => return None,
            Some(v) => v,
        };

        Some(Ok(AspectRatioPreferredDirection::width_from_height(
            *ratio, pref_h,
        )))
    }

    fn preferred_height_from_width(&mut self, pref_w: f32) -> Option<Result<f32, String>> {
        let ratio = match &self.sizing.aspect_ratio {
            None => return None,
            Some(v) => v,
        };

        Some(Ok(AspectRatioPreferredDirection::height_from_width(
            *ratio, pref_w,
        )))
    }

    fn update(&mut self, event: WidgetUpdateEvent) -> Result<(), String> {
        self.clicked_this_frame = false; // reset each frame
        self.draw_pos = event.position;

        let pos: Option<TextureArea> = event.position.into();
        let pos = match pos {
            Some(v) => v,
            None => return Ok(()), // only functionality is being clicked
        };

        for e in event.events.iter_mut().filter(|e| e.available()) {
            if let Event::Mouse(mouse_event) = e.e {
                let point = (mouse_event.x, mouse_event.y);
                if mouse_event.down && mouse_event.changed && pos.contains_point(point) {
                    if event.clipping_rect.contains_point(point) {
                        e.set_consumed();
                        self.clicked_this_frame = true;
                    }
                }
            }
        }

        Ok(())
    }

    fn update_adjust_position(&mut self, pos_delta: (i32, i32)) {
        self.draw_pos.x += pos_delta.0 as f32;
        self.draw_pos.y += pos_delta.1 as f32;
    }

    fn draw(&mut self, sys_interface: &mut T) -> Result<(), String> {
        // as always, snap to integer grid before rendering / using,
        // plus checks that draw area is non-zero
        let pos: Option<crate::core::texture_area::TextureArea> = self.draw_pos.into();
        let pos = match pos {
            Some(v) => v,
            None => return Ok(()),
        };

        if !self.clicked_this_frame {
            let mut texture = sys_interface.missing_texture()?;
            texture.copy(None, pos)?;
        } else {
            println!("debug rect at {:?} was clicked!", pos);
        }
        Ok(())
    }
}
