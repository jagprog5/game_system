pub mod image_display;
pub mod strut;

pub mod border;
pub mod tiled_image_display;

pub mod multi_line_label;
pub mod single_line_label;

pub mod checkbox;

pub mod background;
pub mod button;
pub mod clipper;
pub mod scroller;
pub mod sizing;

pub mod horizontal_layout;
pub mod vertical_layout;

use std::time::{Duration, Instant};

use crate::{
    core::{clipping_rect::ClippingRect, System},
    ui::util::{
        length::{
            clamp, AspectRatioPreferredDirection, MaxLen, MaxLenFailPolicy, MinLen,
            MinLenFailPolicy, PreferredPortion,
        },
        rect::FRect,
    },
};

use super::util::rust::reborrow;

pub struct WidgetUpdateEvent<'sdl> {
    /// given the sizing information that was obtained from the widget (min,
    /// max, etc), a position for this widget has been determined. this is where
    /// the widget is at!
    pub position: FRect,
    /// the clipping rect that will be used during draw
    ///
    /// WidgetUpdateEvent is used during the update phase for the UI (which
    /// occurs before draw). however some widgets also need to know what the
    /// clipping rectangle will be during the update phase (for example, a
    /// button which is scrolled outside of a scroller bounds will no longer be
    /// inside the visible area and should not react to user input).
    pub clipping_rect: ClippingRect,
    /// handle all events from backend
    ///
    /// set to None to consume an event, meaning that other widgets will not be
    /// able to use it (this events ref is shown to all widgets in the
    /// interface). secondary purpose: events which are not used by the UI are
    /// passed down to the rest of the application.
    pub events: &'sdl mut Option<crate::core::event::Event>,
    /// time since previous event, maybe zero if first event
    pub dt: Duration,
}

impl<'sdl> WidgetUpdateEvent<'sdl> {
    /// create a new event, same as self, but with a different position.
    /// intended to be passed to a layout's children
    pub fn sub_event(&mut self, position: FRect) -> WidgetUpdateEvent<'_> {
        WidgetUpdateEvent {
            // do a re-borrow. create a mutable borrow of the mutable borrow
            // output lifetime is elided - it's the re-borrowed lifetime
            position,
            clipping_rect: self.clipping_rect,
            events: reborrow(self.events),
            dt: self.dt,
        }
    }

    pub fn dup(&mut self) -> WidgetUpdateEvent<'_> {
        self.sub_event(self.position)
    }
}

/// widgets form a hierarchy, and are updated and drawn in a top down way
pub trait Widget<T: crate::core::System> {
    /// the widget will never have a width or height smaller than this width or
    /// height, respectively.
    fn min(&self, _sys_interface: &mut T) -> Result<(MinLen, MinLen), String> {
        Ok((MinLen::LAX, MinLen::LAX))
    }

    fn min_w_fail_policy(&self) -> MinLenFailPolicy {
        MinLenFailPolicy::CENTERED
    }
    fn min_h_fail_policy(&self) -> MinLenFailPolicy {
        MinLenFailPolicy::CENTERED
    }

    /// the widget will never have a width or height greater than this width or
    /// height, respectively, unless it would conflict with the minimum width or
    /// height, respectively.
    fn max(&self, _sys_interface: &mut T) -> Result<(MaxLen, MaxLen), String> {
        Ok((MaxLen::LAX, MaxLen::LAX))
    }

    fn max_w_fail_policy(&self) -> MaxLenFailPolicy {
        MaxLenFailPolicy::CENTERED
    }
    fn max_h_fail_policy(&self) -> MaxLenFailPolicy {
        MaxLenFailPolicy::CENTERED
    }

    /// portion of parent. sometimes used as a weight between competing components
    fn preferred_portion(&self) -> (PreferredPortion, PreferredPortion) {
        (PreferredPortion::FULL, PreferredPortion::FULL)
    }

    /// implementors should use this to request an aspect ratio (additionally,
    /// the min and max should have the same ratio)
    fn preferred_width_from_height(
        &self,
        _pref_h: f32,
        _sys_interface: &mut T,
    ) -> Option<Result<f32, String>> {
        None
    }

    /// implementors should use this to request an aspect ratio (additionally,
    /// the min and max should have the same ratio)
    fn preferred_height_from_width(
        &self,
        _pref_w: f32,
        _sys_interface: &mut T,
    ) -> Option<Result<f32, String>> {
        None
    }

    /// when enforcing a preferred aspect ratio, is the widget allows to exceed
    /// the parent's boundaries?
    ///
    /// generally this shouldn't be changed from the default implementation.
    fn preferred_ratio_exceed_parent(&self) -> bool {
        false
    }

    /// called for all widgets each frame before any call to draw
    ///
    /// if the UI is being lazily updated - meaning that the UI is only updated
    /// and drawn once input events are received or state changes, then the
    /// screen can remain idle for a while. however this is unsuited for
    /// animations or other effects:
    ///  - true indicates that another frame should follow quickly after this
    ///  - false means don't care
    fn update(
        &mut self,
        _event: WidgetUpdateEvent,
        _sys_interface: &mut T,
    ) -> Result<bool, String> {
        Ok(false)
    }

    /// draw. called after all widgets are update each frame
    fn draw(&self, sys_interface: &mut T) -> Result<(), String>;
}

/// each frame after update_gui, the widget should be drawn with widget.draw()
///
/// dt is the duration since the previous frame, or maybe zero if it's the first
/// frame
pub fn update_gui<'b, T: crate::core::System + 'b>(
    widget: &'b mut dyn Widget<T>,
    events: &'b mut Option<crate::core::event::Event>,
    system: &mut T,
    dt: Duration,
) -> Result<bool, String> {
    let (w, h) = system.size()?;

    let position = place(
        widget,
        FRect {
            x: 0.,
            y: 0.,
            w: w.get() as f32,
            h: h.get() as f32,
        },
        AspectRatioPreferredDirection::default(),
        system,
    )?;

    let widget_event = WidgetUpdateEvent {
        position,
        events,
        clipping_rect: ClippingRect::None,
        dt,
    };
    widget.update(widget_event, system)
}

/// given a widget's min, max lengths and fail policies, what's the widget's
/// lengths and offset within the parent.
pub fn place<T: crate::core::System>(
    widget: &dyn Widget<T>,
    parent: FRect,
    ratio_direction: AspectRatioPreferredDirection,
    system: &mut T,
) -> Result<FRect, String> {
    let (max_w, max_h) = widget.max(system)?;
    let (min_w, min_h) = widget.min(system)?;
    let (preferred_portion_w, preferred_portion_h) = widget.preferred_portion();
    let pre_clamp_w = preferred_portion_w.get(parent.w);
    let pre_clamp_h = preferred_portion_h.get(parent.h);
    let mut w = clamp(pre_clamp_w, min_w, max_w);
    let mut h = clamp(pre_clamp_h, min_h, max_h);

    match ratio_direction {
        AspectRatioPreferredDirection::WidthFromHeight => {
            if let Some(new_w) = widget.preferred_width_from_height(h, system) {
                let new_w = new_w?;
                let new_w_max_clamp = if widget.preferred_ratio_exceed_parent() {
                    max_w
                } else {
                    max_w.strictest(MaxLen(pre_clamp_w))
                };
                w = clamp(new_w, min_w, max_w.strictest(new_w_max_clamp));
            }
        }
        AspectRatioPreferredDirection::HeightFromWidth => {
            if let Some(new_h) = widget.preferred_height_from_width(w, system) {
                let new_h = new_h?;
                let new_h_max_clamp = if widget.preferred_ratio_exceed_parent() {
                    max_h
                } else {
                    max_h.strictest(MaxLen(pre_clamp_h))
                };
                h = clamp(new_h, min_h, max_h.strictest(new_h_max_clamp));
            }
        }
    }

    let x_offset = crate::ui::util::length::place(
        w,
        parent.w,
        widget.min_w_fail_policy(),
        widget.max_w_fail_policy(),
    );
    let y_offset = crate::ui::util::length::place(
        h,
        parent.h,
        widget.min_h_fail_policy(),
        widget.max_h_fail_policy(),
    );

    Ok(FRect {
        x: parent.x + x_offset,
        y: parent.y + y_offset,
        w,
        h,
    })
}

pub enum HandlerReturnValue {
    /// the next draw can wait a very long time for user input
    DelayNextFrame,
    /// the next draw should occur reasonably soon after this one
    NextFrame,
    /// stop the gui now. exits before any more updates or draws
    Stop,
}

pub fn gui_loop<T: System, UpdateF, DrawF>(
    delay: Duration,
    system_interface: &mut T,
    mut update_handler: UpdateF,
    mut draw_handler: DrawF,
) -> Result<(), String>
where
    UpdateF: FnMut(
        &mut T,
        Option<crate::core::event::Event>,
        Duration,
    ) -> Result<HandlerReturnValue, String>,
    DrawF: FnMut(&mut T) -> Result<(), String>,
{
    // timestamp of just before the previous call to update. starts as now so
    // that there is a very small (or zero) dt given to the first update call
    //
    // set to just before the previous call to update
    let mut previous_update_call = Instant::now();

    // the next update call should happen at or before deadline. none if it can
    // wait forever for events before calling update
    //
    // starts as Some(now) so that the first frame occurs right away
    let mut deadline: Option<Instant> = Some(previous_update_call);

    loop {
        let frame_begin = Instant::now();

        let mut following_frame_quick = false;
        // frame loop
        loop {
            // loop - update several times per draw
            let event = match deadline {
                Some(deadline) => {
                    let now = Instant::now();
                    if now >= deadline {
                        None
                    } else {
                        let duration = deadline - now;
                        system_interface.event_timeout(duration)
                    }
                }
                None => Some(system_interface.event()),
            };

            let now = Instant::now();
            // let deadline = deadline.get_or_insert(previous_update_call + delay);
            let dt = now - previous_update_call;
            previous_update_call = now;
            following_frame_quick |= match update_handler(system_interface, event, dt)? {
                HandlerReturnValue::Stop => return Ok(()),
                HandlerReturnValue::DelayNextFrame => false,
                HandlerReturnValue::NextFrame => true,
            };

            if event.is_none() {
                break;
            }
        }
        draw_handler(system_interface)?;

        match following_frame_quick {
            true => {
                deadline = Some(frame_begin + delay);
            },
            false => {
                deadline = None;
            },
        }
    }
}
