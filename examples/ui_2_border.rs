use std::{path::Path, time::Duration};

use game_system::{
    core::{color::Color, texture_rect::TextureRect},
    ui::{
        util::{
            aspect_ratio::AspectRatioFailPolicy,
            length::{MaxLen, MaxLenPolicy, MinLen, MinLenPolicy, PreferredPortion},
        },
        widget::{
            border::Border, gui_loop, texture::Texture, update_gui, HandlerReturnValue, Widget,
        },
    },
};

fn do_example<T: game_system::core::System>(
    font_file_content: &'static [u8],
) -> Result<(), String> {
    const WIDTH: u32 = 400;
    const HEIGHT: u32 = 400;
    const DELAY: Duration = Duration::from_micros(16666);

    let image_path = Path::new(".")
        .join("examples")
        .join("assets")
        .join("test.jpg");

    let border_path = Path::new(".")
        .join("examples")
        .join("assets")
        .join("border.png");

    let mut system = T::new(
        Some((
            "border",
            (WIDTH as u32).try_into().unwrap(),
            (HEIGHT as u32).try_into().unwrap(),
        )),
        font_file_content,
        false,
    )?;

    let mut texture_widget = Texture::new(image_path);
    texture_widget.aspect_ratio_fail_policy = AspectRatioFailPolicy::Stretch;
    texture_widget.request_aspect_ratio = false;
    texture_widget.pref_w = PreferredPortion(0.5);
    texture_widget.pref_h = PreferredPortion(0.5);
    texture_widget.min_w_policy = MinLenPolicy::Literal(MinLen::LAX);
    texture_widget.max_w_policy = MaxLenPolicy::Literal(MaxLen::LAX);
    texture_widget.min_h_policy = MinLenPolicy::Literal(MinLen::LAX);
    texture_widget.max_h_policy = MaxLenPolicy::Literal(MaxLen::LAX);

    let mut border = Border::new(
        Box::new(texture_widget),
        border_path,
        TextureRect {
            x: 0,
            y: 0,
            w: 15.try_into().unwrap(),
            h: 5.try_into().unwrap(),
        },
        TextureRect {
            x: 16,
            y: 0,
            w: 5.try_into().unwrap(),
            h: 5.try_into().unwrap(),
        },
    );

    gui_loop(DELAY, &mut system, |system, events, dt| {
        let r = update_gui(&mut border, events, system, dt)?;

        // after gui update, use whatever events are left
        for e in events.iter_mut().filter(|e| e.available()) {
            match e.e {
                game_system::core::event::Event::Mouse(mouse_event) => {
                    if mouse_event.down && mouse_event.changed {
                        e.set_consumed(); // intentional redundant
                        println!(
                            "nothing consumed the click! {:?}",
                            (mouse_event.x, mouse_event.y)
                        );
                    }
                }
                game_system::core::event::Event::Key(key_event) => {
                    if key_event.key == 27 {
                        // esc
                        e.set_consumed(); // intentional redundant
                        return Ok(HandlerReturnValue::Stop);
                    }
                }
                game_system::core::event::Event::Quit => {
                    e.set_consumed(); // intentional redundant
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
        border.draw(system)?;
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
