pub mod backends;
pub mod clipping_rect;
pub mod color;
pub mod event;
pub mod texture_rect;

use std::num::NonZeroU16;
use std::num::NonZeroU32;
use std::path::Path;
use std::path::PathBuf;
use std::time::Duration;

use clipping_rect::ClippingRect;
use color::Color;
use color::Surface;
use event::Event;
use texture_rect::TextureDestination;
use texture_rect::TextureDestinationF;
use texture_rect::TextureRect;
use texture_rect::TextureSource;
use texture_rect::TextureSourceF;

pub trait LoopingSoundHandle<'a>: Sized {
    fn new(path: &'a Path) -> Self;
}

pub trait TextureHandle<'system>: Sized {
    /// copy texture to window. applies alpha blending
    fn copy<Src, Dst>(&mut self, src: Src, dst: Dst) -> Result<(), String>
    where
        Src: Into<TextureSource>,
        Dst: Into<TextureDestination>;

    /// copy texture to window. applies alpha blending
    fn copy_f<Src, Dst>(&mut self, src: Src, dst: Dst) -> Result<(), String>
    where
        Src: Into<TextureSourceF>,
        Dst: Into<TextureDestinationF>;

    /// get the size of this texture; width, height.
    fn size(&self) -> Result<(NonZeroU32, NonZeroU32), String>;

    /// returns a row major array of the pixels in the texture
    ///
    /// warning! this operation may be slow
    fn pixels<Src>(&mut self, src: Src) -> Result<Surface, String>
    where
        Src: Into<TextureSource>;
}

pub trait System: Sized {
    type LoopingSoundHandle<'a>: crate::core::LoopingSoundHandle<'a>;
    /// applies nearest neighbor sampling
    type ImageTextureHandle<'system>: crate::core::TextureHandle<'system>
    where
        Self: 'system;
    /// should apply scaling based on setting passed to System::new()
    type TextTextureHandle<'system>: crate::core::TextureHandle<'system>
    where
        Self: 'system;

    /// initialize everything for the game
    ///
    /// if size is none, creates a full screen vsync window resolution matching
    /// the screen
    ///
    /// if size is Some(...), creates a resizable window with title and size
    ///
    /// provide font file data. it will be used for text rendering operations.
    /// it can ref an empty array if no text rendering will occur
    ///
    /// set if font texture interpolation should occurs - the correct setting
    /// for this is situational. if a blocky font is used, interpolation should
    /// be set to false (it will apply nearest neighbor sampling to retain
    /// blocky-ness). but smooth looking fonts should have interpolation set to
    /// true (it will apply some unspecified interpolation or smoothing)
    fn new(
        size: Option<(&str, NonZeroU32, NonZeroU32)>,
        font_file_data: &'static [u8],
        font_texture_interpolate: bool,
    ) -> Result<Self, String>;

    /// see new()
    fn recreate_window(
        &mut self,
        size: Option<(&str, NonZeroU32, NonZeroU32)>,
    ) -> Result<(), String>;

    /// image paths will be relative to this base path
    fn texture_path_base(&mut self, base: &Path);

    fn get_texture_path_base(&self) -> &Path;

    /// both sound and music paths will be relative to this base path
    fn audio_path_base(&mut self, base: &Path);

    fn get_audio_path_base(&self) -> &Path;

    /// the size of the window canvas, width height
    fn size(&self) -> Result<(NonZeroU32, NonZeroU32), String>;

    /// set to the provided color, clearing all drawing
    ///
    /// this ignores the clipping rectangle
    fn clear(&mut self, color: Color) -> Result<(), String>;

    /// make the content appear on the window
    fn present(&mut self) -> Result<(), String>;

    /// makes drawing only appear within a specified region
    fn clip(&mut self, c: ClippingRect);

    fn get_clip(&mut self) -> ClippingRect;

    /// load texture from file or reuse from (unspecified) cache
    fn image<'a, P>(&mut self, image_path: P) -> Result<Self::ImageTextureHandle<'_>, String>
    where
        P: Into<PathLike<'a>>;

    /// render text or reuse from (unspecified) cache
    ///
    /// there is no guarantee that the provided point size will be the one that
    /// is used to render the font - the output texture size is unspecified and
    /// should be scaled appropriately.
    ///
    /// text should be discretized - if it's possible for a large number of
    /// different tuple(text, color, wrap_width) keys to exist, then this will
    /// not work well with the cache
    fn text(
        &mut self,
        text: NonEmptyStr,
        color: Color,
        point_size: NonZeroU16,
        wrap_width: Option<NonZeroU32>,
    ) -> Result<Self::TextTextureHandle<'_>, String>;

    /// software render texture or reuse from (unspecified) cache. parallelism
    /// is recommended, and the output texture size should be small
    ///
    /// the provided key is user defined and should uniquely and
    /// deterministically include all information about how the texture is
    /// generated - bijective mapping
    ///
    /// textures should be discretized - if it's possible for a large number of
    /// different texture keys to be used at the same time, then this will not
    /// work well with the cache
    fn pixels<'a, K, G>(
        &mut self,
        key: K,
        generation_function: G,
    ) -> Result<Self::ImageTextureHandle<'_>, String>
    where
        K: Into<BytesLike<'a>>,
        G: Fn(&mut Self) -> Result<Surface, String>;

    /// non blocking
    ///
    /// load sound from file or reuse from cache and play it. the backend may
    /// choose to silently do nothing, for example if too many sounds are
    /// playing concurrently
    ///
    /// direction: from 0 to 1. 0 is north, increasing rotates clockwise. 1
    /// wraps back to north
    ///
    /// distance from 0 to 1 inclusively. a distance of 0 has full volume. a
    /// distance of 1 will be very quiet but may not be silent
    fn sound<'a, 's, P>(
        &'s mut self,
        sound: P,
        direction: f32,
        distance: f32,
    ) -> Result<(), String>
    where
        P: Into<PathLike<'a>>,
        's: 'a;

    /// non blocking
    ///
    /// load sound from file or reuse from cache and play it looping forever.
    ///
    /// the handle is meant to be managed by the single entity that is producing
    /// the sound - calls from that entity must use the same mutable handle
    /// reference and calling this will adjust the looping sound if it is
    /// playing. this function should be called each frame by that entity!
    ///
    /// fade_in_duration, if set, will only be applied if this looping sound
    /// just started playing
    ///
    /// direction and distance is described in sound()
    fn loop_sound<'a>(
        &mut self,
        handle: &mut Self::LoopingSoundHandle<'a>,
        direction: f32,
        distance: f32,
        fade_in_duration: Option<Duration>,
    ) -> Result<(), String>;

    /// non blocking
    ///
    /// fades out the looping sound and stops it  
    /// this resets the handle's internal state so that if it is used in
    /// loop_sound() after being stopped, it will start up and reference a new
    /// looping sound
    fn stop_loop_sound<'a>(
        &mut self,
        handle: &mut Self::LoopingSoundHandle<'a>,
        fade_out_duration: Option<Duration>,
    );

    /// non blocking
    ///
    /// play music looping forever
    ///
    /// if music is currently playing, fades it out. the fade out duration is
    /// used to stop the currently playing track, not the next one that will be
    /// playing from this call
    fn music<'a, 's, P>(
        &mut self,
        music: P,
        fade_out_duration: Option<Duration>,
        fade_in_duration: Option<Duration>,
    ) -> Result<(), String>
    where
        P: Into<PathLike<'a>>,
        's: 'a;

    /// non blocking
    fn stop_music(&mut self, fade_out_duration: Option<Duration>) -> Result<(), String>;

    /// from 0 to 1 inclusively
    fn set_music_volume(&mut self, volume: f32);

    /// from 0 to 1 inclusively
    fn music_volume(&self) -> f32;

    /// receive input from the user. wait forever until that happens
    fn event(&mut self) -> Event;

    /// receive input from the user. wait a max amount of time to wait in
    /// milliseconds
    fn event_timeout(&mut self, timeout: Duration) -> Option<Event>;
}

// =============================================================================

pub enum PathLike<'a> {
    Path(&'a Path),
    Buf(PathBuf),
    Parts(&'a [&'static str]),
}

impl<'a> PathLike<'a> {
    pub fn get_path(self, maybe_buf: &'a mut Option<PathBuf>) -> &'a Path {
        match self {
            PathLike::Path(path) => path,
            PathLike::Buf(buf) => {
                *maybe_buf = Some(buf);
                maybe_buf.as_ref().unwrap()
            }
            PathLike::Parts(parts) => {
                let buf: PathBuf = parts.iter().collect();
                *maybe_buf = Some(buf);
                maybe_buf.as_ref().unwrap()
            }
        }
    }
}

impl<'a> From<PathBuf> for PathLike<'a> {
    fn from(p: PathBuf) -> Self {
        PathLike::Buf(p)
    }
}

impl<'a> From<&'a PathBuf> for PathLike<'a> {
    fn from(p: &'a PathBuf) -> Self {
        PathLike::Path(p)
    }
}

impl<'a> From<&'a Path> for PathLike<'a> {
    fn from(p: &'a Path) -> Self {
        PathLike::Path(p)
    }
}

impl<'a> From<&'a [&'static str]> for PathLike<'a> {
    fn from(parts: &'a [&'static str]) -> Self {
        PathLike::Parts(parts)
    }
}

impl<'a> From<PathLike<'a>> for PathBuf {
    fn from(ps: PathLike<'a>) -> Self {
        match ps {
            PathLike::Path(p) => p.to_path_buf(),
            PathLike::Buf(buf) => buf,
            PathLike::Parts(parts) => parts.iter().collect(),
        }
    }
}

// =============================================================================

pub enum BytesLike<'a> {
    Slice(&'a [u8]),
    Vec(Vec<u8>),
}

impl<'a> From<Vec<u8>> for BytesLike<'a> {
    fn from(v: Vec<u8>) -> Self {
        BytesLike::Vec(v)
    }
}

impl<'a> From<&'a Vec<u8>> for BytesLike<'a> {
    fn from(v: &'a Vec<u8>) -> Self {
        BytesLike::Slice(v)
    }
}

impl<'a> From<&'a [u8]> for BytesLike<'a> {
    fn from(v: &'a [u8]) -> Self {
        BytesLike::Slice(v)
    }
}

impl<'a> From<BytesLike<'a>> for Vec<u8> {
    fn from(b: BytesLike<'a>) -> Self {
        match b {
            BytesLike::Slice(s) => s.to_vec(),
            BytesLike::Vec(v) => v,
        }
    }
}

// =============================================================================

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct NonEmptyStr<'a>(&'a str);

impl<'a> TryInto<NonEmptyStr<'a>> for &'a str {
    type Error = ();

    fn try_into(self) -> Result<NonEmptyStr<'a>, Self::Error> {
        if self.is_empty() {
            Err(())
        } else {
            Ok(NonEmptyStr(self))
        }
    }
}
