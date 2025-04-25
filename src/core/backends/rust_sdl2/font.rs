use std::{
    ffi::{CStr, CString},
    num::NonZeroU32,
    os::raw::c_int,
};

use sdl2::{rwops::RWops, surface::Surface, sys::SDL_Color, ttf::Sdl2TtfContext};

/// my own font minimal font wrapper, largely copied from rust-sdl2. was having
/// difficulty with lifetimes, in particular I wanted System to be a self
/// contained (referential) struct
///
/// this is roughly equivalent to the "unsafe_textures" features of rust-sdl2
pub(crate) struct Font {
    raw: *mut sdl2::sys::ttf::TTF_Font,
    #[allow(dead_code)]
    rwops: RWops<'static>,
}

impl Font {
    pub fn new(
        _ttf_context: &Sdl2TtfContext,
        rwops: RWops<'static>,
        point_size: u16,
    ) -> Result<Self, String> {
        unsafe {
            let raw = sdl2::sys::ttf::TTF_OpenFontRW(rwops.raw(), 0, point_size as c_int);
            if raw.is_null() {
                let err = sdl2::sys::SDL_GetError();
                Err(CStr::from_ptr(err).to_str().unwrap().to_owned())
            } else {
                Ok(Font { raw, rwops })
            }
        }
    }

    pub fn render(
        &self,
        text: &str,
        wrap_width: Option<NonZeroU32>,
    ) -> Result<sdl2::surface::Surface, String> {
        unsafe {
            // enforced only for this backend
            let cstr = CString::new(text).map_err(|_| "render text contained null")?;
            let foreground = SDL_Color {
                r: 0xFF,
                g: 0xFF,
                b: 0xFF,
                a: 0xFF,
            };
            let out = match wrap_width {
                Some(wrap_width) => sdl2::sys::ttf::TTF_RenderUTF8_Blended_Wrapped(
                    self.raw,
                    cstr.as_ptr(),
                    foreground,
                    wrap_width.get(),
                ),
                None => sdl2::sys::ttf::TTF_RenderUTF8_Blended(self.raw, cstr.as_ptr(), foreground),
            };

            if out.is_null() {
                let err = sdl2::sys::SDL_GetError();
                Err(CStr::from_ptr(err).to_str().unwrap().to_owned())
            } else {
                Ok(Surface::from_ll(out))
            }
        }
    }
}

impl Drop for Font {
    fn drop(&mut self) {
        unsafe {
            // safety: all fonts are dropped before the ttf context closes
            debug_assert!(sdl2::sys::ttf::TTF_WasInit() != 0);
            sdl2::sys::ttf::TTF_CloseFont(self.raw);
        }
    }
}
