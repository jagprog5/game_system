use std::time::{Duration, Instant};

use game_system::ui::widget::UIEvent;

#[allow(dead_code)]
pub enum HandlerReturnValue {
    DelayNextFrame,
    NextFrame,
    Stop,
}

/// a helper for the examples. but could do done in a variety of ways
#[allow(dead_code)]
pub fn gui_loop<'a, T: game_system::core::System<'a>, F>(
    max_delay: Duration,
    system_interface: &mut T,
    mut handler: F,
) -> Result<(), String> where
    F: FnMut(&mut T, &mut [UIEvent], Duration) -> Result<HandlerReturnValue, String>,
{
    // accumulate the events for this frame
    let mut events_accumulator: Vec<UIEvent> = Vec::new();

    // use for dt calculation
    let mut previous_handle_call = Instant::now();
    loop {
        let next_handle_draw_call = Instant::now();
        let handle_result = handler(system_interface, &mut events_accumulator, next_handle_draw_call - previous_handle_call)?;
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
                    events_accumulator.push(UIEvent::new(event));
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
                        events_accumulator.push(UIEvent::new(event));
                    }
                }
            },
        };
    }
}
