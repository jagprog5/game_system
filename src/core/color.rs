#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

#[rustfmt::skip]
impl Color {
    pub const BLACK:  Color =      Color { r: 0,   g: 0,   b: 0,   a: 255 };
    pub const WHITE:  Color =      Color { r: 255, g: 255, b: 255, a: 255 };
    pub const RED:    Color =      Color { r: 255, g: 0,   b: 0,   a: 255 };
    pub const GREEN:  Color =      Color { r: 0,   g: 255, b: 0,   a: 255 };
    pub const BLUE:   Color =      Color { r: 0,   g: 0,   b: 255, a: 255 };
    pub const YELLOW: Color =      Color { r: 255, g: 255, b: 0,   a: 255 };
    pub const CYAN:   Color =      Color { r: 0,   g: 255, b: 255, a: 255 };
    pub const MAGENTA:Color =      Color { r: 255, g: 0,   b: 255, a: 255 };
    pub const TRANSPARENT: Color = Color { r: 0,   g: 0,   b: 0,   a: 0   };
}
