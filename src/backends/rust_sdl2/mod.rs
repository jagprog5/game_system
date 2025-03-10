use std::{
    collections::BTreeMap,
    num::{NonZeroU16, NonZeroU32, NonZeroUsize},
    path::{Path, PathBuf},
    rc::Rc,
    time::{Duration, Instant},
};

mod font;
mod math;
mod texture_key;
mod texture_wrapper;

use font::Font;
use lru::LruCache;
use math::capped_next_power_of_two;
use sdl2::{
    image::{LoadTexture, Sdl2ImageContext}, mixer::{Channel, Chunk, Sdl2MixerContext}, mouse::MouseButton, rect::Rect, render::{Canvas, TextureCreator}, rwops::RWops, sys::SDL_Event, ttf::Sdl2TtfContext, video::{Window, WindowContext}, AudioSubsystem, EventPump, Sdl, VideoSubsystem
};
use texture_key::TextureKey;
use texture_wrapper::TextureWrapper;

use crate::{Event, NonEmptyStr, System, TextureDestination};

pub struct RustSDL2System<'font_data> {
    audio_cache: LruCache<PathBuf, Rc<Chunk>>,

    /// if a chunk is pushed out of the audio cache (causing the chunk to drop)
    /// then it will stop playing even if it shouldn't stop. by keeping a ref on
    /// the channel it is played on, this prevents this
    ///
    /// minor side-effect: up to MIX_CHANNELS (8) different copies of the chunk
    /// could exist, since channel_refs isn't consulted when the audio_cache is
    /// looked up. this worst case is fine
    channel_refs: [Option<Rc<Chunk>>; sdl2::sys::mixer::MIX_CHANNELS as usize],

    texture_cache: LruCache<TextureKey, TextureWrapper>,

    /// associates a point size with a loaded font. discretized (there can only
    /// be a handful of elements)
    loaded_fonts: BTreeMap<NonZeroU16, Font<'font_data>>,

    event_pump: EventPump,

    creator: TextureCreator<WindowContext>,
    canvas: Canvas<Window>,

    // dropped in member order stated
    ttf_context: Sdl2TtfContext,
    _image: Sdl2ImageContext,
    _mixer: Sdl2MixerContext,
    _video: VideoSubsystem,
    _audio: AudioSubsystem,
    // dropped last
    _sdl: Sdl,
    font_file_data: &'font_data [u8],
}

// exposed part of the interface
pub struct Texture<'sys> {
    txt: &'sys mut sdl2::render::Texture,
    canvas: &'sys mut Canvas<Window>,
}

// ================ constants specific to this backend =========================
/// the maximum number of textures that are loaded at any time, from image or
/// rendered text. sprite sheets recommended
static MAX_TEXTURE_CACHE_SIZE: NonZeroUsize = unsafe { NonZeroUsize::new_unchecked(100) };
static MAX_AUDIO_CACHE_SIZE: NonZeroUsize = unsafe { NonZeroUsize::new_unchecked(50) };

impl<'sys> crate::Texture<'sys> for Texture<'sys> {
    fn copy<Dst>(&mut self, src: crate::TextureArea, dst: Dst) -> Result<(), String>
    where
        Dst: Into<TextureDestination>,
    {
        let dst = dst.into();
        let src = sdl2::rect::Rect::new(src.x, src.y, src.w.into(), src.h.into());

        let texture_attributes = match dst {
            TextureDestination::Int(.., texture_attributes)
            | TextureDestination::Float(.., texture_attributes) => texture_attributes,
        };

        if texture_attributes.r != u8::MAX
            || texture_attributes.g != u8::MAX
            || texture_attributes.b != u8::MAX
        {
            // handle non default rgb mod
            self.txt.set_color_mod(
                texture_attributes.r,
                texture_attributes.g,
                texture_attributes.b,
            );
        }

        if texture_attributes.a != u8::MAX {
            // handle non default alpha mod
            self.txt.set_alpha_mod(texture_attributes.a);
        }

        let ret = match dst {
            TextureDestination::Int(dst, maybe_rotation, ..) => {
                let dst = sdl2::rect::Rect::new(dst.x, dst.y, dst.w.into(), dst.h.into());
                match maybe_rotation {
                    None => self.canvas.copy(&self.txt, src, dst),
                    Some(rot) => {
                        let angle: f32 = rot.angle.into();
                        let angle: f64 = angle.into();
                        let point = rot
                            .point
                            .map(|point| sdl2::rect::Point::from((point.0, point.1)));
                        self.canvas.copy_ex(
                            &self.txt,
                            src,
                            dst,
                            angle,
                            point,
                            rot.flip_horizontal,
                            rot.flip_vertical,
                        )
                    }
                }
            }
            TextureDestination::Float(dst, maybe_rotation, ..) => {
                let dst =
                    sdl2::rect::FRect::new(dst.x.into(), dst.y.into(), dst.w.into(), dst.h.into());
                match maybe_rotation {
                    None => self.canvas.copy_f(&self.txt, src, dst),
                    Some(rot) => {
                        let angle: f32 = rot.angle.into();
                        let angle: f64 = angle.into();
                        let point = rot.point.map(|point| {
                            sdl2::rect::FPoint::from((point.0.into(), point.1.into()))
                        });
                        self.canvas.copy_ex_f(
                            &self.txt,
                            src,
                            dst,
                            angle,
                            point,
                            rot.flip_horizontal,
                            rot.flip_vertical,
                        )
                    }
                }
            }
        };

        // reset attributes
        if texture_attributes.r != u8::MAX
            || texture_attributes.g != u8::MAX
            || texture_attributes.b != u8::MAX
        {
            self.txt.set_color_mod(u8::MAX, u8::MAX, u8::MAX);
        }

        if texture_attributes.a != u8::MAX {
            self.txt.set_alpha_mod(u8::MAX);
        }

        ret
    }

    fn size(&self) -> Result<(NonZeroU32, NonZeroU32), String> {
        let query = self.txt.query();
        let width = NonZeroU32::new(query.width).ok_or("texture width zero")?;
        let height = NonZeroU32::new(query.height).ok_or("texture height zero")?;
        Ok((width, height))
    }
}

impl<'font_data> System<'font_data> for RustSDL2System<'font_data> {
    fn new(font_file_data: &'font_data [u8]) -> Result<Self, String> {
        let sdl = sdl2::init()?;
        let video = sdl.video()?;
        let audio = sdl.audio()?;
        sdl2::mixer::open_audio(
            44_100,
            sdl2::mixer::AUDIO_S16LSB,
            sdl2::mixer::DEFAULT_CHANNELS,
            1_024,
        )?;

        let window = video
            .window("", 0, 0)
            .fullscreen_desktop()
            .build()
            .map_err(|e| e.to_string())?;

        let canvas = window
            .into_canvas()
            .present_vsync()
            .build()
            .map_err(|e| e.to_string())?;
        let creator = canvas.texture_creator();

        Ok(RustSDL2System {
            audio_cache: LruCache::new(MAX_AUDIO_CACHE_SIZE),
            channel_refs: Default::default(),
            texture_cache: LruCache::new(MAX_TEXTURE_CACHE_SIZE),
            loaded_fonts: Default::default(),
            event_pump: sdl.event_pump()?,
            creator,
            canvas,
            ttf_context: sdl2::ttf::init().map_err(|e| e.to_string())?,
            // empty flags - don't load any dynamic libs up front. they will be
            // loaded as needed the first time the respective file format is loaded
            _image: sdl2::image::init(sdl2::image::InitFlag::empty())?,
            _mixer: sdl2::mixer::init(sdl2::mixer::InitFlag::empty())?,
            _video: video,
            _audio: audio,
            _sdl: sdl,
            font_file_data,
        })
    }

    fn size(&self) -> Result<(NonZeroU32, NonZeroU32), String> {
        let raw = self.canvas.output_size()?;
        let width = NonZeroU32::new(raw.0).ok_or("canvas width zero")?;
        let height = NonZeroU32::new(raw.1).ok_or("canvas height zero")?;
        Ok((width, height))
    }

    #[allow(refining_impl_trait)]
    fn texture(&mut self, image_path: &Path) -> Result<Texture, String> {
        let texture_key = TextureKey::from_path(image_path);

        let txt = self.texture_cache.try_get_or_insert_mut_ref(
            &texture_key,
            || -> Result<TextureWrapper, String> {
                self.creator.load_texture(image_path).map(|mut txt| {
                    // Nearest scale mode is the default for sdl2 (but not sdl3!)
                    txt.set_blend_mode(sdl2::render::BlendMode::Blend);
                    TextureWrapper(txt)
                })
            },
        )?;

        Ok(Texture {
            txt: &mut txt.0,
            canvas: &mut self.canvas,
        })
    }

    fn clear(&mut self, color: sdl2::pixels::Color) -> Result<(), String> {
        self.canvas.set_draw_color(color);
        self.canvas.clear();
        Ok(())
    }

    fn present(&mut self) -> Result<(), String> {
        self.canvas.present();
        Ok(())
    }

    #[allow(refining_impl_trait)]
    fn text(
        &mut self,
        text: NonEmptyStr,
        point_size: NonZeroU16,
        wrap_width: Option<NonZeroU32>,
    ) -> Result<Texture<'_>, String> {
        // the point size is discretized in some way. that's because there is
        // some overhead associated with actually loading the font file data
        // into the font object (a font object is used per point size) - would
        // not be good to load every font size

        // the binning strategy used here is to use the next greater power of 2
        // point size (going upwards to never lose detail)
        let point_size = capped_next_power_of_two(point_size);

        let texture_key = match wrap_width {
            Some(wrap_width) => {
                TextureKey::from_rendered_wrapped_text(text.0, point_size.get(), wrap_width.get())
            }
            None => TextureKey::from_rendered_text(text.0, point_size.get()),
        };

        let txt = self.texture_cache.try_get_or_insert_mut_ref(
            &texture_key,
            || -> Result<TextureWrapper, String> {
                // must recreate the texture as it is not in the cache.
                let font = match self.loaded_fonts.get(&point_size) {
                    Some(v) => v, // point size is available
                    None => {
                        // must create font object for points size
                        let rwops =
                            RWops::from_bytes(self.font_file_data).map_err(|e| e.to_string())?;
                        let font = Font::new(&self.ttf_context, rwops, point_size.get())?;
                        self.loaded_fonts.insert(point_size, font);
                        // sanity check on discretization method
                        debug_assert!(self.loaded_fonts.len() < 20);
                        self.loaded_fonts.get(&point_size).unwrap()
                    }
                };

                // the texture is rendered!
                let surface = font.render(text.0, wrap_width)?;

                let mut texture = self
                    .creator
                    .create_texture_from_surface(surface)
                    .map_err(|e| e.to_string())?;
                texture.set_blend_mode(sdl2::render::BlendMode::Blend);
                Ok(TextureWrapper(texture))
            },
        )?;

        Ok(Texture {
            txt: &mut txt.0,
            canvas: &mut self.canvas,
        })
    }

    fn clip(&mut self, c: crate::ClippingRect) {
        self.canvas.set_clip_rect(match c {
            crate::ClippingRect::Some(texture_area) => sdl2::render::ClippingRect::Some(Rect::new(
                texture_area.x,
                texture_area.y,
                texture_area.w.get(),
                texture_area.h.get(),
            )),
            crate::ClippingRect::Zero => sdl2::render::ClippingRect::Zero,
            crate::ClippingRect::None => sdl2::render::ClippingRect::None,
        })
    }

    fn get_clip(&mut self) -> crate::ClippingRect {
        match self.canvas.clip_rect() {
            sdl2::render::ClippingRect::Some(rect) => {
                crate::ClippingRect::Some(crate::TextureArea {
                    x: rect.x,
                    y: rect.y,
                    w: rect.width().try_into().unwrap(),
                    h: rect.height().try_into().unwrap(),
                })
            }
            sdl2::render::ClippingRect::Zero => crate::ClippingRect::Zero,
            sdl2::render::ClippingRect::None => crate::ClippingRect::None,
        }
    }

    fn sound(
        &mut self,
        sound: &std::path::Path,
        direction: f32,
        distance: f32,
    ) -> Result<(), String> {
        // if a chunk's volume is changed, it applies retroactively to any
        // currently playing chunks as well. I don't like this. instead, setting
        // volume, etc, by channel is better
        let mut channel: Option<Channel> = None;

        for i in 0..sdl2::sys::mixer::MIX_CHANNELS as i32 {
            let ch = sdl2::mixer::Channel(i);
            if !ch.is_playing() {
                channel = Some(ch);
                break;
            }
        }

        let channel = match channel {
            Some(v) => v,
            None => return Ok(()), // don't do anything but don't give error
        };

        let chunk = self
            .audio_cache
            .try_get_or_insert_ref(sound, || -> Result<Rc<Chunk>, String> {
                Ok(Rc::new(Chunk::from_file(sound)?))
            })?;

        self.channel_refs[channel.0 as usize] = Some(chunk.clone());

        let angle = (direction * 360.0).round() as i16;
        let distance = (distance * 0xFF as f32).round() as u8;
        channel.set_position(angle, distance)?;
        channel.play(&chunk, 0)?;
        Ok(())
    }

    #[allow(refining_impl_trait)]
    fn loop_sound<'a>(
        &mut self,
        handle: &mut LoopingSoundHandle<'a>,
        direction: f32,
        distance: f32,
        fade_in_duration: Option<Duration>,
    ) -> Result<(), String> {
        let (channel, newly_playing) = match handle.channel {
            Some(v) => (v, false), // already playing
            None => {
                // need to reserve a channel to play on it
                let channel_to_use = (|| {
                    for i in 0..sdl2::sys::mixer::MIX_CHANNELS as i32 {
                        let ch = sdl2::mixer::Channel(i);
                        if !ch.is_playing() {
                            return Some(ch);
                        }
                    }
                    None
                })();

                match channel_to_use {
                    None => return Ok(()), // no available channels
                    Some(ch) => (ch, true),
                }
            }
        };

        let angle = (direction * 360.0).round() as i16;
        let distance = (distance * 0xFF as f32).round() as u8;
        channel.set_position(angle, distance)?;

        if newly_playing {
            let chunk = self
                .audio_cache
                .try_get_or_insert_ref(handle.path, || -> Result<Rc<Chunk>, String> {
                    Ok(Rc::new(Chunk::from_file(handle.path)?))
                })?;

            self.channel_refs[channel.0 as usize] = Some(chunk.clone());
            match fade_in_duration {
                Some(fade_in_duration) => {
                    channel.fade_in(&chunk, -1, fade_in_duration.as_millis() as i32)
                }
                None => channel.play(&chunk, -1),
            }?;
            handle.channel = Some(channel); // last step
        } else {
            // refresh the entry in the cache even if already playing
            let _ = self.audio_cache.try_get_or_insert_ref(handle.path, || {
                // it was pushed out of the cache (unlikely if adjust_sound is
                // frequent). however, it is still in the channel_refs
                let maybe_ref = &self.channel_refs[channel.0 as usize];
                // unwrap guaranteed ok since channel_refs at index was set to
                // Some() above when newly_playing. but __just in case__, doing
                // a try_get here instead. if failed, does not refresh entry
                let chunk_rc = match maybe_ref.as_ref() {
                    Some(v) => v,
                    None => {
                        debug_assert!(false);
                        return Err(());
                    }
                };
                Ok(chunk_rc.clone())
            });
        }

        Ok(())
    }

    fn stop_loop_sound<'a>(
        &mut self,
        handle: &mut Self::LoopingSoundHandle<'a>,
        fade_out_duration: Option<Duration>,
    ) {
        let channel = match handle.channel {
            Some(v) => v,
            None => return,
        };

        self.channel_refs[channel.0 as usize] = None;

        match fade_out_duration {
            Some(fade_out_duration) => {
                channel.fade_out(fade_out_duration.as_millis() as i32);
            }
            None => channel.halt(),
        }
    }

    fn event(&mut self) -> Event {
        loop {
            let maybe_e = translate_sdl_event(self.event_pump.wait_event());
            if let Some(e) = maybe_e {
                return e
            }
        }
    }

    fn event_timeout(&mut self, timeout: Duration) -> Option<crate::Event> {
        let start_time = Instant::now();
        loop {
            let duration_since_start = Instant::now() - start_time;
            if duration_since_start >= timeout {
                return None
            }

            let duration_remaining = timeout - duration_since_start;

            let event_in = self.event_pump.wait_event_timeout(duration_remaining.as_millis() as u32);
            match event_in {
                Some(e) => {
                    let maybe_e = translate_sdl_event(e);
                    if let Some(e) = maybe_e {
                        return Some(e)
                    }
                    // do another iteration
                },
                None => return None,
            }
        }
    }

    type LoopingSoundHandle<'a> = LoopingSoundHandle<'a>;
}

fn translate_sdl_event(i: sdl2::event::Event) -> Option<Event> {
    let i32_to_byte = |i: i32| -> Option<u8> {
        if (0..=255).contains(&i) {
            Some(i as u8)
        } else {
            None
        }
    };
    match i {
        sdl2::event::Event::Quit { .. } => return Some(Event::Quit),
        sdl2::event::Event::Window { win_event, .. } => match win_event {
            sdl2::event::WindowEvent::SizeChanged(w, h) => {
                let i32_to_nonzero_u32 = |i: i32| -> NonZeroU32 {
                    unsafe {
                        if i <= 0 {
                            NonZeroU32::new_unchecked(1)
                        } else {
                            NonZeroU32::new_unchecked(i as u32)
                        }
                    }
                };
                return Some(Event::Window(crate::Window {
                    width: i32_to_nonzero_u32(w),
                    height: i32_to_nonzero_u32(h),
                }));
            }
            _ => {}
        },
        sdl2::event::Event::KeyDown { keycode, .. } => {
            let keycode = match keycode {
                Some(v) => {
                    let v: i32 = *v;
                    i32_to_byte(v)
                }
                None => None,
            };
            match keycode {
                Some(key) => return Some(Event::Key(crate::KeyEvent { key, down: true })),
                None => {}
            }
        }
        sdl2::event::Event::KeyUp { keycode, .. } => {
            let keycode = match keycode {
                Some(v) => {
                    let v: i32 = *v;
                    i32_to_byte(v)
                }
                None => None,
            };
            match keycode {
                Some(key) => return Some(Event::Key(crate::KeyEvent { key, down: false })),
                None => {}
            }
        }
        sdl2::event::Event::MouseMotion {
            mousestate, x, y, ..
        } => {
            return Some(Event::Mouse(crate::MouseEvent {
                x: x as u32,
                y: y as u32,
                down: mousestate.left(),
                changed: false
            }))
        }
        sdl2::event::Event::MouseButtonDown {
            mouse_btn: MouseButton::Left,
            x,
            y,
            ..
        } => {
            // if mouse_btn.
            return Some(Event::Mouse(crate::MouseEvent {
                x: x as u32,
                y: y as u32,
                down: true,
                changed: true,
            }));
        }
        sdl2::event::Event::MouseButtonUp {
            mouse_btn: MouseButton::Left,
            x,
            y,
            ..
        } => {
            return Some(Event::Mouse(crate::MouseEvent {
                x: x as u32,
                y: y as u32,
                down: false,
                changed: true,
            }))
        }
        _ => {}
    }
    return None
}

pub struct LoopingSoundHandle<'a> {
    channel: Option<Channel>,
    path: &'a Path,
}

impl<'a> crate::LoopingSoundHandle<'a> for LoopingSoundHandle<'a> {
    fn new(path: &'a Path) -> Self {
        Self {
            channel: None,
            path,
        }
    }
}
