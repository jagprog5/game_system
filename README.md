# Game System

This defines traits for a 2D game framework and gives an implementation based on
[rust-sdl2](https://github.com/Rust-SDL2/rust-sdl2).

## Core

- memory management (textures / audio)
    - specify resource by file path and don't worry about managing anything!
- window creation (single window support only)
- input event handling
    - mouse
    - keyboard
    - window
- textures
    - from image file
    - from rendered font
        - pt size
        - wrap width
        - color
    - src + dst + rotation
    - clipping rectangle (aka scissor)
- audio
    - sounds
        - direction and volume
        - looping support - adjust while playing
    - music
    - fade in / out

## UI

 - only uses the core interface
 - immediate mode
 - optional super low idle CPU usage (only update on events received)
 - widgets
    - tree hierarchy
    - tiled background
    - tiled border
    - button
    - checkbox
    - debug (test sizing)
    - multi line label
    - single line label
    - strut (force spacing)
    - texture widget
 - layout
    - vertical / horizontal
    - scroller
 - sizing information
    - min
    - max
    - preferred portion (e.g. 50%)
    - preferred aspect ratio
    - displacement (on min or max length failure. e.g. "right align text")