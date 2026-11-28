//! Sound (docs/design/audio.md).
//!
//! - `cues`: the sounds, in two soundtracks rendered at build time by
//!   Sonomorph (`build.rs`): the orchestral one from `audio/score/` by its
//!   studio (music and stingers as QOA stereo, effects as mu-law), the
//!   8-bit one from `audio/cues/` by its chip8 engine (all mu-law);
//! - `director`: which sound plays when, where and at what pitch: pure,
//!   tested.
//!
//! The mixer (effects, stingers and streamed music into one stereo stream)
//! and the device stream that runs it are Sonomorph's
//! (`sonomorph::advanced`), the crate BRYX's sound engines were extracted
//! to: BRYX names its sounds (`Cue`) and decides when they play, and
//! [`Audio`] plays each in the soundtrack the player picked (T).
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

pub use director::{SoundMode, Soundtrack};

/// What the director asks Sonomorph's mixer to do, in BRYX's cues.
pub(crate) type Command = sonomorph::advanced::Command<cues::Cue>;

/// What the mixer hears: the director's command with its cue in one
/// soundtrack.
#[cfg_attr(not(feature = "audio"), allow(dead_code))]
type Rendered = sonomorph::advanced::Command<cues::Rendition>;

/// The director's `command` in `soundtrack`. Without the `audio` feature
/// there is no mixer to hear it, and only its test calls it.
#[cfg_attr(not(feature = "audio"), allow(dead_code))]
fn rendered(command: Command, soundtrack: Soundtrack) -> Rendered {
    use sonomorph::advanced::Command as C;
    let r = |cue| cues::Rendition { soundtrack, cue };
    match command {
        C::Play {
            sound,
            gain,
            pan,
            pitch,
            duck,
        } => C::Play {
            sound: r(sound),
            gain,
            pan,
            pitch,
            duck,
        },
        C::Music(track) => C::Music(track.map(r)),
        C::MusicLevel(level) => C::MusicLevel(level),
        C::Enable { effects, music } => C::Enable { effects, music },
        C::CutHeld => C::CutHeld,
    }
}

use crate::events::GameEvent;
use crate::game::Game;

pub struct Audio {
    director: director::Director,
    mode: SoundMode,
    soundtrack: Soundtrack,
    #[cfg(feature = "audio")]
    output: Option<sonomorph::advanced::Output<cues::Rendition>>,
}

impl Audio {
    /// Opens the audio device. Without one, or without the `audio`
    /// feature, every call is a no-op and the game plays silent.
    pub fn new(mode: SoundMode, soundtrack: Soundtrack) -> Self {
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
                output.send(rendered(mode.command(), soundtrack));
                Some(output)
            }
            Err(err) => {
                eprintln!("audio: {err}; playing without sound");
                None
            }
        };
        // Seeds the levelset playlist: a fresh order each session.
        let mut director = director::Director::new(rand::random());
        director.soundtrack = soundtrack;
        Self {
            director,
            mode,
            soundtrack,
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
                output.send(rendered(command, self.soundtrack));
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
            output.send(rendered(self.mode.command(), self.soundtrack));
        }
    }

    /// T: the orchestral soundtrack, the 8-bit one, and back. The music
    /// playing crossfades to its other rendition; sounds already playing
    /// finish in the soundtrack they started in.
    pub fn cycle_soundtrack(&mut self) {
        self.soundtrack = self.soundtrack.next();
        self.director.soundtrack = self.soundtrack;
        #[cfg(feature = "audio")]
        if let Some(output) = &self.output {
            if let Some(cue) = self.director.music() {
                output.send(rendered(Command::Music(Some(cue)), self.soundtrack));
            }
        }
    }

    /// What the HUD shows for the T key; `None` when this build or this
    /// machine has no sound to set.
    pub fn soundtrack_label(&self) -> Option<&'static str> {
        #[cfg(feature = "audio")]
        if self.output.is_some() {
            return Some(self.soundtrack.label());
        }
        None
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

#[cfg(test)]
mod tests {
    use super::*;
    use cues::{Cue, Rendition};
    use sonomorph::advanced::Command as C;

    /// The director speaks in cues; the mixer hears each in the soundtrack
    /// picked, and every other command as it was.
    #[test]
    fn a_command_plays_its_cue_in_the_soundtrack_picked() {
        let in_8_bit = |cue| Rendition {
            soundtrack: Soundtrack::EightBit,
            cue,
        };
        let play = C::Play {
            sound: Cue::Brick,
            gain: 0.6,
            pan: -0.5,
            pitch: 1.5,
            duck: 0.0,
        };
        assert_eq!(
            rendered(play, Soundtrack::EightBit),
            C::Play {
                sound: in_8_bit(Cue::Brick),
                gain: 0.6,
                pan: -0.5,
                pitch: 1.5,
                duck: 0.0,
            }
        );
        assert_eq!(
            rendered(C::Music(Some(Cue::Level(7))), Soundtrack::EightBit),
            C::Music(Some(in_8_bit(Cue::Level(7))))
        );
        assert_eq!(
            rendered(C::Music(Some(Cue::MusicMenu)), Soundtrack::Orchestral),
            C::Music(Some(Rendition {
                soundtrack: Soundtrack::Orchestral,
                cue: Cue::MusicMenu,
            }))
        );
        assert_eq!(
            rendered(C::Music(None), Soundtrack::EightBit),
            C::Music(None)
        );
        assert_eq!(
            rendered(C::MusicLevel(0.4), Soundtrack::EightBit),
            C::MusicLevel(0.4)
        );
        assert_eq!(
            rendered(
                C::Enable {
                    effects: true,
                    music: false
                },
                Soundtrack::Orchestral
            ),
            C::Enable {
                effects: true,
                music: false
            }
        );
        assert_eq!(rendered(C::CutHeld, Soundtrack::EightBit), C::CutHeld);
    }
}
