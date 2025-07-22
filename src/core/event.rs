/// window size change. indicates the new size
#[derive(Debug, Clone, Copy)]
pub struct Window {
    pub width: u32,
    pub height: u32,
}

/// state of the primary (left) mouse button
#[derive(Debug, Clone, Copy)]
pub struct MouseEvent {
    pub x: i32,
    pub y: i32,
    pub down: bool,
    /// is the state of down different from what it was immediately before this
    pub changed: bool,
}

/// intent is for the system to work on mobile as well, so this might not be
/// available!
#[derive(Debug, Clone, Copy)]
pub struct MouseWheelEvent {
    pub x: i32,
    pub y: i32,
    pub wheel_dx: i32,
    pub wheel_dy: i32,
}

#[derive(Debug, Clone, Copy)]
pub struct KeyEvent {
    /// the key that was typed, accounting for keyboard layout
    pub key: u8,
    /// indicates if this key is up or down
    pub down: bool,
}

pub(crate) fn ascii_more_to_upper(i: u8) -> u8 {
    match i {
        b'a'..=b'z' => i - 32,
        b'1' => b'!',
        b'2' => b'@',
        b'3' => b'#',
        b'4' => b'$',
        b'5' => b'%',
        b'6' => b'^',
        b'7' => b'&',
        b'8' => b'*',
        b'9' => b'(',
        b'0' => b')',

        b'-' => b'_',
        b'=' => b'+',
        b'[' => b'{',
        b']' => b'}',
        b'\\' => b'|',
        b';' => b':',
        b'\'' => b'"',
        b',' => b'<',
        b'.' => b'>',
        b'/' => b'?',
        b'`' => b'~',

        _ => i,
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Event {
    /// more variants might be added. this is a forward compatibility
    /// placeholder!
    Other,
    Quit,
    Window(Window),
    Mouse(MouseEvent),
    MouseWheel(MouseWheelEvent),
    Key(KeyEvent),
}
