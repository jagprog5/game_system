use std::cell::Cell;

use crate::{
    core::{clipping_rect::ClippingRect, texture_rect::TextureRect},
    ui::{
        util::{
            length::{
                AspectRatioPreferredDirection, MaxLen, MaxLenFailPolicy, MinLen, MinLenFailPolicy,
                PreferredPortion,
            },
            rect::FRect,
        },
        widget::{sizing::NestedContentSizing, Widget, WidgetUpdateEvent},
    },
};

#[derive(Debug, Default, Clone, Copy)]
pub enum DragState {
    #[default]
    None,
    /// waiting for mouse to move far enough before beginning dragging
    DragStart((i32, i32)),
    /// contains drag diff
    Dragging((i32, i32)),
}

#[derive(Default)]
pub enum ScrollAspectRatioDirectionPolicy {
    #[default]
    Inherit,
    Literal(AspectRatioPreferredDirection),
}

/// translates its content - facilitates scrolling. also applies clipping rect
/// to contained content
///
/// does NOT do any form of culling for widgets which are not visible in the
/// current viewing area - all contained widgets are updated and drawn. it is
/// the responsibility of the contained widgets themselves to cull if they
/// choose to
///
/// it is the responsibility of the contained widget to filter out mouse events
/// which are not within the sdl clipping rectangle (which is set for both draw,
/// as well as update, for convenience)
pub struct Scroller<'font_data, 'b, 'scroll_state, T: crate::core::System<'font_data>> {
    /// manhattan distance that the mouse must travel before it's considered a
    /// click and drag scroll
    pub drag_deadzone: u32,
    pub scroll_x_enabled: bool,
    pub scroll_y_enabled: bool,
    /// for drag scrolling
    pub drag_state: &'scroll_state Cell<DragState>,
    pub scroll_x: &'scroll_state Cell<i32>,
    pub scroll_y: &'scroll_state Cell<i32>,
    pub contained: Box<dyn Widget<'font_data, T> + 'b>,

    pub sizing: NestedContentSizing,

    /// true restricts the scrolling to keep the contained in frame
    pub restrict_scroll: bool,

    pub lock_small_content_y: Option<MaxLenFailPolicy>,
    pub lock_small_content_x: Option<MaxLenFailPolicy>,

    /// calculated during update, stored for draw.
    /// used for clipping rect calculations
    clipping_rect_for_contained_from_update: ClippingRect,
    position_for_contained_from_update: FRect,
}

impl<'font_data, 'b, 'scroll_state, T: crate::core::System<'font_data>>
    Scroller<'font_data, 'b, 'scroll_state, T>
{
    /// scroll_x, scroll_y, and drag_state are states which should be retained
    /// between frames
    pub fn new(
        scroll_x_enabled: bool,
        scroll_y_enabled: bool,
        drag_state: &'scroll_state Cell<DragState>,
        scroll_x: &'scroll_state Cell<i32>,
        scroll_y: &'scroll_state Cell<i32>,
        contains: Box<dyn Widget<'font_data, T> + 'b>,
    ) -> Self {
        Self {
            drag_state,
            drag_deadzone: 10,
            scroll_x_enabled,
            scroll_y_enabled,
            scroll_x,
            scroll_y,
            contained: contains,
            restrict_scroll: true,
            lock_small_content_y: None,
            lock_small_content_x: None,
            sizing: NestedContentSizing::Inherit,
            clipping_rect_for_contained_from_update: ClippingRect::None,
            position_for_contained_from_update: Default::default(),
        }
    }
}

/// apply even if scroll is not enabled (as what if it was enabled previously
/// and content was moved off screen)
fn apply_scroll_restrictions(
    mut position_for_contained: TextureRect,
    event_position: TextureRect,
    scroll_x: &mut i32,
    scroll_y: &mut i32,
    lock_small_content_y: Option<MaxLenFailPolicy>,
    lock_small_content_x: Option<MaxLenFailPolicy>,
) {
    position_for_contained.x += *scroll_x;
    position_for_contained.y += *scroll_y;

    let position_for_contained_h = position_for_contained.h.get() as i32;
    let position_for_contained_w = position_for_contained.w.get() as i32;

    let event_position_h = event_position.h.get() as i32;
    let event_position_w = event_position.w.get() as i32;

    if position_for_contained_h < event_position_h {
        // the contained thing is smaller than the parent
        if let Some(lock_small_content_y) = lock_small_content_y {
            *scroll_y = ((event_position_h - position_for_contained_h) as f32
                * lock_small_content_y.0)
                .round() as i32;
        } else {
            let violating_top = position_for_contained.y < event_position.y;
            let violating_bottom = position_for_contained.y + position_for_contained_h
                > event_position.y + event_position_h;

            if violating_top {
                *scroll_y += (event_position.y - position_for_contained.y) as i32;
            } else if violating_bottom {
                *scroll_y -= ((position_for_contained.y + position_for_contained_h)
                    - (event_position.y + event_position_h)) as i32;
            }
        }
    } else {
        let down_from_top = position_for_contained.y > event_position.y;

        let up_from_bottom = position_for_contained.y + position_for_contained_h
            < event_position.y + event_position_h;

        if down_from_top {
            *scroll_y += (event_position.y - position_for_contained.y) as i32;
        } else if up_from_bottom {
            *scroll_y -= ((position_for_contained.y + position_for_contained_h)
                - (event_position.y + event_position_h)) as i32;
        }
    }

    if position_for_contained_w < event_position_w {
        // the contained thing is smaller than the parent
        if let Some(lock_small_content_x) = lock_small_content_x {
            *scroll_x = ((event_position_w - position_for_contained_w) as f32
                * lock_small_content_x.0)
                .round() as i32;
        } else {
            let violating_left = position_for_contained.x < event_position.x;
            let violating_right = position_for_contained.x + position_for_contained_w
                > event_position.x + event_position_w;

            if violating_left {
                *scroll_x += (event_position.x - position_for_contained.x) as i32;
            } else if violating_right {
                *scroll_x -= ((position_for_contained.x + position_for_contained_w)
                    - (event_position.x + event_position_w)) as i32;
            }
        }
    } else {
        let left_from_right = position_for_contained.x > event_position.x;

        let right_from_left = position_for_contained.x + position_for_contained_w
            < event_position.x + event_position_w;

        if left_from_right {
            *scroll_x += (event_position.x - position_for_contained.x) as i32;
        } else if right_from_left {
            *scroll_x -= ((position_for_contained.x + position_for_contained_w)
                - (event_position.x + event_position_w)) as i32;
        }
    }
}

impl<'font_data, 'b, 'scroll_state, T: crate::core::System<'font_data>> Widget<'font_data, T>
    for Scroller<'font_data, 'b, 'scroll_state, T>
{
    fn min(&self, sys_interface: &mut T) -> Result<(MinLen, MinLen), String> {
        self.sizing.min(self.contained.as_ref(), sys_interface)
    }

    fn min_w_fail_policy(&self) -> MinLenFailPolicy {
        self.sizing.min_w_fail_policy(self.contained.as_ref())
    }

    fn min_h_fail_policy(&self) -> MinLenFailPolicy {
        self.sizing.min_h_fail_policy(self.contained.as_ref())
    }

    fn max(&self, sys_interface: &mut T) -> Result<(MaxLen, MaxLen), String> {
        self.sizing.max(self.contained.as_ref(), sys_interface)
    }

    fn max_w_fail_policy(&self) -> MaxLenFailPolicy {
        self.sizing.max_w_fail_policy(self.contained.as_ref())
    }

    fn max_h_fail_policy(&self) -> MaxLenFailPolicy {
        self.sizing.max_h_fail_policy(self.contained.as_ref())
    }

    fn preferred_portion(&self) -> (PreferredPortion, PreferredPortion) {
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
            .preferred_link_allowed_exceed_portion(self.contained.as_ref())
    }

    fn update(
        &mut self,
        mut event: WidgetUpdateEvent,
        sys_interface: &mut T,
    ) -> Result<bool, String> {
        let pos: Option<TextureRect> = event.position.into();
        let pos = match pos {
            Some(v) => v,
            None => {
                // this scroll area occupies zero area. this can't be
                // meaningfully updated.

                // just in case, the events are still passed to the contained
                // widget anyway. what if it needs to react to a key press or
                // something despite having no draw area?

                self.clipping_rect_for_contained_from_update = ClippingRect::Zero;
                return self.contained.update(event, sys_interface);
            }
        };

        // the scroller might consume events. for example, if it is clicked and
        // dragged as part of scrolling. setting the events to consumed happens
        // after they are passed to the contained widget (otherwise the inside
        // content would not be click-able)

        let mut defer_consumed: Vec<bool> = Vec::new();
        defer_consumed.resize(event.events.len(), false);

        // handle click and drag scroll
        for (index, e) in event
            .events
            .iter_mut()
            .enumerate()
            .filter(|(_index, e)| e.available())
        {
            match e.e {
                crate::core::event::Event::MouseWheel(m) => {
                    if pos.contains_point((m.x, m.y))
                        && event.clipping_rect.contains_point((m.x, m.y))
                    {
                        self.scroll_x.set(self.scroll_x.get() - m.wheel_dx * 7);
                        self.scroll_y.set(self.scroll_y.get() + m.wheel_dy * 7);
                    }
                }
                crate::core::event::Event::Mouse(m) => {
                    if !m.down {
                        self.drag_state.set(DragState::None);
                        continue;
                    }

                    if let DragState::None = self.drag_state.get() {
                        if m.changed
                            && pos.contains_point((m.x, m.y))
                            && event.clipping_rect.contains_point((m.x, m.y))
                        {
                            self.drag_state.set(DragState::DragStart((m.x, m.y)));
                            // fall through
                        }
                    }

                    if let DragState::DragStart((start_x, start_y)) = self.drag_state.get() {
                        let dragged_far_enough_x =
                            (start_x - m.x).unsigned_abs() > self.drag_deadzone;
                        let dragged_far_enough_y =
                            (start_y - m.y).unsigned_abs() > self.drag_deadzone;
                        let trigger_x = dragged_far_enough_x && self.scroll_x_enabled;
                        let trigger_y = dragged_far_enough_y && self.scroll_y_enabled;
                        if trigger_x || trigger_y {
                            self.drag_state.set(DragState::Dragging((
                                m.x - self.scroll_x.get(),
                                m.y - self.scroll_y.get(),
                            )));
                            // intentional fallthrough
                        }
                    }

                    if let DragState::Dragging((drag_x, drag_y)) = self.drag_state.get() {
                        if self.scroll_x_enabled {
                            self.scroll_x.set(m.x - drag_x);
                        }
                        if self.scroll_y_enabled {
                            self.scroll_y.set(m.y - drag_y);
                        }
                    }

                    // LAST: if currently dragging then consume all mouse events
                    match self.drag_state.get() {
                        DragState::Dragging(_) | DragState::DragStart(_) => {
                            defer_consumed[index] = true;
                        }
                        _ => {}
                    }
                }
                _ => {}
            }
        }

        self.position_for_contained_from_update =
            self.sizing
                .position_for_contained(self.contained.as_ref(), &event, sys_interface)?;

        let position_for_contained: Option<TextureRect> =
            self.position_for_contained_from_update.into();
        let position_for_contained = match position_for_contained {
            Some(v) => v,
            None => {
                return self.contained.update(event, sys_interface); // same as above
            }
        };

        if self.restrict_scroll {
            let mut scroll_x_arg = self.scroll_x.get();
            let mut scroll_y_arg = self.scroll_y.get();
            apply_scroll_restrictions(
                position_for_contained,
                pos,
                &mut scroll_x_arg,
                &mut scroll_y_arg,
                self.lock_small_content_y,
                self.lock_small_content_x,
            );
            self.scroll_x.set(scroll_x_arg);
            self.scroll_y.set(scroll_y_arg);
        }

        self.clipping_rect_for_contained_from_update =
            event.clipping_rect.intersect_area(Some(pos));
        self.position_for_contained_from_update.x += self.scroll_x.get() as f32;
        self.position_for_contained_from_update.y += self.scroll_y.get() as f32;

        let mut event_for_contained = event.sub_event(self.position_for_contained_from_update);
        event_for_contained.clipping_rect = self.clipping_rect_for_contained_from_update;
        let ret = self.contained.update(event_for_contained, sys_interface)?;

        for i in 0..defer_consumed.len() {
            if defer_consumed[i] {
                event.events[i].set_consumed();
            }
        }

        Ok(ret)
    }

    fn draw(&self, sys_interface: &mut T) -> Result<(), String> {
        let previous_clipping_rect = sys_interface.get_clip();
        sys_interface.clip(self.clipping_rect_for_contained_from_update);
        let draw_result = self.contained.draw(sys_interface);
        sys_interface.clip(previous_clipping_rect); // restore
        draw_result
    }
}
