//! Sound (docs/design/audio.md).
//!
//! - `cues`: the sounds, rendered at build time from `audio/score/` by
//!   Sonomorph's studio (`build.rs`): the music and the stingers stored as
//!   QOA stereo, the effects as mu-law;
//! - `director`: which sound plays when, where and at what pitch: pure,
//!   tested.
//!
//! The mixer (effects, stingers and streamed music into one stereo stream)
//! and the device stream that runs it are Sonomorph's
//! (`sonomorph::advanced`), the crate BRYX's sound engine was extracted to:
//! BRYX names its sounds (`Cue`) and decides when they play.
//!
//! Like the renderers, sound only *reads* the game: `main.rs` hands each
//! tick's events over before draining them (events.rs's contract), and
//! nothing here can change what happens.

// Without the `audio` feature there is no device to feed, so the bank and
// the director are only exercised by their tests.
#[cfg_attr(not(feature = "audio"), allow(dead_code))]
mod cues;
#[cfg_attr(not(feature = "audio"), allow(dead_code))]
mod director;

pub use director::SoundMode;

/// What the director asks Sonomorph's mixer to do, in BRYX's cues.
pub(crate) type Command = sonomorph::advanced::Command<cues::Cue>;

use crate::events::GameEvent;
use crate::game::Game;

pub struct Audio {
    director: director::Director,
    mode: SoundMode,
    #[cfg(feature = "audio")]
    output: Option<sonomorph::advanced::Output<cues::Cue>>,
}

impl Audio {
    /// Opens the audio device. Without one, or without the `audio`
    /// feature, every call is a no-op and the game plays silent.
    pub fn new(mode: SoundMode) -> Self {
        // The mixer reads the effects at the rate build.rs rendered them at
        // (a QOA render carries its own).
        #[cfg(feature = "audio")]
        let config = sonomorph::advanced::MixerConfig {
            sample_rate: cues::CUE_SAMPLE_RATE,
            ..Default::default()
        };
        #[cfg(feature = "audio")]
        let output = match sonomorph::advanced::Output::open_with(config) {
            Ok(output) => {
                output.send(mode.command());
                Some(output)
            }
            Err(err) => {
                eprintln!("audio: {err}; playing without sound");
                None
            }
        };
        Self {
            // Seeds the levelset playlist: a fresh order each session.
            director: director::Director::new(rand::random()),
            mode,
            #[cfg(feature = "audio")]
            output,
        }
    }

    /// Hands one simulation tick to the sound director.
    pub fn observe(&mut self, game: &Game, events: &[GameEvent]) {
        let commands = self.director.observe(game, events);
        #[cfg(feature = "audio")]
        if let Some(output) = &self.output {
            for command in commands {
                output.send(command);
            }
        }
        #[cfg(not(feature = "audio"))]
        drop(commands);
    }

    /// The menu's jukebox: the track its Music row picked plays on the
    /// menu in place of the theme (0 the theme, `n` level `n`'s); `None`
    /// gives the menu its theme back.
    pub fn set_jukebox(&mut self, track: Option<u8>) {
        self.director.jukebox = track;
    }

    /// M: all sound, effects only, off, and round again.
    pub fn cycle_mode(&mut self) {
        self.mode = self.mode.next();
        #[cfg(feature = "audio")]
        if let Some(output) = &self.output {
            output.send(self.mode.command());
        }
    }

    /// What the HUD shows for the M key; `None` when this build or this
    /// machine has no sound to set.
    pub fn label(&self) -> Option<&'static str> {
        #[cfg(feature = "audio")]
        if self.output.is_some() {
            return Some(self.mode.label());
        }
        None
    }
}
