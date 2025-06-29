use std::{path::Path, time::Duration};

use game_system::{
    core::color::Color,
    ui::{
        util::{
            aspect_ratio::AspectRatioFailPolicy,
            length::{
                MaxLen, MaxLenFailPolicy, MaxLenPolicy, MinLen, MinLenFailPolicy, MinLenPolicy,
            },
        },
        widget::{
            gui_loop,
            horizontal_layout::HorizontalLayout,
            image_display::ImageDisplay,
            strut::Strut,
            update_gui,
            vertical_layout::{MajorAxisMaxLenPolicy, VerticalLayout},
            FrameTransiency, HandlerReturnValue, Widget,
        },
    },
};

fn do_example<T: game_system::core::System>(
    font_file_content: &'static [u8],
) -> Result<(), String> {
    const WIDTH: f32 = 800.;
    const HEIGHT: f32 = 400.;

    let image_path = Path::new(".")
        .join("examples")
        .join("assets")
        .join("test.jpg");

    let mut system = T::new(
        Some((
            "layout test",
            (WIDTH as u32).try_into().unwrap(),
            (HEIGHT as u32).try_into().unwrap(),
        )),
        font_file_content,
        false,
    )?;

    const DELAY: Duration = Duration::from_micros(16666);

    gui_loop(DELAY, &mut system, |system, events, dt| {
        // constructs the whole GUI each frame. other examples don't bother
        // doing this, but it's key to claiming it's a "immediate mode" gui.
        let mut horizontal_0 = ImageDisplay::new(&["examples", "assets", "test.jpg"][..]);
        horizontal_0.aspect_ratio_fail_policy = AspectRatioFailPolicy::Stretch;
        horizontal_0.request_aspect_ratio = false;
        // horizontal_0.
        horizontal_0.min_h_policy = MinLenPolicy::Literal((HEIGHT - 20.).into());
        horizontal_0.min_w_policy = MinLenPolicy::Literal(100f32.into());
        horizontal_0.max_h_policy = MaxLenPolicy::Literal((HEIGHT - 20.).into());
        horizontal_0.max_w_policy = MaxLenPolicy::Literal((WIDTH / 5.).into());

        let mut horizontal_1 = ImageDisplay::new(image_path.clone());
        horizontal_1.aspect_ratio_fail_policy = AspectRatioFailPolicy::Stretch;
        horizontal_1.request_aspect_ratio = false;
        horizontal_1.min_h_policy = MinLenPolicy::Literal((HEIGHT - 20.).into());
        horizontal_1.min_w_policy = MinLenPolicy::Literal(100f32.into());
        horizontal_1.max_h_policy = MaxLenPolicy::Literal((HEIGHT - 20.).into());
        horizontal_1.max_w_policy = MaxLenPolicy::Literal((WIDTH / 4.).into());
        horizontal_1.max_h_fail_policy = MaxLenFailPolicy::POSITIVE;
        horizontal_1.min_h_fail_policy = MinLenFailPolicy::POSITIVE;

        let mut horizontal_2 = ImageDisplay::new(image_path.clone());
        horizontal_2.aspect_ratio_fail_policy = AspectRatioFailPolicy::Stretch;
        horizontal_2.request_aspect_ratio = false;
        horizontal_2.min_h_policy = MinLenPolicy::Literal((HEIGHT - 20.).into());
        horizontal_2.min_w_policy = MinLenPolicy::Literal(100f32.into());
        horizontal_2.max_h_policy = MaxLenPolicy::Literal((HEIGHT - 20.).into());
        horizontal_2.max_w_policy = MaxLenPolicy::Literal((WIDTH / 3.).into());
        horizontal_2.max_h_fail_policy = MaxLenFailPolicy::NEGATIVE;
        horizontal_2.min_h_fail_policy = MinLenFailPolicy::NEGATIVE;

        let horizontal_3 = Strut::new((0.0.into(), 0.0.into()), (20.0.into(), 0.0.into()));

        let mut v_elem_0 = ImageDisplay::new(image_path.clone());
        v_elem_0.aspect_ratio_fail_policy = AspectRatioFailPolicy::Stretch;
        v_elem_0.request_aspect_ratio = false;
        v_elem_0.min_h_policy = MinLenPolicy::Literal((HEIGHT / 4.).into());
        v_elem_0.min_w_policy = MinLenPolicy::Literal(MinLen::LAX);
        v_elem_0.max_h_policy = MaxLenPolicy::Literal((HEIGHT / 3.).into());
        v_elem_0.max_w_policy = MaxLenPolicy::Literal(MaxLen::LAX);
        v_elem_0.pref_h = 0.5.into();

        let mut v_elem_1 = ImageDisplay::new(image_path.clone());
        v_elem_1.aspect_ratio_fail_policy = AspectRatioFailPolicy::Stretch;
        v_elem_1.request_aspect_ratio = false;
        v_elem_1.min_h_policy = MinLenPolicy::Literal((HEIGHT / 4.).into());
        v_elem_1.min_w_policy = MinLenPolicy::Literal(MinLen::LAX);
        v_elem_1.max_h_policy = MaxLenPolicy::Literal((HEIGHT / 2.).into());
        v_elem_1.max_w_policy = MaxLenPolicy::Literal(MaxLen::LAX);
        v_elem_1.pref_h = 0.5.into();

        let mut v_elem_2 = ImageDisplay::new(image_path.clone());
        v_elem_2.aspect_ratio_fail_policy = AspectRatioFailPolicy::Stretch;
        v_elem_2.request_aspect_ratio = false;
        v_elem_2.min_h_policy = MinLenPolicy::Literal((HEIGHT / 4.).into());
        v_elem_2.min_w_policy = MinLenPolicy::Literal(MinLen::LAX);
        v_elem_2.max_h_policy = MaxLenPolicy::Literal((HEIGHT / 3.).into());
        v_elem_2.max_w_policy = MaxLenPolicy::Literal(MaxLen::LAX);
        v_elem_2.pref_h = 0.5.into();

        let mut horizontal_4 = VerticalLayout::<'_, T> {
            max_h_policy: MajorAxisMaxLenPolicy::Spread,
            ..Default::default()
        };
        horizontal_4.elems.push(Box::new(v_elem_0));
        horizontal_4.elems.push(Box::new(v_elem_1));
        horizontal_4.elems.push(Box::new(v_elem_2));

        let mut v_elem_0 = ImageDisplay::new(image_path.clone());
        v_elem_0.aspect_ratio_fail_policy = AspectRatioFailPolicy::Stretch;
        v_elem_0.request_aspect_ratio = false;
        v_elem_0.min_h_policy = MinLenPolicy::Literal((HEIGHT / 4.).into());
        v_elem_0.min_w_policy = MinLenPolicy::Literal(MinLen::LAX);
        v_elem_0.max_h_policy = MaxLenPolicy::Literal((HEIGHT / 3.).into());
        v_elem_0.max_w_policy = MaxLenPolicy::Literal(MaxLen::LAX);
        v_elem_0.pref_h = 0.5.into();

        let mut v_elem_1 = ImageDisplay::new(image_path.clone());
        v_elem_1.aspect_ratio_fail_policy = AspectRatioFailPolicy::Stretch;
        v_elem_1.request_aspect_ratio = false;
        v_elem_1.min_h_policy = MinLenPolicy::Literal((HEIGHT / 4.).into());
        v_elem_1.min_w_policy = MinLenPolicy::Literal(MinLen::LAX);
        v_elem_1.max_h_policy = MaxLenPolicy::Literal((HEIGHT / 2.).into());
        v_elem_1.max_w_policy = MaxLenPolicy::Literal(MaxLen::LAX);
        v_elem_1.pref_h = 0.5.into();

        let mut v_elem_2 = ImageDisplay::new(image_path.clone());
        v_elem_2.aspect_ratio_fail_policy = AspectRatioFailPolicy::Stretch;
        v_elem_2.request_aspect_ratio = false;
        v_elem_2.min_h_policy = MinLenPolicy::Literal((HEIGHT / 4.).into());
        v_elem_2.min_w_policy = MinLenPolicy::Literal(MinLen::LAX);
        v_elem_2.max_h_policy = MaxLenPolicy::Literal((HEIGHT / 3.).into());
        v_elem_2.max_w_policy = MaxLenPolicy::Literal(MaxLen::LAX);
        v_elem_2.pref_h = 0.5.into();
        let mut horizontal_5 = VerticalLayout::<'_, T> {
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

        if !matches!(r, FrameTransiency::NextFrameNow) {
            system.clear(Color {
                r: 0,
                g: 0,
                b: 0,
                a: 0xFF,
            })?;
            horizontal_layout.draw(system)?;
            system.present()?;
        }

        Ok(HandlerReturnValue::Some(r))
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
