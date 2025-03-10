// test script that moves through the functionality of a thing

use std::time::Instant;
use std::{num::NonZero, path::Path, time::Duration};

use game_system::LoopingSoundHandle;
use game_system::{Texture, TextureArea, TextureDestination};
use sdl2::pixels::Color;

fn do_test<'a, T: game_system::System<'a>>(font_file_content: &'a [u8]) -> Result<(), String> {
    let mut system = T::new(font_file_content)?;

    let window_size = system.size()?;
    {
        let image_path = Path::new(".")
            .join("examples")
            .join("assets")
            .join("test.jpg");
        let mut test_texture = system.texture(&image_path)?;
        let test_texture_size = test_texture.size()?;

        // top right, copy with no rotation and simple scaling
        test_texture.copy(
            TextureArea {
                x: 0,
                y: 0,
                w: test_texture_size.0,
                h: test_texture_size.1,
            },
            TextureDestination::Int(
                TextureArea {
                    x: window_size.0.get() as i32 - 200,
                    y: 0,
                    w: 200.try_into().unwrap(),
                    h: 200.try_into().unwrap(),
                },
                None,
                Color::RGB(255, 0, 255),
            ),
        )?;

        // top left, copy with no rotation and smooth scaling
        test_texture.copy(
            TextureArea {
                x: 0,
                y: 0,
                w: test_texture_size.0,
                h: test_texture_size.1,
            },
            TextureArea {
                x: 0,
                y: 0,
                w: 400.try_into().unwrap(),
                h: 400.try_into().unwrap(),
            },
        )?;
    }

    {
        let mut test_text = system.static_text(
            "press escape after sounds".try_into()?,
            NonZero::new(64).unwrap(),
            None,
        )?;

        let test_texture_size = test_text.size()?;

        // bottom left, copy with no rotation and smooth scaling
        test_text.copy(
            TextureArea {
                x: 0,
                y: 0,
                w: test_texture_size.0,
                h: test_texture_size.1,
            },
            TextureArea {
                x: 0,
                y: 400.try_into().unwrap(),
                w: test_texture_size.0,
                h: test_texture_size.1,
            },
        )?;
    }

    system.present()?;

    let noise_sound = Path::new(".")
        .join("examples")
        .join("assets")
        .join("noise.mp3");

    // twice of left ear, quite. and once on right ear, loud
    for _ in 0..3 {
        system.sound(&noise_sound, 0.75, 0.95)?;
        std::thread::sleep(Duration::from_millis(175));
        system.sound(&noise_sound, 0.25, 0.)?;
        std::thread::sleep(Duration::from_millis(175));
    }

    let mut handle = T::LoopingSoundHandle::new(&noise_sound);

    let speed = 1000;
    // right to left
    for i in 0..speed {
        system.loop_sound(
            &mut handle,
            0.25 + (i as f32) / (speed as f32 * 2.),
            0.5,
            None,
        )?;
        std::thread::sleep(Duration::from_millis(1));
    }
    // left to right
    for i in 0..1000 {
        system.loop_sound(
            &mut handle,
            0.75 - (i as f32) / (speed as f32 * 2.),
            0.5,
            None,
        )?;
        std::thread::sleep(Duration::from_millis(1));
    }

    // fade out
    system.stop_loop_sound(&mut handle, Some(Duration::from_millis(1000)));
    std::thread::sleep(Duration::from_millis(1000));

    // fade in then out from center
    let mut handle = T::LoopingSoundHandle::new(&noise_sound);
    system.loop_sound(&mut handle, 0., 0.5, Some(Duration::from_millis(1000)))?;
    std::thread::sleep(Duration::from_millis(1000));
    system.stop_loop_sound(&mut handle, Some(Duration::from_millis(1000)));
    std::thread::sleep(Duration::from_millis(1000));

    // music tests!

    // play sound fading in
    system.music(
        &noise_sound,
        Some(Duration::from_millis(250)),
        Some(Duration::from_millis(250)),
    )?;
    std::thread::sleep(Duration::from_millis(750));
    // fade it out and replace it
    system.music(
        &noise_sound,
        Some(Duration::from_millis(250)),
        Some(Duration::from_millis(250)),
    )?;
    std::thread::sleep(Duration::from_millis(750));
    // // fade it out and replace it abrupt
    // system.music(&noise_sound, None, Some(Duration::from_millis(250)))?;
    // std::thread::sleep(Duration::from_millis(750));
    // // fade it out and replace it abrupt
    // system.music(&noise_sound, Some(Duration::from_millis(250)), None)?;
    // system.music(&noise_sound, Some(Duration::from_millis(250)), None)?;
    // system.music(&noise_sound, Some(Duration::from_millis(250)), None)?;
    // system.music(&noise_sound, Some(Duration::from_millis(250)), None)?;
    // system.music(&noise_sound, Some(Duration::from_millis(250)), None)?;
    // system.music(&noise_sound, Some(Duration::from_millis(250)), None)?;
    // system.music(&noise_sound, Some(Duration::from_millis(250)), None)?;
    // std::thread::sleep(Duration::from_millis(750));
    // stop it abruptly
    system.stop_music(Some(Duration::from_millis(250)))?;

    loop {
        let before = Instant::now();
        let maybe_event = system.event_timeout(Duration::from_millis(17));
        match maybe_event {
            None => {}
            Some(event) => {
                match event {
                    game_system::Event::Quit => break,
                    game_system::Event::Key(key_event) => {
                        if key_event.key == 27 {
                            // ESC
                            break;
                        }
                    }
                    _ => {}
                }
            }
        }
        println!(
            "{} {:?}",
            (Instant::now() - before).as_millis(),
            maybe_event
        );
    }

    Ok(())
}

fn main() -> Result<(), String> {
    let font_file_contents = include_bytes!("assets/TEMPSITC-REDUCED.TTF");

    #[cfg(feature = "rust-sdl2")]
    return do_test::<game_system::backends::rust_sdl2::RustSDL2System>(font_file_contents);

    // OTHER BACKENDS HERE
    // ...

    #[allow(unreachable_code)]
    Err("No backend enabled! Enable a feature (e.g., `--features rust-sdl2`).".to_owned())
}
