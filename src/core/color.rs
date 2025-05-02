use std::num::NonZeroU32;

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

/// Color, but guaranteed to be aligned and packed
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
#[repr(C, packed)]
pub struct ColorPacked {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

#[derive(Clone, Eq, PartialEq, Debug)]
pub struct Surface {
    /// must be appropriate for the size of the data
    pub width: NonZeroU32,
    /// must not be empty
    pub data: Vec<ColorPacked>,
}

impl Surface {
    pub fn as_mut_bytes(&mut self) -> &mut [u8] {
        let byte_len = self.data.len() * std::mem::size_of::<ColorPacked>();
        let byte_slice: &mut [u8] =
            unsafe { std::slice::from_raw_parts_mut(self.data.as_mut_ptr() as *mut u8, byte_len) };
        byte_slice
    }
}

#[rustfmt::skip]
impl Color {
    pub const BLACK:       Color = Color { r: 0,   g: 0,   b: 0,   a: 255 };
    pub const WHITE:       Color = Color { r: 255, g: 255, b: 255, a: 255 };
    pub const RED:         Color = Color { r: 255, g: 0,   b: 0,   a: 255 };
    pub const GREEN:       Color = Color { r: 0,   g: 255, b: 0,   a: 255 };
    pub const BLUE:        Color = Color { r: 0,   g: 0,   b: 255, a: 255 };
    pub const YELLOW:      Color = Color { r: 255, g: 255, b: 0,   a: 255 };
    pub const CYAN:        Color = Color { r: 0,   g: 255, b: 255, a: 255 };
    pub const MAGENTA:     Color = Color { r: 255, g: 0,   b: 255, a: 255 };
    pub const TRANSPARENT: Color = Color { r: 0,   g: 0,   b: 0,   a: 0   };
}
