pub(crate) struct TextureWrapper(pub sdl2::render::Texture);

impl Drop for TextureWrapper {
    // safe destroy(), since these Textures will be dropped before the parent
    // canvas + creator is dropped
    fn drop(&mut self) {
        unsafe { sdl2::sys::SDL_DestroyTexture(self.0.raw()) }
    }
}
