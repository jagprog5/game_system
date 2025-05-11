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
    /// handle all events from backend. contains events in order of occurrence
    ///
    /// set to None to consume an event, meaning that other widgets will not be
    /// able to use it. secondary purpose: events which are not used by the UI
    /// are passed down to the rest of the application.
    ///
    /// for simplicity (and performance), the following iteration order is done:
    ///
    /// for each widget:  
    ///     for each event:  
    ///         widget.handle(event)
    ///
    /// but this is an approximation of what should be happening which works a
    /// majority of the time. here is the ideal iteration order (which does not
    /// happen):
    ///
    /// for each event:  
    ///     for each widget:  
    ///         widget.handle(event)
    ///
    /// there is a difference in the iteration order; each event should be fully
    /// processed by everything before moving on to the next event. related:
    /// https://youtu.be/JxI3Eu5DPwE?si=58H4XhTT2m7XgM3W&t=254
    ///
    /// this hasn't meaningfully come up yet but it's something to be mindful of
    pub events: &'sdl mut [Option<crate::core::event::Event>],
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
    events: &'b mut [Option<crate::core::event::Event>],
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
    /// the next frame can wait a very long time for user input
    DelayNextFrame,
    /// the next frame should occur reasonably soon after this one
    NextFrame,
    Stop,
}

#[allow(dead_code)]
pub fn gui_loop<T: System, F>(
    max_delay: Duration,
    system_interface: &mut T,
    mut handler: F,
) -> Result<(), String>
where
    F: FnMut(
        &mut T,
        &mut [Option<crate::core::event::Event>],
        Duration,
    ) -> Result<HandlerReturnValue, String>,
{
    // accumulate the events for this frame
    let mut events_accumulator: Vec<Option<crate::core::event::Event>> = Vec::new();

    // use for dt calculation
    let mut previous_handle_call = Instant::now();
    loop {
        let next_handle_draw_call = Instant::now();
        let handle_result = handler(
            system_interface,
            &mut events_accumulator,
            next_handle_draw_call - previous_handle_call,
        )?;
        previous_handle_call = next_handle_draw_call;

        // handle events accumulation
        events_accumulator.clear();

        match handle_result {
            HandlerReturnValue::Stop => return Ok(()),
            HandlerReturnValue::DelayNextFrame | HandlerReturnValue::NextFrame => {
                let oldest_event = if let HandlerReturnValue::DelayNextFrame = handle_result {
                    // wait up to forever for the first event of this frame to
                    // come in
                    let event = system_interface.event();
                    events_accumulator.push(Some(event));
                    Instant::now()
                } else {
                    previous_handle_call
                };

                // don't send off the event immediately! wait a bit and
                // accumulate several events to be processed together. max bound
                // on waiting so that the first event received isn't too stale

                loop {
                    let max_time = oldest_event + max_delay;
                    let now = Instant::now();
                    if max_time <= now {
                        break; // can't wait any longer
                    }

                    let time_to_wait = max_time - now;
                    if let Some(event) = system_interface.event_timeout(time_to_wait) {
                        events_accumulator.push(Some(event));
                    }
                }
            }
        };
    }
}
