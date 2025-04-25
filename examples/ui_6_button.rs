use std::{cell::Cell, num::NonZeroU32, path::Path, time::Duration};

use game_system::{
    core::{color::Color, texture_rect::TextureRect},
    ui::{
        util::length::{MaxLen, MaxLenFailPolicy, MinLenFailPolicy, PreferredPortion},
        widget::vertical_layout::VerticalLayout,
        widget::{
            background::Background,
            border::Border,
            button::{Button, ButtonInheritSizing},
            gui_loop,
            multi_line_label::{MultiLineLabel, MultiLineMinHeightFailPolicy},
            scroller::{DragState, Scroller},
            single_line_label::SingleLineLabel,
            sizing::{CustomSizing, NestedContentSizing},
            tiled_texture::TiledTexture,
            update_gui, HandlerReturnValue, Widget,
        },
    },
};

fn do_example<T: game_system::core::System>(
    font_file_content: &'static [u8],
) -> Result<(), String> {
    const WIDTH: u32 = 400;
    const HEIGHT: u32 = 250;
    const DELAY: Duration = Duration::from_micros(16666);

    let window_settings = (
        "button",
        (WIDTH as u32).try_into().unwrap(),
        (HEIGHT as u32).try_into().unwrap(),
    );

    let mut system = T::new(Some(window_settings), font_file_content)?;

    // a button with a border and background

    let button_release = Cell::new(false);

    let idle = SingleLineLabel::new::<T>("idle".into());
    let mut hovered = SingleLineLabel::new::<T>("hovered".into());
    hovered.max_h = MaxLen(40.);
    let pressed = SingleLineLabel::new::<T>("pressed".into());

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
    button.hotkey = Some(b'a');

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
        Box::new(button),
        Box::new(TiledTexture::new((
            background_path.clone(),
            TextureRect {
                x: 0,
                y: 0,
                w: sixteen,
                h: sixteen,
            }
            .into(),
        ))),
    );

    let button = Border::new(
        Box::new(background),
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

    // next, have some scrollable text
    let mut text = MultiLineLabel::new(
        "scroll down to read! this\nis\na lot of\nmultiline text\nand a ton of lore as well!"
            .into(),
        20.try_into().unwrap(),
        Color {
            r: 0xFF,
            g: 0xFF,
            b: 0xFF,
            a: 0xFF,
        },
    );
    // the multiline widget's bounds should respect the text (don't cut it off
    // or get around it in some other way) - and if the widget is too big then
    // allow it to expand downwards. and if it's too small, then stay upwards
    text.min_h_policy =
        MultiLineMinHeightFailPolicy::None(MinLenFailPolicy::POSITIVE, MaxLenFailPolicy::NEGATIVE);

    let drag_state = Cell::new(DragState::default());
    let scroll_x = Cell::new(0i32);
    let scroll_y = Cell::new(0i32);

    // put the text in a vertical scroller
    let mut scroller = Scroller::new(
        false,
        true,
        &drag_state,
        &scroll_x,
        &scroll_y,
        Box::new(text),
    );
    scroller.lock_small_content_y = Some(MaxLenFailPolicy::NEGATIVE);
    scroller.sizing = NestedContentSizing::Custom(Default::default());

    let mut layout = VerticalLayout::default();
    layout.elems.push(Box::new(scroller));
    layout.elems.push(Box::new(button));

    let mut background_sizing = CustomSizing::default();
    background_sizing.preferred_h = PreferredPortion(0.75);
    background_sizing.preferred_w = PreferredPortion(0.75);

    let mut background = Background::new(
        Box::new(layout),
        Box::new(TiledTexture::new((
            background_path,
            TextureRect {
                x: 0,
                y: 0,
                w: sixteen,
                h: sixteen,
            }
            .into(),
        ))),
    );
    background.sizing = NestedContentSizing::Custom(background_sizing);

    gui_loop(DELAY, &mut system, |system, events, dt| {
        let r = update_gui(&mut background, events, system, dt)?;

        if button_release.get() {
            system.sound(&button_sound_path, 0., 0.)?;
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
