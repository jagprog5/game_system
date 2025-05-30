use std::{cell::Cell, time::Duration};

use game_system::{
    core::color::Color,
    ui::{
        util::{
            aspect_ratio::AspectRatioFailPolicy,
            length::{MaxLen, MaxLenFailPolicy, MinLen, MinLenFailPolicy},
            rust::CellRefOrCell,
        },
        widget::{
            gui_loop,
            multi_line_label::{MultiLineLabel, MultiLineMinHeightFailPolicy},
            single_line_label::SingleLineLabel,
            update_gui, HandlerReturnValue, Widget,
        },
        widget::{horizontal_layout::HorizontalLayout, vertical_layout::VerticalLayout},
    },
};

fn do_example<T: game_system::core::System>(
    font_file_content: &'static [u8],
) -> Result<(), String> {
    const WIDTH: u32 = 800;
    const HEIGHT: u32 = 600;
    const DELAY: Duration = Duration::from_micros(16666);

    let mut system = T::new(
        Some((
            "labels",
            (WIDTH as u32).try_into().unwrap(),
            (HEIGHT as u32).try_into().unwrap(),
        )),
        font_file_content,
        true,
    )?;

    // ====================== TOP LABEL ========================================

    let top_label_text = Cell::new("hello".to_owned());

    let mut top_label = SingleLineLabel::new(CellRefOrCell::Ref(&top_label_text));

    top_label.min_h_fail_policy = MinLenFailPolicy::NEGATIVE; // go up if too small
    top_label.min_h = MinLen(50.); // for testing
    top_label.max_h = MaxLen(150.);

    // right align in vertical layout
    top_label.max_w_fail_policy = MaxLenFailPolicy::POSITIVE;
    top_label.min_w_fail_policy = MinLenFailPolicy::NEGATIVE;

    // ====================== MIDDLE LABEL =====================================

    let mut middle_label = SingleLineLabel::new("the quick brown fox".into());
    middle_label.request_aspect_ratio = false;

    // ======================== BOTTOM LABELS ==================================

    let bottom_left_label =
        SingleLineLabel::new(CellRefOrCell::from(Cell::new("horizontal".to_owned())));

    let mut bottom_right_label =
        SingleLineLabel::new(CellRefOrCell::from(Cell::new("horizontal2q|".to_owned())));
    bottom_right_label.min_h_fail_policy = MinLenFailPolicy::NEGATIVE;
    bottom_right_label.min_h = MinLen(50.); // for testing
    bottom_right_label.max_h = MaxLen(100.);
    // right align + varying size in horizontal layout is a bit more tricky
    bottom_right_label.max_w_fail_policy = MaxLenFailPolicy::POSITIVE;
    bottom_right_label.min_w_fail_policy = MinLenFailPolicy::NEGATIVE;
    bottom_right_label.aspect_ratio_fail_policy = AspectRatioFailPolicy::ZoomOut((1., 0.5));

    let multiline_string_displayed = "Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat. Duis aute irure dolor in reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla pariatur. Excepteur sint occaecat cupidatat non proident, sunt in culpa qui officia deserunt mollit anim id est laborum.".to_owned();
    let mut multiline_widget =
        MultiLineLabel::new(multiline_string_displayed.into(), 20.try_into().unwrap());
    multiline_widget.min_h_policy = MultiLineMinHeightFailPolicy::CutOff(1.0);
    multiline_widget.max_h_policy = MaxLenFailPolicy::NEGATIVE;

    let mut bottom_layout = HorizontalLayout::default();
    let mut layout = VerticalLayout::default();
    layout.elems.push(Box::new(top_label));
    layout.elems.push(Box::new(middle_label));
    layout.elems.push(Box::new(multiline_widget));
    bottom_layout.elems.push(Box::new(bottom_left_label));
    bottom_layout.elems.push(Box::new(bottom_right_label));
    layout.elems.push(Box::new(bottom_layout));

    gui_loop(DELAY, &mut system, |system, events, dt| {
        for e in events.iter_mut().filter(|e| e.is_some()) {
            match e.unwrap() {
                game_system::core::event::Event::Window(window) => {
                    top_label_text.set(format!("{}x{}", window.width.get(), window.height.get()));
                }
                _ => {}
            }
        }
        let r = update_gui(&mut layout, events, system, dt)?;

        // after gui update, use whatever events are left
        for e in events.iter_mut().filter(|e| e.is_some()) {
            match e.unwrap() {
                game_system::core::event::Event::Mouse(mouse_event) => {
                    if !mouse_event.down && mouse_event.changed {
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
        layout.draw(system)?;
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
