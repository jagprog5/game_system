use std::num::NonZeroU32;

/// window size change. indicates the new size
#[derive(Debug, Clone, Copy)]
pub struct Window {
    pub width: NonZeroU32,
    pub height: NonZeroU32,
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

#[derive(Debug, Clone, Copy)]
pub enum Event {
    /// more variants might be added. this is a placeholder!
    Other,
    Quit,
    Window(Window),
    Mouse(MouseEvent),
    MouseWheel(MouseWheelEvent),
    Key(KeyEvent),
}
