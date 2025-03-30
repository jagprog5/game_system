use std::time::Duration;

use game_system::{
    core::color::Color,
    ui::{
        layout::{
            horizontal_layout::HorizontalLayout,
            vertical_layout::{MajorAxisMaxLenPolicy, VerticalLayout},
        },
        util::length::{MaxLenFailPolicy, MinLen, MinLenFailPolicy, MinLenPolicy},
        widget::{debug::Debug, gui_loop, strut::Strut, update_gui, HandlerReturnValue, Widget},
    },
};



fn do_example<'font_data, T: game_system::core::System<'font_data> + 'font_data>(
    font_file_content: &'font_data [u8],
) -> Result<(), String> {
    const WIDTH: f32 = 800.;
    const HEIGHT: f32 = 400.;

    let mut system = T::new(
        Some((
            "layout test",
            (WIDTH as u32).try_into().unwrap(),
            (HEIGHT as u32).try_into().unwrap(),
        )),
        font_file_content,
    )?;

    const DELAY: Duration = Duration::from_micros(16666);

    gui_loop(DELAY, &mut system, |system, events, dt| {
        // constructs the whole GUI each frame. other examples don't bother
        // doing this, but it's key to claiming it's a "immediate mode" gui.
        let mut horizontal_0 = Debug::default();
        horizontal_0.sizing.min_h = (HEIGHT - 20.).into();
        horizontal_0.sizing.min_w = 100f32.into();
        horizontal_0.sizing.max_h = (HEIGHT - 20.).into();
        horizontal_0.sizing.max_w = (WIDTH / 5.).into();
    
        let mut horizontal_1 = Debug::default();
        horizontal_1.sizing.min_h = (HEIGHT - 20.).into();
        horizontal_1.sizing.min_w = 100f32.into();
        horizontal_1.sizing.max_h = (HEIGHT - 20.).into();
        horizontal_1.sizing.max_w = (WIDTH / 4.).into();
        horizontal_1.sizing.max_h_fail_policy = MaxLenFailPolicy::POSITIVE;
        horizontal_1.sizing.min_h_fail_policy = MinLenFailPolicy::POSITIVE;
    
        let mut horizontal_2 = Debug::default();
        horizontal_2.sizing.min_h = (HEIGHT - 20.).into();
        horizontal_2.sizing.min_w = 100f32.into();
        horizontal_2.sizing.max_h = (HEIGHT - 20.).into();
        horizontal_2.sizing.max_w = (WIDTH / 3.).into();
        horizontal_2.sizing.max_h_fail_policy = MaxLenFailPolicy::NEGATIVE;
        horizontal_2.sizing.min_h_fail_policy = MinLenFailPolicy::NEGATIVE;
    
        let horizontal_3 = Strut::shrinkable(20.0.into(), 0.0.into());
    
        let mut v_elem_0 = Debug::default();
        v_elem_0.sizing.min_h = (HEIGHT / 4.).into();
        v_elem_0.sizing.max_h = (HEIGHT / 3.).into();
        v_elem_0.sizing.preferred_h = 0.5.into();
    
        let mut v_elem_1 = Debug::default();
        v_elem_1.sizing.min_h = (HEIGHT / 4.).into();
        v_elem_1.sizing.max_h = (HEIGHT / 2.).into();
        v_elem_1.sizing.preferred_h = 0.5.into();
    
        let mut v_elem_2 = Debug::default();
        v_elem_2.sizing.min_h = (HEIGHT / 4.).into();
        v_elem_2.sizing.max_h = (HEIGHT / 3.).into();
        v_elem_2.sizing.preferred_h = 0.5.into();
    
        let mut horizontal_4 = VerticalLayout::<'font_data, '_, T> {
            max_h_policy: MajorAxisMaxLenPolicy::Spread,
            ..Default::default()
        };
        horizontal_4.elems.push(Box::new(v_elem_0));
        horizontal_4.elems.push(Box::new(v_elem_1));
        horizontal_4.elems.push(Box::new(v_elem_2));
    
        let mut v_elem_0 = Debug::default();
        v_elem_0.sizing.min_h = (HEIGHT / 4.).into();
        v_elem_0.sizing.max_h = (HEIGHT / 3.).into();
        v_elem_0.sizing.preferred_h = 0.5.into();
    
        let mut v_elem_1 = Debug::default();
        v_elem_1.sizing.min_h = (HEIGHT / 4.).into();
        v_elem_1.sizing.max_h = (HEIGHT / 2.).into();
        v_elem_1.sizing.preferred_h = 0.5.into();
    
        let mut v_elem_2 = Debug::default();
        v_elem_2.sizing.min_h = (HEIGHT / 4.).into();
        v_elem_2.sizing.max_h = (HEIGHT / 3.).into();
        v_elem_2.sizing.preferred_h = 0.5.into();
        let mut horizontal_5 = VerticalLayout::<'font_data, '_, T> {
            max_h_fail_policy: MaxLenFailPolicy::NEGATIVE,
            ..Default::default()
        };
    
        horizontal_5.elems.push(Box::new(v_elem_0));
        horizontal_5.elems.push(Box::new(v_elem_1));
        horizontal_5.elems.push(Box::new(v_elem_2));
    
        let mut horizontal_layout = HorizontalLayout::default();
        // allow to be smaller than children, to show min len fail policies
        horizontal_layout.min_h_policy = MinLenPolicy::Literal(MinLen::LAX);
    
        horizontal_layout.elems.push(Box::new(horizontal_0));
        horizontal_layout.elems.push(Box::new(horizontal_1));
        horizontal_layout.elems.push(Box::new(horizontal_2));
        horizontal_layout.elems.push(Box::new(horizontal_3));
        horizontal_layout.elems.push(Box::new(horizontal_4));
        horizontal_layout.elems.push(Box::new(horizontal_5));
        let r = update_gui(&mut horizontal_layout, events, system, dt)?;

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
        horizontal_layout.draw(system)?;
        system.present()?;
        // Ok(HandlerReturnValue::NextFrame)
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
