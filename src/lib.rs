pub mod backends;

use std::num::NonZeroU16;
use std::num::NonZeroU32;
use std::path::Path;
use std::time::Duration;

use sdl2::pixels::Color;
use typed_floats::{NonNaNFinite, StrictlyPositiveFinite};

pub trait LoopingSoundHandle<'a>: Sized {
    fn new(path: &'a Path) -> Self;
}

/// use the system data and expose functionality
pub trait System<'font_data>: Sized {
    type LoopingSoundHandle<'a>: crate::LoopingSoundHandle<'a>;
    type Texture<'system>: crate::Texture<'system>
    where
        Self: 'system;
    type TextureOwned<'system>: crate::Texture<'system>
    where
        Self: 'system;

    /// init everything and create one full screen vsync window with a
    /// resolution matching the screen resolution
    ///
    /// provide font file data. it will be used for text rendering operations
    fn new(font_file_data: &'font_data [u8]) -> Result<Self, String>;

    /// the size of the window canvas, width height
    fn size(&self) -> Result<(NonZeroU32, NonZeroU32), String>;

    /// set the screen to the provided color, clearing all drawing
    fn clear(&mut self, color: Color) -> Result<(), String>;

    /// make the drawing appear to the screen
    fn present(&mut self) -> Result<(), String>;

    /// makes drawing only appear within a specified region
    fn clip(&mut self, c: ClippingRect);

    fn get_clip(&mut self) -> ClippingRect;

    /// load texture from file or reuse from (unspecified) cache. the texture
    /// instance can then be used to draw to the screen
    fn texture(&mut self, image_path: &Path) -> Result<Self::Texture<'_>, String>;

    /// render text or reuse from (unspecified) cache. the texture instance can
    /// then be used to draw to the screen
    ///
    /// there is no guarantee that the provided point size will be the one that
    /// is used to render the font - the output texture size is unspecified and
    /// should be scaled appropriately
    /// 
    /// calls to this function MUST not have arguments which constantly change.
    /// for example, a frame count text is guaranteed to change every frame and
    /// should not be used here. if arguments do constantly change each frame,
    /// then dynamic_text() must be used instead
    fn static_text(
        &mut self,
        text: NonEmptyStr,
        point_size: NonZeroU16,
        wrap_width: Option<NonZeroU32>,
    ) -> Result<Self::Texture<'_>, String>;

    /// see static_text for details. unlike static_text, the cache is not used
    fn dynamic_text(
        &mut self,
        text: NonEmptyStr,
        point_size: NonZeroU16,
        wrap_width: Option<NonZeroU32>,
    ) -> Result<Self::TextureOwned<'_>, String>;

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
    fn sound(&mut self, sound: &Path, direction: f32, distance: f32) -> Result<(), String>;

    /// non blocking
    ///
    /// load sound from file or reuse from cache and play it looping forever.
    ///
    /// the handle is meant to be managed by the single entity that is producing
    /// the sound - calls from that entity must use the same mutable handle
    /// reference and calling this will adjust the looping sound if it is
    /// playing
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
    /// loop_sound after being stopped, it will start up and reference a new
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
    fn music(
        &mut self,
        music: &Path,
        fade_out_duration: Option<Duration>,
        fade_in_duration: Option<Duration>,
    ) -> Result<(), String>;

    /// non blocking
    fn stop_music(&mut self, fade_out_duration: Option<Duration>) -> Result<(), String>;

    /// from 0 to 1 inclusively
    fn set_music_volume(&mut self, volume: f32);

    /// from 0 to 1 inclusively
    fn get_music_volume(&self) -> f32;

    /// receive input from the user. wait forever until that happens
    fn event(&mut self) -> Event;

    /// receive input from the user. wait a max amount of time to wait in
    /// milliseconds (tending to round down)
    fn event_timeout(&mut self, timeout: Duration) -> Option<Event>;
}

// =============================================================================

#[derive(Debug, Copy, Clone)]
pub struct TextureArea {
    pub x: i32,
    pub y: i32,
    pub w: NonZeroU32,
    pub h: NonZeroU32,
}

#[derive(Debug, Copy, Clone)]
pub struct TextureAreaF {
    pub x: NonNaNFinite<f32>,
    pub y: NonNaNFinite<f32>,
    pub w: StrictlyPositiveFinite<f32>,
    pub h: StrictlyPositiveFinite<f32>,
}

#[derive(Debug, Copy, Clone)]
pub struct TextureRotation {
    /// degrees clockwise
    pub angle: NonNaNFinite<f32>,
    /// point in destination (x, y) in which the rotation is about. center if
    /// not stated
    pub point: Option<(i32, i32)>,
    pub flip_horizontal: bool,
    pub flip_vertical: bool,
}

#[derive(Debug, Copy, Clone)]
pub struct TextureRotationF {
    /// degrees clockwise
    pub angle: NonNaNFinite<f32>,
    /// point in destination (x, y) in which the rotation is about. center if
    /// not stated
    pub point: Option<(NonNaNFinite<f32>, NonNaNFinite<f32>)>,
    pub flip_horizontal: bool,
    pub flip_vertical: bool,
}

/// arg for render copy operations. where and how should the texture be placed
pub enum TextureDestination {
    Int(TextureArea, Option<TextureRotation>, Color),
    Float(TextureAreaF, Option<TextureRotationF>, Color),
}

impl From<TextureArea> for TextureDestination {
    fn from(area: TextureArea) -> Self {
        TextureDestination::Int(area, None, Color::WHITE)
    }
}

impl From<TextureAreaF> for TextureDestination {
    fn from(area: TextureAreaF) -> Self {
        TextureDestination::Float(area, None, Color::WHITE)
    }
}

/// exposes ability to draw texture onto the screen
pub trait Texture<'system>: Sized {
    /// copy a part of the texture onto the screen. applies alpha blending.
    /// applies nearest sampling
    fn copy<Dst>(&mut self, src: TextureArea, dst: Dst) -> Result<(), String>
    where
        Dst: Into<TextureDestination>;

    /// get the size of this texture; width, height.
    fn size(&self) -> Result<(NonZeroU32, NonZeroU32), String>;
}

// =============================================================================

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct NonEmptyStr<'a>(&'a str);

impl<'a> TryInto<NonEmptyStr<'a>> for &'a str {
    type Error = &'static str;

    fn try_into(self) -> Result<NonEmptyStr<'a>, Self::Error> {
        if self.is_empty() {
            Err("empty str")
        } else {
            Ok(NonEmptyStr(self))
        }
    }
}

// =============================================================================

#[derive(Clone, Copy, Debug)]
pub enum ClippingRect {
    /// a non-zero area clipping rect
    Some(TextureArea),
    /// a clipping rect with zero area
    Zero,
    /// the absence of a clipping rect
    None,
}

impl ClippingRect {
    pub fn intersection(&self, other: ClippingRect) -> ClippingRect {
        match self {
            ClippingRect::Zero => ClippingRect::Zero,
            ClippingRect::None => other,
            ClippingRect::Some(self_rect) => match other {
                ClippingRect::Zero => ClippingRect::Zero,
                ClippingRect::None => *self,
                ClippingRect::Some(rect) => {
                    let x1 = self_rect.x.max(rect.x);
                    let y1 = self_rect.y.max(rect.y);
                    let x2 =
                        (self_rect.x + self_rect.w.get() as i32).min(rect.x + rect.w.get() as i32);
                    let y2 =
                        (self_rect.y + self_rect.h.get() as i32).min(rect.y + rect.h.get() as i32);
                    if x2 > x1 && y2 > y1 {
                        ClippingRect::Some(TextureArea {
                            x: x1,
                            y: y1,
                            w: NonZeroU32::new((x2 - x1) as u32).unwrap(),
                            h: NonZeroU32::new((y2 - y1) as u32).unwrap(),
                        })
                    } else {
                        ClippingRect::Zero
                    }
                }
            },
        }
    }
}

// =============================================================================

/// window size change. indicates the new size
#[derive(Debug, Clone, Copy)]
pub struct Window {
    pub width: NonZeroU32,
    pub height: NonZeroU32,
}

/// state of the primary (left) mouse button
#[derive(Debug, Clone, Copy)]
pub struct MouseEvent {
    pub x: u32,
    pub y: u32,
    pub down: bool,
    /// is the state of down different from what it was immediately before this
    pub changed: bool,
}

#[derive(Debug, Clone, Copy)]
pub struct KeyEvent {
    /// the key that was typed, accounting for keyboard layout
    pub key: u8,
    /// indicates if this key is up or down
    pub down: bool,
}

#[derive(Debug, Clone, Copy)]
pub enum Event {
    Quit,
    Window(Window),
    Mouse(MouseEvent),
    Key(KeyEvent),
}
