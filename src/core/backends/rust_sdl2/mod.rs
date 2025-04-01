mod cache_checker;
mod font;
mod math;
mod texture_key;
mod texture_wrapper;

use std::{
    collections::BTreeMap,
    num::{NonZeroU16, NonZeroU32},
    path::{Path, PathBuf},
    rc::Rc,
    sync::Mutex,
    time::{Duration, Instant},
};

use cache_checker::CacheThrashingChecker;
use font::Font;
use lru::LruCache;
use math::capped_next_power_of_two;
use sdl2::{
    image::{LoadTexture, Sdl2ImageContext},
    mixer::{Channel, Chunk, Music, Sdl2MixerContext},
    mouse::MouseButton,
    rect::Rect,
    render::{Canvas, TextureCreator},
    rwops::RWops,
    surface::Surface,
    sys::mixer::MIX_MAX_VOLUME,
    ttf::Sdl2TtfContext,
    video::{Window, WindowContext},
    AudioSubsystem, EventPump, Sdl, VideoSubsystem,
};
use texture_key::TextureKey;
use texture_wrapper::TextureWrapper;

use crate::core::{
    color::Color,
    event::MouseWheelEvent,
    texture_area::{TextureDestinationF, TextureSource, TextureSourceF},
    Event, NonEmptyStr, System, TextureDestination,
};

/// there's only one sdl mixer music callback globally which accepts a function
/// pointer. so there has to be a global state :(
struct MusicContext {
    /// the music that is currently playing right now
    pub current_music: Option<Music<'static>>,
    /// the music to play after current_music has faded out, and how long to
    /// fade it in
    pub next_music: Option<(Music<'static>, Option<Duration>)>,
}

unsafe impl Send for MusicContext {}
unsafe impl Sync for MusicContext {}

static MUSIC_CONTEXT: Mutex<MusicContext> = Mutex::new(MusicContext {
    current_music: None,
    next_music: None,
});

/// when the previous music fades out, set up the next music
fn music_finished_hook() {
    let mut ctx = MUSIC_CONTEXT.lock().unwrap();
    ctx.current_music = None;
    if let Some((next_music, fade_in_duration)) = ctx.next_music.take() {
        match fade_in_duration {
            Some(fade_in_duration) => next_music.fade_in(-1, fade_in_duration.as_millis() as i32),
            None => next_music.play(-1),
        }
        .unwrap();
        ctx.current_music = Some(next_music);
    }
}

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

    /// used for both image textures and text textures
    texture_cache: LruCache<TextureKey, TextureWrapper>,
    /// used to detect cache thrashing
    texture_cache_health_checker: CacheThrashingChecker,
    texture_cache_miss_threshold: u32,

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

impl<'font_data> Drop for RustSDL2System<'font_data> {
    fn drop(&mut self) {
        // just REALLY being sure here. I don't want any surprises later.
        // unhook before members are dropped (including Mixer Quit)
        sdl2::mixer::Music::unhook_finished();
        let mut music_context = MUSIC_CONTEXT.lock().unwrap();
        music_context.current_music = None;
        music_context.next_music = None;
    }
}

pub struct Texture<'sys> {
    txt: &'sys mut sdl2::render::Texture,
    canvas: &'sys mut Canvas<Window>,
}

impl<'sys> crate::core::TextureHandle<'sys> for Texture<'sys> {
    fn copy<Src, Dst>(&mut self, src: Src, dst: Dst) -> Result<(), String>
    where
        Src: Into<TextureSource>,
        Dst: Into<TextureDestination>,
    {
        let dst = dst.into();
        let src: TextureSource = src.into();
        let src = match src {
            TextureSource::WholeTexture => None,
            TextureSource::Area(src) => Some(sdl2::rect::Rect::new(
                src.x,
                src.y,
                src.w.into(),
                src.h.into(),
            )),
        };

        if dst.2.r != u8::MAX || dst.2.g != u8::MAX || dst.2.b != u8::MAX {
            // handle non default rgb mod
            self.txt.set_color_mod(dst.2.r, dst.2.g, dst.2.b);
        }

        if dst.2.a != u8::MAX {
            // handle non default alpha mod
            self.txt.set_alpha_mod(dst.2.a);
        }

        let ret = {
            let TextureDestination(dst, maybe_rotation, _) = dst;

            let dst = sdl2::rect::Rect::from_ll(sdl2::sys::SDL_Rect {
                x: dst.x,
                y: dst.y,
                w: dst.w.get() as i32,
                h: dst.h.get() as i32,
            });
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
        };

        // reset attributes
        if dst.2.r != u8::MAX || dst.2.g != u8::MAX || dst.2.b != u8::MAX {
            self.txt.set_color_mod(u8::MAX, u8::MAX, u8::MAX);
        }

        if dst.2.a != u8::MAX {
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

    fn copy_f<Src, Dst>(&mut self, src: Src, dst: Dst) -> Result<(), String>
    where
        Src: Into<TextureSourceF>,
        Dst: Into<crate::core::texture_area::TextureDestinationF>,
    {
        let dst = dst.into();
        let src: TextureSourceF = src.into();

        // unfortunately sdl2 does not support floating point txt src. sdl3
        // supports this though!
        let src = match src {
            TextureSourceF::WholeTexture => None,
            TextureSourceF::Area(src) => Some(sdl2::rect::Rect::new(
                src.x.get().round() as i32,
                src.y.get().round() as i32,
                src.w.get().round() as u32,
                src.h.get().round() as u32,
            )),
        };

        if dst.2.r != u8::MAX || dst.2.g != u8::MAX || dst.2.b != u8::MAX {
            // handle non default rgb mod
            self.txt.set_color_mod(dst.2.r, dst.2.g, dst.2.b);
        }

        if dst.2.a != u8::MAX {
            // handle non default alpha mod
            self.txt.set_alpha_mod(dst.2.a);
        }

        let ret = {
            let TextureDestinationF(dst, maybe_rotation, _) = dst;
            let dst =
                sdl2::rect::FRect::new(dst.x.into(), dst.y.into(), dst.w.into(), dst.h.into());
            match maybe_rotation {
                None => self.canvas.copy_f(&self.txt, src, dst),
                Some(rot) => {
                    let angle: f32 = rot.angle.into();
                    let angle: f64 = angle.into();
                    let point = rot
                        .point
                        .map(|point| sdl2::rect::FPoint::from((point.0.into(), point.1.into())));
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
        };

        // reset attributes
        if dst.2.r != u8::MAX || dst.2.g != u8::MAX || dst.2.b != u8::MAX {
            self.txt.set_color_mod(u8::MAX, u8::MAX, u8::MAX);
        }

        if dst.2.a != u8::MAX {
            self.txt.set_alpha_mod(u8::MAX);
        }

        ret
    }
}

impl<'font_data> System<'font_data> for RustSDL2System<'font_data> {
    type LoopingSoundHandle<'a> = LoopingSoundHandle<'a>;
    type ImageTextureHandle<'system>
        = Texture<'system>
    where
        Self: 'system;
    // ImageTextureHandle = TextTextureHandle here, but maybe not for other
    // backends! distinction added to help forward compatibility
    type TextTextureHandle<'system>
        = Self::ImageTextureHandle<'system>
    where
        Self: 'system;

    fn new(
        size: Option<(&str, NonZeroU32, NonZeroU32)>,
        font_file_data: &'font_data [u8],
    ) -> Result<Self, String> {
        let sdl = sdl2::init()?;
        let video = sdl.video()?;
        let audio = sdl.audio()?;
        sdl2::mixer::open_audio(
            44_100,
            sdl2::mixer::AUDIO_S16LSB,
            sdl2::mixer::DEFAULT_CHANNELS,
            1_024,
        )?;
        sdl2::mixer::Music::hook_finished(music_finished_hook);

        let window = match size {
            Some(size) => {
                let mut ret = video.window(size.0, size.1.get(), size.2.get());
                ret.resizable();
                ret
            }
            None => {
                let mut ret = video.window("", 0, 0);
                ret.fullscreen_desktop();
                ret
            }
        }
        .build()
        .map_err(|e| e.to_string())?;

        let canvas = window
            .into_canvas()
            .present_vsync()
            .build()
            .map_err(|e| e.to_string())?;
        let creator = canvas.texture_creator();

        Ok(RustSDL2System {
            // audio cache capacity is fixed. from the POV of gameplay, if too
            // many different types of sounds are playing rapidly then that
            // would be quite weird. keep in mind too, no sound is played if no
            // channel is available (and it wouldn't try to load anything in
            // that case), so there is a limit already
            audio_cache: LruCache::new(
                (sdl2::sys::mixer::MIX_CHANNELS as usize * 4)
                    .try_into()
                    .unwrap(),
            ),
            channel_refs: Default::default(),
            // texture cache has a dynamically increasing capacity with some
            // arbitrary small starting capacity. there could be cases where
            // plenty of textures are drawn to the screen at the same time -
            // wanted to account for this and make it always work
            texture_cache: LruCache::new(32.try_into().unwrap()),
            texture_cache_health_checker: Default::default(),
            // every time the texture cache is doubled, the threshold is also
            // doubled. just in case there's an entity that loads texture once
            // per frame or something (but in reality sprite sheets should be
            // used for that case)
            texture_cache_miss_threshold: 2,
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

    fn recreate_window(
        &mut self,
        size: Option<(&str, NonZeroU32, NonZeroU32)>,
    ) -> Result<(), String> {
        // texture must be dropped first, before parent canvas / creator
        self.texture_cache.clear();
        self.texture_cache_health_checker.reset();

        let window = match size {
            Some(size) => self._video.window(size.0, size.1.get(), size.2.get()),
            None => {
                let mut ret = self._video.window("", 0, 0);
                ret.fullscreen_desktop();
                ret
            }
        }
        .build()
        .map_err(|e| e.to_string())?;

        let canvas = window
            .into_canvas()
            .present_vsync()
            .build()
            .map_err(|e| e.to_string())?;
        let creator = canvas.texture_creator();

        // replacement order is super important here
        self.creator = creator;
        self.canvas = canvas;
        Ok(())
    }

    fn size(&self) -> Result<(NonZeroU32, NonZeroU32), String> {
        let raw = self.canvas.output_size()?;
        let width = NonZeroU32::new(raw.0).ok_or("canvas width zero")?;
        let height = NonZeroU32::new(raw.1).ok_or("canvas height zero")?;
        Ok((width, height))
    }

    fn texture(&mut self, image_path: &Path) -> Result<Texture, String> {
        let texture_key = TextureKey::from_path(image_path);

        let txt = self.texture_cache.try_get_or_insert_mut_ref(
            &texture_key,
            || -> Result<TextureWrapper, String> {
                self.texture_cache_health_checker.cache_miss_occurred();
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

    fn clear(&mut self, c: Color) -> Result<(), String> {
        self.canvas
            .set_draw_color(sdl2::pixels::Color::RGBA(c.r, c.g, c.b, c.a));
        self.canvas.clear();
        Ok(())
    }

    fn present(&mut self) -> Result<(), String> {
        self.canvas.present();

        // see text() for reason
        let _ = self.missing_texture()?;

        let previous_n_frames_had_cache_misses = self.texture_cache_health_checker.frame_end();
        if previous_n_frames_had_cache_misses >= self.texture_cache_miss_threshold {
            self.texture_cache_miss_threshold *= 2;
            debug_assert!(self.texture_cache.cap().get() < 1000); // sane upper bound
            self.texture_cache.resize(
                (self.texture_cache.cap().get() * 2usize)
                    .try_into()
                    .unwrap(),
            );
            self.texture_cache_health_checker.reset();
        }
        Ok(())
    }

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
                // NO!
                // self.texture_cache_health_checker.cache_miss_occurred();

                // the thinking is as follows: if there is short lived text,
                // such as from a frame counter (any of the properties change
                // each frame), then this will cause the cache to keep growing
                // forever given the current expansion rules (it gives cache
                // miss each frame and so thrashing is assumed). not having this
                // here has the same overall effect, it's just checked when
                // image textures are loaded instead (since the text will push
                // things out of the cache and then that will be detected when
                // image texture cache misses occur).

                // there is one parasitic case, if the app only ever draws text
                // and never draws textures at all, then the cache misses aren't
                // registered. for safety this is prevented by loading the debug
                // texture each frame in present()

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
                texture.set_scale_mode(sdl2::render::ScaleMode::Linear);
                Ok(TextureWrapper(texture))
            },
        )?;

        Ok(Texture {
            txt: &mut txt.0,
            canvas: &mut self.canvas,
        })
    }

    fn clip(&mut self, c: crate::core::ClippingRect) {
        self.canvas.set_clip_rect(match c {
            crate::core::ClippingRect::Some(texture_area) => {
                sdl2::render::ClippingRect::Some(Rect::new(
                    texture_area.x,
                    texture_area.y,
                    texture_area.w.get(),
                    texture_area.h.get(),
                ))
            }
            crate::core::ClippingRect::Zero => sdl2::render::ClippingRect::Zero,
            crate::core::ClippingRect::None => sdl2::render::ClippingRect::None,
        })
    }

    fn get_clip(&mut self) -> crate::core::ClippingRect {
        match self.canvas.clip_rect() {
            sdl2::render::ClippingRect::Some(rect) => {
                crate::core::ClippingRect::Some(crate::core::TextureRect {
                    x: rect.x,
                    y: rect.y,
                    w: rect.width().try_into().unwrap(),
                    h: rect.height().try_into().unwrap(),
                })
            }
            sdl2::render::ClippingRect::Zero => crate::core::ClippingRect::Zero,
            sdl2::render::ClippingRect::None => crate::core::ClippingRect::None,
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
                    // no available channels. since loop_sound is called
                    // frequently, it will try again soon
                    None => return Ok(()),
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
                return e;
            }
        }
    }

    fn event_timeout(&mut self, timeout: Duration) -> Option<crate::core::Event> {
        let start_time = Instant::now();
        loop {
            let duration_since_start = Instant::now() - start_time;
            if duration_since_start >= timeout {
                return None;
            }

            let duration_remaining = timeout - duration_since_start;
            let duration_remaining = duration_remaining.as_millis() as u32;
            if duration_remaining == 0 {
                return None; // just in case
            }

            let event_in = self.event_pump.wait_event_timeout(duration_remaining);
            match event_in {
                Some(e) => {
                    let maybe_e = translate_sdl_event(e);
                    if let Some(e) = maybe_e {
                        return Some(e);
                    }
                    // do another iteration
                }
                None => return None,
            }
        }
    }

    fn stop_music(&mut self, fade_out_duration: Option<Duration>) -> Result<(), String> {
        let mut ctx = MUSIC_CONTEXT.lock().unwrap();
        ctx.next_music = None;
        match fade_out_duration {
            Some(v) => sdl2::mixer::Music::fade_out(v.as_millis() as i32)?,
            None => {
                ctx.current_music = None;
            }
        }
        Ok(())
    }

    fn music(
        &mut self,
        music: &Path,
        fade_out_duration: Option<Duration>,
        fade_in_duration: Option<Duration>,
    ) -> Result<(), String> {
        let music = sdl2::mixer::Music::from_file(music)?;
        let mut ctx = MUSIC_CONTEXT.lock().unwrap();

        if let Some(_) = ctx.current_music.as_ref() {
            if let Some(v) = fade_out_duration {
                // some music is currently playing and a fade out was requested
                ctx.next_music = Some((music, fade_in_duration));
                sdl2::mixer::Music::fade_out(v.as_millis() as i32)?;
                return Ok(());
            }
        }

        // all other cases
        match fade_in_duration {
            Some(v) => music.fade_in(-1, v.as_millis() as i32)?,
            None => music.play(-1)?,
        }
        ctx.current_music = Some(music);
        Ok(())
    }

    fn set_music_volume(&mut self, volume: f32) {
        sdl2::mixer::Music::set_volume((volume * MIX_MAX_VOLUME as f32).round() as i32);
    }

    fn music_volume(&self) -> f32 {
        sdl2::mixer::Music::get_volume() as f32 / MIX_MAX_VOLUME as f32
    }

    fn missing_texture(&mut self) -> Result<Self::ImageTextureHandle<'_>, String> {
        let texture_key = TextureKey::debug_key();

        let txt = self.texture_cache.try_get_or_insert_mut_ref(
            &texture_key,
            || -> Result<TextureWrapper, String> {
                self.texture_cache_health_checker.cache_miss_occurred();

                // generate the debug texture

                let mut surface =
                    Surface::new(256, 256, sdl2::pixels::PixelFormatEnum::ARGB8888).unwrap();
                surface
                    .set_blend_mode(sdl2::render::BlendMode::None)
                    .unwrap();
                surface.with_lock_mut(|buffer| {
                    for x in 0i32..256 {
                        for y in 0i32..256 {
                            let pixel_offset = (4 * (x + y * 256)) as usize;
                            if x <= 3 || x >= 252 || y <= 3 || y >= 252 {
                                let v = ((x / 4 + y / 4) % 2) as u8;
                                buffer[pixel_offset] = v * 0xff;
                                buffer[pixel_offset + 1] = v * 0xff;
                                buffer[pixel_offset + 2] = v * 0xff;
                                buffer[pixel_offset + 3] = 0xff;
                            } else {
                                buffer[pixel_offset] = ((y as f32 / 255.0) * 0xFF as f32) as u8;
                                buffer[pixel_offset + 1] = ((x as f32 / 255.0) * 0xFF as f32) as u8;
                                buffer[pixel_offset + 2] = 0xFF - buffer[pixel_offset + 1];
                                buffer[pixel_offset + 3] = ((x * y) % 0xFF) as u8;
                            }
                        }
                    }
                });

                self.creator
                    .create_texture_from_surface(surface)
                    .map(|mut txt| {
                        // Nearest scale mode is the default for sdl2 (but not sdl3!)
                        txt.set_blend_mode(sdl2::render::BlendMode::Blend);
                        TextureWrapper(txt)
                    })
                    .map_err(|e| e.to_string())
            },
        )?;

        Ok(Texture {
            txt: &mut txt.0,
            canvas: &mut self.canvas,
        })
    }
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
                return Some(Event::Window(crate::core::event::Window {
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
                Some(key) => {
                    return Some(Event::Key(crate::core::event::KeyEvent { key, down: true }))
                }
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
                Some(key) => {
                    return Some(Event::Key(crate::core::event::KeyEvent {
                        key,
                        down: false,
                    }))
                }
                None => {}
            }
        }
        sdl2::event::Event::MouseMotion {
            mousestate, x, y, ..
        } => {
            return Some(Event::Mouse(crate::core::event::MouseEvent {
                x,
                y,
                down: mousestate.left(),
                changed: false,
            }))
        }
        sdl2::event::Event::MouseButtonDown {
            mouse_btn: MouseButton::Left,
            x,
            y,
            ..
        } => {
            return Some(Event::Mouse(crate::core::event::MouseEvent {
                x,
                y,
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
            return Some(Event::Mouse(crate::core::event::MouseEvent {
                x,
                y,
                down: false,
                changed: true,
            }))
        }
        sdl2::event::Event::MouseWheel {
            direction,
            x,
            y,
            mouse_x,
            mouse_y,
            ..
        } => {
            let multiplier: i32 = match direction {
                sdl2::mouse::MouseWheelDirection::Flipped => -1,
                _ => 1,
            };
            return Some(Event::MouseWheel(MouseWheelEvent {
                x: mouse_x,
                y: mouse_y,
                wheel_dx: x * multiplier,
                wheel_dy: y * multiplier,
            }));
        }
        _ => {}
    }
    return None;
}

pub struct LoopingSoundHandle<'a> {
    channel: Option<Channel>,
    path: &'a Path,
}

impl<'a> crate::core::LoopingSoundHandle<'a> for LoopingSoundHandle<'a> {
    fn new(path: &'a Path) -> Self {
        Self {
            channel: None,
            path,
        }
    }
}
