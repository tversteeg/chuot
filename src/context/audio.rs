//! Zero-cost abstraction types for building more complicated audio playbacks.

use kira::sound::Region;

use crate::Context;

/// Specify how an audio clip should be played.
///
/// Must call [`Self::play`] to play the clip.
///
/// Used by [`crate::Context::audio`].
pub struct AudioContext<'path, 'ctx> {
    /// Path of the sprite to draw.
    pub(crate) path: &'path str,
    /// Reference to the context the sprite will draw in when finished.
    pub(crate) ctx: &'ctx Context,
    /// Volume of the sound.
    pub(crate) volume: Option<f32>,
    /// Panning, left or right.
    pub(crate) panning: Option<f32>,
    /// Which part of the song to loop.
    pub(crate) loop_region: Option<Region>,
    /// Which part of the song to play.
    pub(crate) playback_region: Option<Region>,
}

impl<'path, 'ctx> AudioContext<'path, 'ctx> {
    /// Set the volume of the sound.
    ///
    /// # Arguments
    ///
    /// * `volume` - Volume multiplication factor in the range `0.0..=1.0`.
    #[inline(always)]
    #[must_use]
    pub const fn with_volume(mut self, volume: f32) -> Self {
        self.volume = Some(volume);

        self
    }

    /// Set the panning of the sound.
    ///
    /// # Arguments
    ///
    /// * `panning` - Which of the stereo speakers to use, `0.0` is hard left, `1.0` is hard right and `0.5` is both equally (default).
    #[inline(always)]
    #[must_use]
    pub const fn pan(mut self, panning: f32) -> Self {
        self.panning = Some(panning);

        self
    }

    /// Loop the whole song.
    ///
    /// This is equivalent to [`Self::with_loop_region(..)`].
    #[inline(always)]
    #[must_use]
    pub fn with_loop(mut self) -> Self {
        self.loop_region = Some((..).into());

        self
    }

    /// Set the region of the sound that should be looped.
    ///
    /// # Arguments
    ///
    /// * `loop_region` - Range of seconds that should be looped.
    #[inline(always)]
    #[must_use]
    pub fn with_loop_region(mut self, loop_region: impl Into<Region>) -> Self {
        self.loop_region = Some(loop_region.into());

        self
    }

    /// Set the region of the sound that should be played.
    ///
    /// # Arguments
    ///
    /// * `playback_region` - Range of seconds that should be played.
    #[inline(always)]
    #[must_use]
    pub fn with_playback_region(mut self, playback_region: impl Into<Region>) -> Self {
        self.playback_region = Some(playback_region.into());

        self
    }

    /// Play the audio from start to end.
    ///
    /// # Panics
    ///
    /// - When asset failed loading.
    /// - When the sound could not be played on the manager.
    #[inline(always)]
    pub fn play(self) {
        self.ctx.write(|ctx| {
            // Get the sound data and its settings
            let sound_data = &ctx.assets.audio(self.path).0;
            let mut settings = sound_data.settings.clone();

            // Set the volume
            if let Some(volume) = self.volume {
                settings = settings.volume(volume as f64);
            }

            // Set the panning
            if let Some(panning) = self.panning {
                settings = settings.panning(panning as f64);
            }

            // Set the loop region
            if let Some(loop_region) = self.loop_region {
                settings = settings.loop_region(loop_region);
            }

            // Setup the sound data with the new settings and the playback region
            let mut sound_data = sound_data.with_settings(settings);

            // Set the playback region slice
            if let Some(playback_region) = self.playback_region {
                sound_data = sound_data.slice(playback_region);
            }

            ctx.audio_manager
                .play(sound_data)
                .expect("Error playing audio");
        });
    }
}
