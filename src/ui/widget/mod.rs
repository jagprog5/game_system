pub mod debug;
pub mod strut;
pub mod texture;

pub mod background;
pub mod border;

pub mod multi_line_label;
pub mod single_line_label;

pub mod checkbox;

pub mod button;
pub mod sizing;

use std::{num::NonZeroU32, time::Duration};

use crate::{
    core::clipping_rect::ClippingArea, ui::util::{
        length::{
            clamp, AspectRatioPreferredDirection, MaxLen, MaxLenFailPolicy, MinLen,
            MinLenFailPolicy, PreferredPortion,
        },
        rect::FRect,
    }
};

use super::util::rust::reborrow;

/// two purposes:
///  - used to indicate which events were not used by the UI and should be
///    passed down to the rest of the application
///  - used to ensure that a single widget uses an event
#[derive(Debug, Clone, Copy)]
pub enum ConsumedStatus {
    /// this event has not been consumed by any widget
    None,

    /// this event has been consumed by a non-layout widget. for the most part,
    /// it should be considered consumed, but it might still be used by layouts
    /// (e.g. scroller). this distinction was required for nested scroll widgets
    /// to work (a scroller's contained widget is given the opportunity to
    /// consume events first. that way an inner scroller can consume some scroll
    /// amount before the outer scroller. but if the child is instead something
    /// else which would consume events and prevent a scroll, then it is
    /// ignored)
    ConsumedByWidget,

    /// this event has been consumed by a layout, and should not be used by
    /// anything else
    ConsumedByLayout,
}

#[derive(Debug)]
pub struct UIEvent {
    pub e: crate::core::event::Event,
    consumed_status: ConsumedStatus,
}

impl UIEvent {
    pub fn consumed(&self) -> bool {
        match self.consumed_status {
            ConsumedStatus::None => false,
            _ => true,
        }
    }

    pub fn available(&self) -> bool {
        !self.consumed()
    }

    pub fn consumed_status(&self) -> ConsumedStatus {
        self.consumed_status
    }

    pub fn set_consumed(&mut self) {
        self.consumed_status = ConsumedStatus::ConsumedByWidget;
    }

    pub fn set_consumed_by_layout(&mut self) {
        debug_assert!(match self.consumed_status {
            ConsumedStatus::ConsumedByLayout => false,
            _ => true,
        });
        self.consumed_status = ConsumedStatus::ConsumedByLayout;
    }

    pub fn new(e: crate::core::event::Event) -> Self {
        Self {
            e,
            consumed_status: ConsumedStatus::None,
        }
    }
}

pub struct WidgetUpdateEvent<'sdl> {
    /// the position that this widget is at. this is NOT an sdl2::rect::FRect
    // it's important to keep the sizing as floats as the sizing is being
    // computed.
    // - otherwise there's a lot of casting to and from integer. best to keep it
    //   as floating point until just before use
    // - started running into issues where a one pixel difference leads to a
    //   visible jump. specifically, when a label font size changes in
    //   horizontal layout (a one pixel in height leading to a larger difference
    //   in width due to aspect ratio)
    // - sdl2 has an f32 API
    pub position: FRect,
    /// although the object is updated at a position, give also the clipping rect
    /// that will be in effect once the widget is drawn
    pub clipping_rect: ClippingArea,
    /// in the context of where this widget is in the GUI, does the width or the
    /// height have priority in regard to enforcing an aspect ratio. one length
    /// is figured out first, the the other is calculated based on the first
    pub aspect_ratio_priority: AspectRatioPreferredDirection,
    /// handle all events from sdl. contains events in order of occurrence
    pub events: &'sdl mut [UIEvent],
    /// time since previous event, or 0 if first event
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
            aspect_ratio_priority: self.aspect_ratio_priority,
            events: reborrow(self.events),
            dt: self.dt
        }
    }

    pub fn dup(&mut self) -> WidgetUpdateEvent<'_> {
        self.sub_event(self.position)
    }
}

pub trait Widget<'font_data, T: crate::core::System<'font_data>> {
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
    fn preferred_width_from_height(&self, _pref_h: f32, _sys_interface: &mut T) -> Option<Result<f32, String>> {
        None
    }

    /// implementors should use this to request an aspect ratio (additionally,
    /// the min and max should have the same ratio)
    fn preferred_height_from_width(&self, _pref_w: f32, _sys_interface: &mut T) -> Option<Result<f32, String>> {
        None
    }

    /// generally this shouldn't be changed from the default implementation.
    ///
    /// this effects the behavior of preferred_width_from_height and
    /// preferred_height_from_width.
    ///
    /// if true is returned, the output from those function is not restricted to
    /// be within the preferred portion of the parent (unless this would
    /// conflict with the min len)
    fn preferred_link_allowed_exceed_portion(&self) -> bool {
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
    /// 
    /// a return value of true indicates 
    fn update(&mut self, _event: WidgetUpdateEvent, _sys_interface: &mut T) -> Result<bool, String> {
        Ok(false)
    }

    /// draw. called after all widgets are update each frame
    fn draw(&self, sys_interface: &mut T) -> Result<(), String>;
}

/// each frame after update_gui, the widget should be drawn with widget.draw()
/// 
/// dt is the duration since the previous frame, or 0 if it's the first frame
pub fn update_gui<'font_data, 'b, T: crate::core::System<'font_data> + 'b>(
    widget: &'b mut dyn Widget<'font_data, T>,
    events: &'b mut [UIEvent],
    system: &mut T,
    dt: Duration,
) -> Result<bool, String> {
    let (w, h) = match system.size() {
        Ok(v) => v,
        Err(msg) => {
            debug_assert!(false, "{}", msg); // infallible in prod
            unsafe {
                (
                    NonZeroU32::new_unchecked(320),
                    NonZeroU32::new_unchecked(320),
                )
            }
        }
    };

    let aspect_ratio_priority = AspectRatioPreferredDirection::default();

    let position = place(
        widget,
        FRect {
            x: 0.,
            y: 0.,
            w: w.get() as f32,
            h: h.get() as f32,
        },
        aspect_ratio_priority,
        system,
    )?;

    let widget_event = WidgetUpdateEvent {
        position,
        events,
        aspect_ratio_priority: AspectRatioPreferredDirection::default(),
        clipping_rect: ClippingArea::None,
        dt
    };
    widget.update(widget_event, system)
}

/// given a widget's min, max lengths and fail policies, what's the widget's
/// lengths and offset within the parent.
pub fn place<'a, T: crate::core::System<'a>>(
    widget: &dyn Widget<'a, T>,
    parent: FRect,
    ratio_priority: AspectRatioPreferredDirection,
    system: &mut T,
) -> Result<FRect, String> {
    let (max_w, max_h) = widget.max(system)?;
    let (min_w, min_h) = widget.min(system)?;
    let (preferred_portion_w, preferred_portion_h) = widget.preferred_portion();
    let pre_clamp_w = preferred_portion_w.get(parent.w);
    let pre_clamp_h = preferred_portion_h.get(parent.h);
    let mut w = clamp(pre_clamp_w, min_w, max_w);
    let mut h = clamp(pre_clamp_h, min_h, max_h);

    match ratio_priority {
        AspectRatioPreferredDirection::WidthFromHeight => {
            if let Some(new_w) = widget.preferred_width_from_height(h, system) {
                let new_w = new_w?;
                let new_w_max_clamp = if widget.preferred_link_allowed_exceed_portion() {
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
                let new_h_max_clamp = if widget.preferred_link_allowed_exceed_portion() {
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

