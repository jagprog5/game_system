use std::{cell::Cell, num::NonZeroU32, path::Path, time::Duration};

use example_common::gui_loop::gui_loop;
use game_system::{
    core::{color::Color, texture_area::TextureArea},
    ui::{util::length::MaxLen, widget::{
        background::Background, border::Border, button::{Button, ButtonInheritSizing}, single_line_label::SingleLineLabel, update_gui, Widget
    }},
};

#[path = "example_common/mod.rs"]
mod example_common;

fn do_example<'font_data, T: game_system::core::System<'font_data> + 'font_data>(
    font_file_content: &'font_data [u8],
) -> Result<(), String> {
    const WIDTH: u32 = 400;
    const HEIGHT: u32 = 400;
    const MAX_DELAY: Duration = Duration::from_millis(17);

    let window_settings = (
        "button",
        (WIDTH as u32).try_into().unwrap(),
        (HEIGHT as u32).try_into().unwrap(),
    );

    let mut system = T::new(Some(window_settings), font_file_content)?;

    let button_release = Cell::new(false);

    let idle = SingleLineLabel::new::<'font_data, T>("idle".into());
    let mut hovered = SingleLineLabel::new::<'font_data, T>("hovered".into());
    hovered.max_h = MaxLen(40.);
    let pressed = SingleLineLabel::new::<'font_data, T>("pressed".into());

    let button_sound_path = Path::new(".")
        .join("examples")
        .join("assets")
        .join("ui_test_sound.mp3");

    let mut button = Button::new(
        Box::new(idle),
        Box::new(hovered),
        Box::new(pressed),
        &button_release,
    );
    button.sizing_inherit_choice = ButtonInheritSizing::Hovered;
    button.release_sound = Some(button_sound_path);

    let background_path = Path::new(".")
        .join("examples")
        .join("assets")
        .join("background.jpg");

    let border_path = Path::new(".")
        .join("examples")
        .join("assets")
        .join("border.png");

    let sixteen = unsafe { NonZeroU32::new_unchecked(16) };

    let background = Background::new(
        Some((
            background_path,
            TextureArea {
                x: 0,
                y: 0,
                w: sixteen,
                h: sixteen,
            }
            .into(),
        )),
        Box::new(button),
    );

    let mut border = Border::new(Box::new(background), border_path, TextureArea {
        x: 0,
        y: 0,
        w: 15.try_into().unwrap(),
        h: 5.try_into().unwrap(),
    }, TextureArea {
        x: 16,
        y: 0,
        w: 5.try_into().unwrap(),
        h: 5.try_into().unwrap(),
    });

    gui_loop(MAX_DELAY, &mut system, |system, events| {
        update_gui(&mut border, events, system)?;

        if button_release.get() {
            println!("button was pressed");
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
                        return Ok(true);
                    }
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
        Ok(false)
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
