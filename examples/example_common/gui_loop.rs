use std::time::{Duration, Instant};

use game_system::core::event::Event;
use game_system::ui::widget::UIEvent;

/// a helper for the examples. but could do done in a variety of ways
#[allow(dead_code)]
pub fn gui_loop<'a, T: game_system::core::System<'a>, F>(
    max_delay: Duration,
    system_interface: &mut T,
    mut handler: F,
) -> Result<(), String> where
    F: FnMut(&mut T, &mut [UIEvent]) -> Result<bool, String>, // true iff leave
{
    // accumulate the events for this frame
    let mut events_accumulator: Vec<UIEvent> = Vec::new();

    // do initial draw call
    if handler(system_interface, &mut events_accumulator)? {
        return Ok(())
    }

    'running: loop {
        // wait forever since nothing has happened yet!
        let event = system_interface.event();
        let oldest_event = Instant::now(); // immediately after event received
        if let Event::Quit { .. } = event {
            break 'running;
        }
        events_accumulator.push(UIEvent::new(event));

        // don't send off the event immediately! wait a bit and accumulate
        // several events to be processed together. max bound on waiting so that
        // the first event received isn't too stale
        loop {
            let max_time = oldest_event + max_delay;
            let now = Instant::now();
            if max_time <= now {
                break; // can't wait any longer
            }

            let time_to_wait = max_time - now;
            let event = match system_interface.event_timeout(time_to_wait) {
                None => break, // waited too long
                Some(v) => v,
            };
            if let Event::Quit { .. } = event {
                // even though we are exiting, still handle the events that were
                // received so far
                let _ignore = handler(system_interface, &mut events_accumulator)?;
                break 'running;
            }
            events_accumulator.push(UIEvent::new(event));
        }

        if handler(system_interface, &mut events_accumulator)? {
            break 'running;
        }
        events_accumulator.clear(); // clear after use
    }
    Ok(())
}
