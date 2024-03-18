//! Play sounds and music files.
//!
//! When the `audio` feature is enabled a global audio context will be started when a new window is spawned.

use std::{
    borrow::Cow,
    io::Cursor,
    sync::{Arc, Mutex, OnceLock},
};

use assets_manager::{loader::Loader, Asset, BoxedError};
use kira::{
    manager::{backend::DefaultBackend, AudioManager, AudioManagerSettings},
    sound::static_sound::{StaticSoundData, StaticSoundSettings},
};
use miette::{Context, IntoDiagnostic, Result};

/// Globally accessible audio manager for playing audio the device the game runs on.
pub(crate) static AUDIO_MANAGER: OnceLock<Arc<Mutex<AudioManager<DefaultBackend>>>> =
    OnceLock::new();

/// Audio asset for playing sounds and music.
#[derive(Debug)]
pub struct Audio(StaticSoundData);

impl Audio {
    /// Play the audio from start to end.
    ///
    /// # Panics
    ///
    /// - When the audio manager is not initialized yet, this is done by spawning the game window.
    /// - When the sound could not be played on the manager.
    /// - When a mutex lock could not be acquired (should not happen).
    pub fn play(&self) {
        // Get global the manager
        let manager_ref = AUDIO_MANAGER.get().expect("Audio is not initialized yet, did you try to play the sound before the window is spawned?").clone();
        let mut manager = manager_ref.lock().expect("Could not lock audio manager");

        // Play the sound on the global manager
        // Cloning the sound here is fine because the bytes of the static date are reference counted
        manager.play(self.0.clone()).expect("Error playing sound");
    }
}

impl Asset for Audio {
    // We only support OGG files currently
    const EXTENSION: &'static str = "ogg";

    type Loader = AudioLoader;
}

/// Audio asset loader.
pub struct AudioLoader;

impl Loader<Audio> for AudioLoader {
    fn load(content: Cow<[u8]>, _ext: &str) -> Result<Audio, BoxedError> {
        // Allocate the bytes into a cursor
        let bytes = Cursor::new(content.into_owned());

        // Parse the sound file
        let sound_data = StaticSoundData::from_cursor(bytes, StaticSoundSettings::new())?;

        Ok(Audio(sound_data))
    }
}

/// Start the audio backend.
///
/// # Errors
///
/// - When the audio manager could not find a device to play audio on.
pub(crate) fn init_audio() -> Result<()> {
    // Start the audio manager
    let manager = AudioManager::new(AudioManagerSettings::default())
        .into_diagnostic()
        .wrap_err("Error setting up audio manager")?;

    // Store inside the global
    AUDIO_MANAGER
        .set(Arc::new(Mutex::new(manager)))
        .map_err(|_| miette::miette!("Error setting up already initialized audio manager"))?;

    Ok(())
}
