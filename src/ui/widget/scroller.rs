use std::{cell::Cell, rc::Rc};

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
        widget::{sizing::NestedContentSizing, FrameTransiency, Widget, WidgetUpdateEvent},
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

pub static SCROLLER_DRAG_DEAD_ZONE_DEFAULT: u32 = 10;
pub static SCROLLER_SCROLL_WHEEL_SENSITIVITY_DEFAULT: i32 = 20;

// nested scrollers are not allowed:
//
// 1. the parent scroller must be updated before the children since the parent
//    determines where the children will be placed - scrolling adjusts the
//    position of the children
// 2. the children must be updated before the parent since the children should
//    consume events before the parent - the inner scroller should consume
//    events (making it scroll instead of the parent) and to do that, it needs
//    the events first
//
// A couple ways of solving this.
// 1. satisfy requirement #1. in order:
//     1. update the scroller, but in a way that ignores mouse rect regions
//        which are reserved by the children
//     2. update the children
//    this solution complicates the overall widget interface, needs the child to
//    somehow tell the parent that certain inputs should be ignored. and when
//    doing this, the parent would need to do its layout logic to determine
//    where the child is in the first place (which seems wasteful)
// 2. satisfy requirement #2. in order:
//      1. update the children
//      2. update the scroller
//      3. adjust the position of the children based on the new scroll values
//    this solution complicates the overall widget interface (needs to expose
//    adjusting the position). this is what I did in the first pass of the UI
//    framework to allow nested scrollers:
//    https://github.com/jagprog5/sdl-rust-ui/blob/7530baa7ae7b57f4514899cb2315274e390bc1a6/src/layout/scroller.rs#L615
//
// neither of these solutions are good / overall worth it

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
///
pub struct Scroller<'b, T: crate::core::System> {
    /// manhattan distance that the mouse must travel before it's considered a
    /// click and drag scroll
    pub drag_deadzone: u32,
    pub scroll_wheel_sensitivity: i32,

    /// state which should persist between frames
    pub drag_state: Rc<Cell<DragState>>,
    /// state which should persist between frames. None if disable
    pub scroll_x: Option<Rc<Cell<i32>>>,
    /// state which should persist between frames. None if disable
    pub scroll_y: Option<Rc<Cell<i32>>>,

    pub contained: Box<dyn Widget<T> + 'b>,

    pub sizing: NestedContentSizing,

    pub lock_small_content_x: Option<MaxLenFailPolicy>,
    pub lock_small_content_y: Option<MaxLenFailPolicy>,
    /// an output indicating the scroll amount and the max scroll, respectively
    pub scroll_x_portion: Option<Rc<Cell<(i32, i32)>>>,
    /// an output indicating the scroll amount and the max scroll, respectively
    pub scroll_y_portion: Option<Rc<Cell<(i32, i32)>>>,

    /// calculated during update, stored for draw.
    /// used for clipping rect calculations
    clipping_rect_for_contained_from_update: ClippingRect,
    position_for_contained_from_update: FRect,
}

impl<'b, T: crate::core::System> Scroller<'b, T> {
    /// scroll_x, scroll_y, and drag_state are states which should be persist
    /// between frames
    pub fn new(
        contains: Box<dyn Widget<T> + 'b>,
        scroll_x: Option<Rc<Cell<i32>>>,
        scroll_y: Option<Rc<Cell<i32>>>,
        drag_state: Rc<Cell<DragState>>,
    ) -> Self {
        Self {
            drag_state,
            drag_deadzone: SCROLLER_DRAG_DEAD_ZONE_DEFAULT,
            scroll_wheel_sensitivity: SCROLLER_SCROLL_WHEEL_SENSITIVITY_DEFAULT,
            scroll_x,
            scroll_y,
            scroll_x_portion: None,
            scroll_y_portion: None,
            contained: contains,
            lock_small_content_x: Some(MaxLenFailPolicy::CENTERED),
            lock_small_content_y: Some(MaxLenFailPolicy::NEGATIVE),
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
    lock_small_content_x: Option<MaxLenFailPolicy>,
    lock_small_content_y: Option<MaxLenFailPolicy>,
    scroll_x_portion: Option<&Rc<Cell<(i32, i32)>>>,
    scroll_y_portion: Option<&Rc<Cell<(i32, i32)>>>,
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
            scroll_y_portion.map(|c| c.set((0, 0)));
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
            let top_offset = position_for_contained.y - event_position.y;
            let available_space = event_position_h - position_for_contained_h;
            scroll_y_portion
                .map(|c| c.set((top_offset.clamp(0, available_space), available_space)));
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
        let visible_top = event_position.y - position_for_contained.y;
        let hidden_range = position_for_contained_h - event_position_h;
        scroll_y_portion.map(|c| c.set((visible_top.clamp(0, hidden_range), hidden_range)));
    }

    if position_for_contained_w < event_position_w {
        // the contained thing is smaller than the parent
        if let Some(lock_small_content_x) = lock_small_content_x {
            *scroll_x = ((event_position_w - position_for_contained_w) as f32
                * lock_small_content_x.0)
                .round() as i32;
            scroll_x_portion.map(|c| c.set((0, 0)));
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
            let left_offset = position_for_contained.x - event_position.x;
            let available_space = event_position_w - position_for_contained_w;
            scroll_x_portion
                .map(|c| c.set((left_offset.clamp(0, available_space), available_space)));
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
        let visible_left = event_position.x - position_for_contained.x;
        let hidden_range = position_for_contained_w - event_position_w;
        scroll_x_portion.map(|c| c.set((visible_left.clamp(0, hidden_range), hidden_range)));
    }
}

impl<'b, T: crate::core::System> Widget<T> for Scroller<'b, T> {
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
            .preferred_ratio_exceed_parent(self.contained.as_ref())
    }

    fn update(
        &mut self,
        mut event: WidgetUpdateEvent,
        sys_interface: &mut T,
    ) -> Result<FrameTransiency, String> {
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

        self.position_for_contained_from_update = self.sizing.position_for_contained(
            self.contained.as_ref(),
            event.position,
            sys_interface,
        )?;

        let position_for_contained: Option<TextureRect> =
            self.position_for_contained_from_update.into();

        // in some niche cases, the scroller does not need to consume events
        // because it cannot scroll
        let mut scroll_y_is_effective = true;
        let mut scroll_x_is_effective = true;

        if self.scroll_y.is_none() {
            scroll_y_is_effective = false;
        }

        if self.scroll_x.is_none() {
            scroll_x_is_effective = false;
        }

        if position_for_contained.map(|p| p.h.get()).unwrap_or(0) < pos.h.get()
            && self.lock_small_content_y.is_some()
        {
            scroll_y_is_effective = false;
        }

        if position_for_contained.map(|p| p.w.get()).unwrap_or(0) < pos.w.get()
            && self.lock_small_content_x.is_some()
        {
            scroll_x_is_effective = false;
        }

        if scroll_y_is_effective || scroll_x_is_effective {
            // handle click and drag scroll
            for e in event.events.iter_mut().filter(|e| e.is_some()) {
                match e.unwrap() {
                    crate::core::event::Event::MouseWheel(m) => {
                        if pos.contains_point((m.x, m.y))
                            && event.clipping_rect.contains_point((m.x, m.y))
                        {
                            *e = None;
                            if let DragState::Dragging(_) = self.drag_state.get() {
                                self.drag_state.set(DragState::DragStart((m.x, m.y)));
                            }

                            self.scroll_x.as_ref().map(|scroll_x| {
                                scroll_x.set(
                                    scroll_x.get() - m.wheel_dx * self.scroll_wheel_sensitivity,
                                )
                            });
                            self.scroll_y.as_ref().map(|scroll_y| {
                                scroll_y.set(
                                    scroll_y.get() + m.wheel_dy * self.scroll_wheel_sensitivity,
                                )
                            });
                        }
                    }
                    crate::core::event::Event::Mouse(m) => {
                        if !m.down {
                            // edge case on below - if currently dragging then
                            // events are consumed. but on the falling edge this
                            // should still happen (when about to not be dragging)
                            if let DragState::Dragging(_) = self.drag_state.get() {
                                *e = None;
                            }
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
                            let trigger_x = dragged_far_enough_x && self.scroll_x.is_some();
                            let trigger_y = dragged_far_enough_y && self.scroll_y.is_some();
                            if trigger_x || trigger_y {
                                self.drag_state.set(DragState::Dragging((
                                    m.x - self.scroll_x.as_ref().map(|c| c.get()).unwrap_or(0),
                                    m.y - self.scroll_y.as_ref().map(|c| c.get()).unwrap_or(0),
                                )));
                                // intentional fallthrough
                            }
                        }

                        if let DragState::Dragging((drag_x, drag_y)) = self.drag_state.get() {
                            self.scroll_x
                                .as_ref()
                                .map(|scroll_x| scroll_x.set(m.x - drag_x));
                            self.scroll_y
                                .as_ref()
                                .map(|scroll_y| scroll_y.set(m.y - drag_y));
                        }

                        // LAST: if currently dragging then consume all mouse events
                        match self.drag_state.get() {
                            DragState::Dragging(_) => {
                                *e = None;
                            }
                            _ => {}
                        }
                    }
                    _ => {}
                }
            }
        }

        let position_for_contained = match position_for_contained {
            Some(v) => v,
            None => {
                self.clipping_rect_for_contained_from_update = ClippingRect::Zero;
                return self.contained.update(event, sys_interface); // same as above
            }
        };

        let mut scroll_x_arg = self.scroll_x.as_ref().map(|c| c.get()).unwrap_or(0);
        let mut scroll_y_arg = self.scroll_y.as_ref().map(|c| c.get()).unwrap_or(0);
        apply_scroll_restrictions(
            position_for_contained,
            pos,
            &mut scroll_x_arg,
            &mut scroll_y_arg,
            self.lock_small_content_x,
            self.lock_small_content_y,
            self.scroll_x_portion.as_ref(),
            self.scroll_y_portion.as_ref(),
        );
        self.scroll_x.as_ref().map(|c| c.set(scroll_x_arg));
        self.scroll_y.as_ref().map(|c| c.set(scroll_y_arg));

        self.clipping_rect_for_contained_from_update =
            event.clipping_rect.intersect_area(Some(pos));
        self.position_for_contained_from_update.x +=
            self.scroll_x.as_ref().map(|c| c.get()).unwrap_or(0) as f32;
        self.position_for_contained_from_update.y +=
            self.scroll_y.as_ref().map(|c| c.get()).unwrap_or(0) as f32;

        let mut event_for_contained = event.sub_event(self.position_for_contained_from_update);
        event_for_contained.clipping_rect = self.clipping_rect_for_contained_from_update;
        let ret = self.contained.update(event_for_contained, sys_interface)?;

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
