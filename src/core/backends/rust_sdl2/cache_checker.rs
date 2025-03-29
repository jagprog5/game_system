pub struct CacheMissChecker {
    last_n_frames_had_cache_misses: u32,
    this_frame_had_cache_misses: bool,
}

impl Default for CacheMissChecker {
    fn default() -> Self {
        Self {
            last_n_frames_had_cache_misses: Default::default(),
            this_frame_had_cache_misses: Default::default(),
        }
    }
}

impl CacheMissChecker {
    pub fn cache_miss_occurred(&mut self) {
        self.this_frame_had_cache_misses = true;
    }

    pub fn reset(&mut self) {
        self.last_n_frames_had_cache_misses = 0;
        self.this_frame_had_cache_misses = false;
    }

    /// call at the end of the frame
    pub fn frame_end(&mut self) -> u32 {
        if self.this_frame_had_cache_misses {
            self.last_n_frames_had_cache_misses += 1;
        } else {
            self.last_n_frames_had_cache_misses = 0;
        }
        self.this_frame_had_cache_misses = false;
        self.last_n_frames_had_cache_misses
    }
}
