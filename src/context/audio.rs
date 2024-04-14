//! Zero-cost abstraction types for building more complicated audio playbacks.

use kira::sound::Region;

use crate::{assets::Audio, Context};

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
    pub fn with_volume(mut self, volume: f32) -> Self {
        self.volume = Some(volume);

        self
    }

    /// Set the panning of the sound.
    ///
    /// # Arguments
    ///
    /// * `panning` - Which of the stereo speakers to use, `0.0` is hard left, `1.0` is hard right and `0.5` is both equally (default).
    #[inline(always)]
    pub fn pan(mut self, panning: f32) -> Self {
        self.panning = Some(panning);

        self
    }

    /// Loop the whole song.
    ///
    /// This is equivalent to [`Self::with_loop_region(..)`].
    #[inline(always)]
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
            let sound_data = ctx
                .asset::<Audio>(self.path)
                .0
                .with_modified_settings(|settings| {
                    // Set the volume
                    let settings = if let Some(volume) = self.volume {
                        settings.volume(volume as f64)
                    } else {
                        settings
                    };

                    // Set the panning
                    let settings = if let Some(panning) = self.panning {
                        settings.panning(panning as f64)
                    } else {
                        settings
                    };

                    // Set the loop region
                    let settings = if let Some(loop_region) = self.loop_region {
                        settings.loop_region(loop_region)
                    } else {
                        settings
                    };

                    // Set the playback region
                    if let Some(playback_region) = self.playback_region {
                        settings.playback_region(playback_region)
                    } else {
                        settings
                    }
                });

            ctx.audio_manager
                .play(sound_data)
                .expect("Error playing audio")
        });
    }
}
