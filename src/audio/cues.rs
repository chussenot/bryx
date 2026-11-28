//! The cue bank: every sound BRYX can make, in both of its soundtracks, as
//! `build.rs` rendered them with Sonomorph. The bytes stay where the binary
//! put them, and Sonomorph's mixer reads them:
//! - the orchestral soundtrack, from `audio/score/` by the studio
//!   (`sonomorph::advanced`): the music and the stingers are QOA, 48 kHz
//!   stereo ([`studio`]), decoded a slice at a time as they play; the
//!   effects are 8-bit mu-law mono at [`CUE_SAMPLE_RATE`] ([`pcm`]), read
//!   one sample at a time at the playback rate their pitch asks for;
//! - the 8-bit soundtrack, from `audio/cues/` by the chip8 engine
//!   (`sonomorph::chip8`): every cue is mu-law mono at [`CUE_SAMPLE_RATE`]
//!   ([`chip`]).
//!
//! A [`Rendition`] is a cue in one soundtrack: what the mixer plays. So a
//! sound costs no memory beyond the binary itself.

use sonomorph::advanced::{Clip, Sound};

use super::director::Soundtrack;
#[cfg(all(test, feature = "audio"))]
use sonomorph::chip8::mulaw;

/// The rate `build.rs` renders cues at (its `SAMPLE_RATE`).
pub const CUE_SAMPLE_RATE: f32 = 22_050.0;

/// Level tracks in the score (`audio/compose.py`), one per campaign level.
pub const LEVEL_TRACKS: u8 = 32;

/// One sound. Every variant but `Level` names a score under `audio/score/`
/// and an ABC file under `audio/cues/` (`file`); `Level(n)` is `music_NN`,
/// the track for level `n`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Cue {
    // Effects
    Paddle,
    Brick,
    Armor,
    Capsule,
    Tiny,
    OneUp,
    Laser,
    SuperLaser,
    Asteroid,
    Explosion,
    BallLost,
    Launch,
    Warp,
    Select,
    Pause,
    Steel,
    Gate,
    Expiring,
    // Stingers
    LifeLost,
    LevelClear,
    /// An act's last level cleared (`movement.rs`): the level fanfare,
    /// grown by a bar.
    ActClear,
    GameOver,
    Victory,
    // Music loops
    MusicMenu,
    /// The track for level 1..=[`LEVEL_TRACKS`].
    Level(u8),
}

impl Cue {
    /// Every cue that is not a level track.
    #[cfg(test)]
    pub const NAMED: [Cue; 24] = [
        Cue::Paddle,
        Cue::Brick,
        Cue::Armor,
        Cue::Capsule,
        Cue::Tiny,
        Cue::OneUp,
        Cue::Laser,
        Cue::SuperLaser,
        Cue::Asteroid,
        Cue::Explosion,
        Cue::BallLost,
        Cue::Launch,
        Cue::Warp,
        Cue::Select,
        Cue::Pause,
        Cue::Steel,
        Cue::Gate,
        Cue::Expiring,
        Cue::LifeLost,
        Cue::LevelClear,
        Cue::ActClear,
        Cue::GameOver,
        Cue::Victory,
        Cue::MusicMenu,
    ];

    /// The cue's file stem; `None` for a level track (see [`pcm`]).
    pub fn file(self) -> Option<&'static str> {
        Some(match self {
            Cue::Paddle => "paddle",
            Cue::Brick => "brick",
            Cue::Armor => "armor",
            Cue::Capsule => "capsule",
            Cue::Tiny => "tiny",
            Cue::OneUp => "oneup",
            Cue::Laser => "laser",
            Cue::SuperLaser => "superlaser",
            Cue::Asteroid => "asteroid",
            Cue::Explosion => "explosion",
            Cue::BallLost => "ball_lost",
            Cue::Launch => "launch",
            Cue::Warp => "warp",
            Cue::Select => "select",
            Cue::Pause => "pause",
            Cue::Steel => "steel",
            Cue::Gate => "gate",
            Cue::Expiring => "expiring",
            Cue::LifeLost => "life_lost",
            Cue::LevelClear => "level_clear",
            Cue::ActClear => "act_clear",
            Cue::GameOver => "game_over",
            Cue::Victory => "victory",
            Cue::MusicMenu => "music_menu",
            Cue::Level(_) => return None,
        })
    }

    /// Whether new music waits for this cue to finish: the stingers that
    /// end a level or a run, and the sounds that start play (the serve, the
    /// warp). The mixer holds a music change until none of them sounds.
    pub fn holds_music(self) -> bool {
        matches!(
            self,
            Cue::LevelClear
                | Cue::ActClear
                | Cue::GameOver
                | Cue::Victory
                | Cue::Launch
                | Cue::Warp
        )
    }

    /// Music loops play whole, start to end and round again.
    #[cfg(test)]
    pub fn is_music(self) -> bool {
        matches!(self, Cue::MusicMenu | Cue::Level(_))
    }
}

#[cfg(feature = "audio")]
mod rendered {
    include!(concat!(env!("OUT_DIR"), "/cues.rs"));
    include!(concat!(env!("OUT_DIR"), "/scores.rs"));
    include!(concat!(env!("OUT_DIR"), "/chip.rs"));
}

/// `music_01`..`music_32` without formatting a string: the audio thread
/// looks a track up by level, and must not allocate to do it.
const LEVEL_NAMES: [&str; LEVEL_TRACKS as usize] = [
    "music_01", "music_02", "music_03", "music_04", "music_05", "music_06", "music_07", "music_08",
    "music_09", "music_10", "music_11", "music_12", "music_13", "music_14", "music_15", "music_16",
    "music_17", "music_18", "music_19", "music_20", "music_21", "music_22", "music_23", "music_24",
    "music_25", "music_26", "music_27", "music_28", "music_29", "music_30", "music_31", "music_32",
];

impl Cue {
    /// The name a studio score for this cue has (`audio/score/<name>.score`).
    fn score_name(self) -> Option<&'static str> {
        match self {
            Cue::Level(n) => LEVEL_NAMES.get(usize::from(n).wrapping_sub(1)).copied(),
            named => named.file(),
        }
    }
}

/// The studio render for `cue`, QOA stereo (`sonomorph::advanced::qoa`): the music
/// and the stingers. `None` for an effect ([`pcm`]), and always without
/// the `audio` feature.
pub fn studio(cue: Cue) -> Option<&'static [u8]> {
    #[cfg(feature = "audio")]
    {
        let name = cue.score_name()?;
        rendered::STUDIO
            .iter()
            .find(|(n, _)| *n == name)
            .map(|(_, qoa)| *qoa)
    }
    #[cfg(not(feature = "audio"))]
    {
        let _ = cue.score_name();
        None
    }
}

/// An effect's embedded PCM: mu-law mono at [`CUE_SAMPLE_RATE`], trimmed of
/// trailing silence by `build.rs`. Empty for a cue that is not an effect
/// (the music and the stingers are [`studio`] renders), and without the
/// `audio` feature.
pub fn pcm(cue: Cue) -> &'static [u8] {
    #[cfg(feature = "audio")]
    {
        rendered::CUES
            .iter()
            .find(|(name, _)| Some(*name) == cue.file())
            .map(|(_, pcm)| *pcm)
            .unwrap_or(&[])
    }
    #[cfg(not(feature = "audio"))]
    {
        let _ = cue;
        &[]
    }
}

/// A cue's 8-bit render: mu-law mono at [`CUE_SAMPLE_RATE`], from
/// `audio/cues/<name>.abc` (effects and stingers trimmed of trailing
/// silence, music loops whole). Empty for a level with no track, and
/// without the `audio` feature.
pub fn chip(cue: Cue) -> &'static [u8] {
    #[cfg(feature = "audio")]
    {
        let Some(name) = cue.score_name() else {
            return &[];
        };
        rendered::CHIP
            .iter()
            .find(|(n, _)| *n == name)
            .map(|(_, pcm)| *pcm)
            .unwrap_or(&[])
    }
    #[cfg(not(feature = "audio"))]
    {
        let _ = cue.score_name();
        &[]
    }
}

/// A cue as one soundtrack renders it: what the mixer plays. Two
/// renditions of the same cue are different sounds to it, so changing the
/// soundtrack crossfades the music like changing the track.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Rendition {
    pub soundtrack: Soundtrack,
    pub cue: Cue,
}

/// How Sonomorph's mixer plays a rendition: in the orchestral soundtrack,
/// the cue's studio render if it has one (the music and the stingers,
/// QOA), else its effect (mu-law); in the 8-bit one, its chip8 render. A
/// music change waits for the same cues in both.
impl Sound for Rendition {
    fn clip(self) -> Clip {
        match self.soundtrack {
            Soundtrack::Orchestral => {
                studio(self.cue).map_or_else(|| Clip::MuLaw(pcm(self.cue)), Clip::Qoa)
            }
            Soundtrack::EightBit => Clip::MuLaw(chip(self.cue)),
        }
    }

    fn holds_music(self) -> bool {
        self.cue.holds_music()
    }
}

/// The mu-law bytes for full scale, positive and negative: no level track
/// may reach them.
#[cfg(all(test, feature = "audio"))]
const FULL_SCALE: [u8; 2] = [0x80, 0x00];

#[cfg(all(test, feature = "audio"))]
mod tests {
    use super::*;

    /// A cue's studio render, decoded, if it has one.
    #[cfg(feature = "audio")]
    fn studio_frames(cue: Cue) -> Option<Vec<[i16; 2]>> {
        let mut q = sonomorph::advanced::qoa::QoaStream::new(studio(cue)?).unwrap();
        Some(std::iter::from_fn(|| q.next_frame()).collect())
    }

    #[cfg(feature = "audio")]
    fn every_cue() -> impl Iterator<Item = Cue> {
        Cue::NAMED
            .into_iter()
            .chain((1..=LEVEL_TRACKS).map(Cue::Level))
    }

    #[cfg(feature = "audio")]
    #[test]
    fn every_cue_was_rendered_and_is_audible() {
        for cue in every_cue() {
            // A studio render replaces the ABC one, which is then not
            // embedded at all: one render per cue, never both.
            if let Some(frames) = studio_frames(cue) {
                assert!(pcm(cue).is_empty(), "{cue:?} embeds both renders");
                assert!(
                    frames.iter().any(|f| f[0].abs().max(f[1].abs()) > 2_048),
                    "{cue:?} is silent"
                );
                continue;
            }
            let pcm = pcm(cue);
            assert!(!pcm.is_empty(), "{cue:?} has no render");
            assert!(
                pcm.iter().any(|&b| mulaw::sample(b).abs() > 0.0625),
                "{cue:?} is silent"
            );
        }
        assert!((1..=LEVEL_TRACKS).all(|n| studio(Cue::Level(n)).is_some()));
        assert!(studio(Cue::Level(0)).is_none() && studio(Cue::Level(33)).is_none());
    }

    #[cfg(feature = "audio")]
    #[test]
    fn effects_are_trimmed_short_and_music_loops_are_long() {
        for cue in every_cue() {
            if let Some(frames) = studio_frames(cue) {
                let secs = frames.len() as f32 / 48_000.0;
                if cue.is_music() {
                    assert!(secs > 20.0, "{cue:?}: {secs:.1}s");
                } else {
                    assert!(secs < 9.0, "{cue:?}: {secs:.1}s");
                }
                continue;
            }
            let pcm = pcm(cue);
            let secs = pcm.len() as f32 / CUE_SAMPLE_RATE;
            if cue.is_music() {
                assert!(secs > 20.0, "{cue:?}: {secs:.1}s");
            } else {
                assert!(secs < 8.0, "{cue:?}: {secs:.1}s");
                let last = mulaw::sample(*pcm.last().unwrap()).abs();
                assert!(last > 0.5 / 128.0, "{cue:?} not trimmed");
            }
        }
    }

    /// Every studio score is named after a cue (a typo would leave the
    /// ABC render playing, silently) and was written as 48 kHz stereo QOA.
    #[cfg(feature = "audio")]
    #[test]
    fn every_studio_score_replaces_a_cue_and_is_48_khz_stereo_qoa() {
        for (name, bytes) in rendered::STUDIO {
            let cue = every_cue().find(|c| c.score_name() == Some(*name));
            assert!(cue.is_some(), "audio/score/{name}.score names no cue");
            assert_eq!(studio(cue.unwrap()), Some(*bytes));
            let qoa = sonomorph::advanced::qoa::QoaStream::new(bytes)
                .unwrap_or_else(|e| panic!("{name}: {e:?}"));
            assert_eq!((qoa.channels(), qoa.sample_rate()), (2, 48_000), "{name}");
        }
        assert_eq!(studio(Cue::Level(0)), None);
        assert_eq!(studio(Cue::Level(33)), None);
    }

    /// A studio loop starts with the predictor its own end leaves
    /// (Sonomorph's `Render::to_qoa`, `qoa_encode_seamless`): QOA's default cold start
    /// would code the first slice coarsely and click at every wrap.
    #[cfg(feature = "audio")]
    #[test]
    fn every_studio_loop_starts_warm() {
        for cue in every_cue().filter(|c| c.is_music()) {
            let Some(bytes) = studio(cue) else { continue };
            // File header (8), first frame header (8), then per channel
            // the history: four big-endian 16-bit samples.
            for c in 0..2 {
                let at = 16 + c * 16;
                let history = &bytes[at..at + 8];
                assert!(
                    history.iter().any(|&b| b != 0),
                    "{cue:?} channel {c} starts cold"
                );
            }
        }
    }

    /// The score is mastered (`audio/compose.py`): no level track clips,
    /// so no level is ever louder than the limiter expects.
    #[cfg(feature = "audio")]
    #[test]
    fn no_music_track_clips() {
        for n in 1..=LEVEL_TRACKS {
            if let Some(frames) = studio_frames(Cue::Level(n)) {
                // The studio's limiter holds -1 dBFS; QOA may overshoot
                // by its error, never to full scale.
                let hot = frames
                    .iter()
                    .flatten()
                    .filter(|s| s.unsigned_abs() >= 32_700)
                    .count();
                assert_eq!(hot, 0, "level {n} clips");
                continue;
            }
            let pcm = pcm(Cue::Level(n));
            let hot = pcm.iter().filter(|b| FULL_SCALE.contains(b)).count();
            assert_eq!(hot, 0, "level {n} clips");
        }
    }

    /// The 8-bit soundtrack is whole: every cue has a chip8 render, audible,
    /// and nothing under `audio/cues/` is left without a cue to play it.
    #[cfg(feature = "audio")]
    #[test]
    fn every_cue_has_an_audible_8_bit_render() {
        for cue in every_cue() {
            let pcm = chip(cue);
            assert!(!pcm.is_empty(), "{cue:?} has no 8-bit render");
            assert!(
                pcm.iter().any(|&b| mulaw::sample(b).abs() > 0.0625),
                "{cue:?} is silent in 8-bit"
            );
        }
        for (name, bytes) in rendered::CHIP {
            let cue = every_cue().find(|c| c.score_name() == Some(*name));
            assert!(cue.is_some(), "audio/cues/{name}.abc names no cue");
            assert_eq!(chip(cue.unwrap()), *bytes);
        }
        assert_eq!(rendered::CHIP.len(), every_cue().count());
        assert!(chip(Cue::Level(0)).is_empty() && chip(Cue::Level(33)).is_empty());
    }

    #[cfg(feature = "audio")]
    #[test]
    fn eight_bit_effects_are_trimmed_short_and_music_loops_are_long() {
        for cue in every_cue() {
            let pcm = chip(cue);
            let secs = pcm.len() as f32 / CUE_SAMPLE_RATE;
            if cue.is_music() {
                assert!(secs > 20.0, "{cue:?}: {secs:.1}s");
            } else {
                assert!(secs < 8.0, "{cue:?}: {secs:.1}s");
                let last = mulaw::sample(*pcm.last().unwrap()).abs();
                assert!(last > 0.5 / 128.0, "{cue:?} not trimmed");
            }
        }
    }

    /// The 8-bit score is mastered (`audio/compose.py --render`): no level
    /// track clips.
    #[cfg(feature = "audio")]
    #[test]
    fn no_8_bit_music_track_clips() {
        for n in 1..=LEVEL_TRACKS {
            let hot = chip(Cue::Level(n))
                .iter()
                .filter(|b| FULL_SCALE.contains(b))
                .count();
            assert_eq!(hot, 0, "8-bit level {n} clips");
        }
    }

    /// A rendition plays its own soundtrack's render of its cue, never the
    /// other's.
    #[cfg(feature = "audio")]
    #[test]
    fn a_rendition_plays_its_soundtracks_render() {
        let play = |soundtrack, cue| Rendition { soundtrack, cue }.clip();
        for cue in every_cue() {
            match (play(Soundtrack::Orchestral, cue), studio(cue)) {
                (Clip::Qoa(bytes), Some(qoa)) => assert_eq!(bytes, qoa, "{cue:?}"),
                (Clip::MuLaw(bytes), None) => assert_eq!(bytes, pcm(cue), "{cue:?}"),
                _ => panic!("{cue:?}: orchestral plays the other kind of clip"),
            }
            match play(Soundtrack::EightBit, cue) {
                Clip::MuLaw(bytes) => {
                    assert_eq!(bytes, chip(cue), "{cue:?}");
                    assert_ne!(bytes, pcm(cue), "{cue:?} plays the orchestral effect");
                }
                Clip::Qoa(_) => panic!("{cue:?}: 8-bit plays a QOA clip"),
            }
        }
        let held = Rendition {
            soundtrack: Soundtrack::EightBit,
            cue: Cue::Victory,
        };
        assert!(held.holds_music());
        assert_ne!(
            held,
            Rendition {
                soundtrack: Soundtrack::Orchestral,
                cue: Cue::Victory,
            },
            "the same cue in the other soundtrack is another sound"
        );
    }
}
