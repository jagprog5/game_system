# Game System

## Core

"What's the simplest thing that gets the job done?"

This describes an interface which a 2D app can use to interact with the world. It also has a reference implementation based on SDL2. It handles:

 - window creation (full screen only)
 - texture drawing (simple copy + rotation + clipping)
 - audio (sounds + music, both looping or non looping)
 - basic mouse and keyboard input

Textures and audio are loaded and unloaded by the interface - it is managed by a cache. Just specify the path to the resource and don't worry about memory management! 

## UI

A widget based user interface library ported to work the the core system interface.