use std::{cell::Cell, num::NonZeroU32, path::Path, time::Duration};

use game_system::{
    core::{color::Color, texture_rect::TextureRect},
    ui::{
        util::length::PreferredPortion,
        widget::{
            background::Background,
            checkbox::CheckBox,
            gui_loop,
            sizing::{CustomSizing, NestedContentSizing},
            update_gui, HandlerReturnValue, Widget,
        },
    },
};

fn do_example<'font_data, T: game_system::core::System<'font_data> + 'font_data>(
    font_file_content: &'font_data [u8],
) -> Result<(), String> {
    const WIDTH: u32 = 400;
    const HEIGHT: u32 = 400;
    const DELAY: Duration = Duration::from_micros(16666);

    let background_path = Path::new(".")
        .join("examples")
        .join("assets")
        .join("background.jpg");

    let checkbox_path = Path::new(".")
        .join("examples")
        .join("assets")
        .join("checkbox.png");

    let window_settings = (
        "background tester",
        (WIDTH as u32).try_into().unwrap(),
        (HEIGHT as u32).try_into().unwrap(),
    );

    let mut system = T::new(Some(window_settings), font_file_content)?;

    let sixteen = unsafe { NonZeroU32::new_unchecked(16) };

    let checked = Cell::new(false);
    let changed = Cell::new(false);

    let checkbox = CheckBox::new(
        checkbox_path,
        16.0.into(),
        64.0.into(),
        &checked,
        &changed,
        TextureRect {
            x: 16 * 0,
            y: 0,
            w: sixteen,
            h: sixteen,
        },
        TextureRect {
            x: 16 * 1,
            y: 0,
            w: sixteen,
            h: sixteen,
        },
        TextureRect {
            x: 16 * 2,
            y: 0,
            w: sixteen,
            h: sixteen,
        },
        TextureRect {
            x: 16 * 3,
            y: 0,
            w: sixteen,
            h: sixteen,
        },
    );

    let mut background_sizing = CustomSizing::default();
    background_sizing.preferred_h = PreferredPortion(0.75);
    background_sizing.preferred_w = PreferredPortion(0.75);
    let mut background = Background::<'font_data, '_, T>::new(
        Some((
            background_path,
            TextureRect {
                x: 0,
                y: 0,
                w: sixteen,
                h: sixteen,
            }
            .into(),
        )),
        Box::new(checkbox),
    );
    background.sizing = NestedContentSizing::Custom(background_sizing);

    gui_loop(DELAY, &mut system, |system, events, dt| {
        let r = update_gui(&mut background, events, system, dt)?;

        if changed.get() {
            if checked.get() {
                system.recreate_window(None)?;
            } else {
                system.recreate_window(Some(window_settings))?;
            }
        }

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
        background.draw(system)?;
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
