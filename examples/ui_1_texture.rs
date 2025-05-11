use std::{path::Path, time::Duration};

use game_system::{
    core::color::Color,
    ui::{
        util::{
            aspect_ratio::AspectRatioFailPolicy,
            length::{MaxLen, MaxLenPolicy, MinLen, MinLenPolicy},
        },
        widget::horizontal_layout::HorizontalLayout,
        widget::{gui_loop, image_display::ImageDisplay, update_gui, HandlerReturnValue, Widget},
    },
};

// NOTE: the zoom in case truncates to the nearest integer. for details, see the
// note in game_system::ui::widget::background::Background

fn do_example<T: game_system::core::System>(
    font_file_content: &'static [u8],
) -> Result<(), String> {
    const WIDTH: u32 = 256 * 4;
    const HEIGHT: u32 = 256;
    const DELAY: Duration = Duration::from_micros(16666);

    let image_path = Path::new(".")
        .join("examples")
        .join("assets")
        .join("test.jpg");

    let mut system = T::new(
        Some((
            "left three are aspect ratio failures. last one requests aspect ratio",
            (WIDTH as u32).try_into().unwrap(),
            (HEIGHT as u32).try_into().unwrap(),
        )),
        font_file_content,
        false,
    )?;

    let mut texture0 = ImageDisplay::new(image_path.clone());
    let mut texture1 = ImageDisplay::new(image_path.clone());
    let mut texture2 = ImageDisplay::new(image_path.clone());
    let mut texture3 = ImageDisplay::new(image_path);

    texture0.aspect_ratio_fail_policy = AspectRatioFailPolicy::Stretch;
    texture0.request_aspect_ratio = false;
    texture0.min_w_policy = MinLenPolicy::Literal(MinLen::LAX);
    texture0.max_w_policy = MaxLenPolicy::Literal(MaxLen::LAX);
    texture0.min_h_policy = MinLenPolicy::Literal(MinLen::LAX);
    texture0.max_h_policy = MaxLenPolicy::Literal(MaxLen::LAX);

    texture1.aspect_ratio_fail_policy = AspectRatioFailPolicy::ZoomOut((0.5, 0.5));
    texture1.request_aspect_ratio = false;
    texture1.min_w_policy = MinLenPolicy::Literal(MinLen::LAX);
    texture1.max_w_policy = MaxLenPolicy::Literal(MaxLen::LAX);
    texture1.min_h_policy = MinLenPolicy::Literal(MinLen::LAX);
    texture1.max_h_policy = MaxLenPolicy::Literal(MaxLen::LAX);

    texture2.aspect_ratio_fail_policy = AspectRatioFailPolicy::ZoomIn((0.5, 0.5));
    texture2.request_aspect_ratio = false;
    texture2.min_w_policy = MinLenPolicy::Literal(MinLen::LAX);
    texture2.max_w_policy = MaxLenPolicy::Literal(MaxLen::LAX);
    texture2.min_h_policy = MinLenPolicy::Literal(MinLen::LAX);
    texture2.max_h_policy = MaxLenPolicy::Literal(MaxLen::LAX);

    texture3.preferred_ratio_exceed_parent = true;
    texture3.min_w_policy = MinLenPolicy::Literal(MinLen::LAX);
    texture3.max_w_policy = MaxLenPolicy::Literal(MaxLen::LAX);
    texture3.min_h_policy = MinLenPolicy::Literal(MinLen::LAX);
    texture3.max_h_policy = MaxLenPolicy::Literal(MaxLen::LAX);

    let mut horizontal_layout = HorizontalLayout::default();
    horizontal_layout.elems.push(Box::new(texture0));
    horizontal_layout.elems.push(Box::new(texture1));
    horizontal_layout.elems.push(Box::new(texture2));
    horizontal_layout.elems.push(Box::new(texture3));

    gui_loop(DELAY, &mut system, |system, events, dt| {
        let r = update_gui(&mut horizontal_layout, events, system, dt)?;

        // after gui update, use whatever events are left
        for e in events.iter_mut().filter(|e| e.is_some()) {
            match e.unwrap() {
                game_system::core::event::Event::Mouse(mouse_event) => {
                    if mouse_event.down && mouse_event.changed {
                        *e = None; // intentional redundant
                        println!(
                            "nothing consumed the click! {:?}",
                            (mouse_event.x, mouse_event.y)
                        );
                    }
                }
                game_system::core::event::Event::Key(key_event) => {
                    if key_event.key == 27 {
                        // esc
                        *e = None; // intentional redundant
                        return Ok(HandlerReturnValue::Stop);
                    }
                }
                game_system::core::event::Event::Quit => {
                    *e = None; // intentional redundant
                    return Ok(HandlerReturnValue::Stop);
                }
                _ => {}
            }
        }

        system.clear(Color {
            r: 0,
            g: 0,
            b: 0,
            a: 0xFF,
        })?;
        horizontal_layout.draw(system)?;
        system.present()?;
        Ok(match r {
            true => HandlerReturnValue::NextFrame,
            false => HandlerReturnValue::DelayNextFrame,
        })
    })?;
    Ok(())
}

fn main() -> Result<(), String> {
    let font_file_contents = include_bytes!("assets/TEMPSITC-REDUCED.TTF");

    #[cfg(feature = "rust-sdl2")]
    return do_example::<game_system::core::backends::rust_sdl2::RustSDL2System>(
        font_file_contents,
    );

    // OTHER BACKENDS HERE
    // ...

    #[allow(unreachable_code)]
    Err("No backend enabled! Enable a feature (e.g., `--features rust-sdl2`).".to_owned())
}
