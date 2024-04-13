//! Zero-cost abstraction types for building more complicated audio playbacks.

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
}

impl<'path, 'ctx> AudioContext<'path, 'ctx> {
    /// Play the audio from start to end.
    ///
    /// # Panics
    ///
    /// - When asset failed loading.
    /// - When the sound could not be played on the manager.
    #[inline(always)]
    pub fn play(self) {
        self.ctx.write(|ctx| {
            ctx.audio_manager
                .play(ctx.asset::<Audio>(self.path).0.clone())
                .expect("Error playing audio")
        });
    }
}
