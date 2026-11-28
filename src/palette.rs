//! The one palette both renderers draw with (DESIGN.md, "Colors").
//!
//! Every color is authored here once, as the sRGB hex a designer would
//! write. Each renderer converts it at the point of use:
//! - [`Rgb::linear`] for vertex colors, clear colors and uniforms, because
//!   the shaders write to an sRGB surface and the hardware encodes on
//!   output;
//! - [`Rgb::srgb`] for the procedural sprite recipes in `assets.rs`, whose
//!   bytes go into an `Rgba8UnormSrgb` texture;
//! - [`Rgb::text`] for glyphon, which takes sRGB bytes.
//!
//! Before this module, the same triples were copied into three files and
//! written to the sRGB surface as if they were linear, so `#D94040`
//! rendered as `#ED8989`: a pastel wash over the whole game.
//! `linear_round_trips_through_srgb` pins the conversion.
//!
//! Hue carries meaning in both renderers, and never alone: every capsule
//! also carries its letter ([`capsule_letter`]), and every brick kind its
//! pattern (`assets.rs`). See PRODUCT.md, "Accessibility & Inclusion".

use crate::events::PowerUpKind;
use crate::game::{BallMode, Brick, FallZone, Game, Landing, POWER_WARNING_SECS, TICK_RATE};
use crate::levels::BrickKind;
use crate::structure::{Phase, RULES};

/// An sRGB color as authored: 8 bits per channel, no alpha.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rgb(pub u8, pub u8, pub u8);

const fn hex(v: u32) -> Rgb {
    Rgb((v >> 16) as u8, (v >> 8) as u8, v as u8)
}

/// The sRGB transfer function's decode half, one channel in `0..=1`.
fn decode(c: f32) -> f32 {
    if c <= 0.040_45 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }
}

impl Rgb {
    /// Linear-light RGBA, for anything a shader writes to the sRGB surface.
    pub fn linear(self) -> [f32; 4] {
        self.linear_alpha(1.0)
    }

    /// [`Rgb::linear`] with an explicit alpha (alpha is never encoded).
    pub fn linear_alpha(self, alpha: f32) -> [f32; 4] {
        [
            decode(f32::from(self.0) / 255.0),
            decode(f32::from(self.1) / 255.0),
            decode(f32::from(self.2) / 255.0),
            alpha,
        ]
    }

    /// The authored channels as `0..=1` floats, still sRGB-encoded: the
    /// working space of `assets.rs`'s recipes.
    pub fn srgb(self) -> [f32; 3] {
        [
            f32::from(self.0) / 255.0,
            f32::from(self.1) / 255.0,
            f32::from(self.2) / 255.0,
        ]
    }

    /// A clear color for a render pass (linear, opaque).
    pub fn clear(self) -> wgpu::Color {
        let [r, g, b, _] = self.linear();
        wgpu::Color {
            r: f64::from(r),
            g: f64::from(g),
            b: f64::from(b),
            a: 1.0,
        }
    }

    /// The color as glyphon text takes it.
    pub fn text(self) -> glyphon::Color {
        glyphon::Color::rgb(self.0, self.1, self.2)
    }
}

// -- surfaces -------------------------------------------------------------
// Dark because the game is a field of small, fast, bright objects: the
// ground recedes so they read. Blue-black rather than neutral, to sit under
// warm and cool entity hues alike.

/// Outside the canvas, when the window's aspect differs from the canvas's.
pub const LETTERBOX: Rgb = hex(0x040507);
/// The HUD band above the field.
pub const BAND: Rgb = hex(0x080A0F);
/// The playfield's ground, in both renderers.
pub const FIELD: Rgb = hex(0x10141C);
/// Hairlines: the ceiling rule's minor ticks, keycap outlines.
pub const RULE: Rgb = hex(0x2C3445);
/// The ceiling rule itself and its major ticks.
pub const RULE_STRONG: Rgb = hex(0x56617A);
/// The 3D renderer's side and top walls.
pub const WALL: Rgb = hex(0x2A3140);

/// Each campaign act's one accent (`movement.rs`), in act order:
/// Foundations' chalk ochre, Structures' steel blue, Machines' amber,
/// the Vault's violet. It colors the act's ground motif, the ceiling rule
/// and the 3D walls, never an object the player tracks.
pub const ACT_ACCENTS: [Rgb; 4] = [hex(0xC9A66B), hex(0x6F9FD8), hex(0xE08A3C), hex(0xA08AE8)];

// -- text -----------------------------------------------------------------

/// Values, titles, anything the player reads first. 16:1 on `BAND`.
pub const TEXT: Rgb = hex(0xE9ECF3);
/// Captions and secondary lines. 6.7:1 on `BAND`, 5.9:1 on the scrim.
pub const TEXT_DIM: Rgb = hex(0x8E97AB);
/// Leading zeros of the score: present, but not read.
pub const TEXT_FAINT: Rgb = hex(0x485166);

/// Over the field in Menu/Paused/GameOver/Victory: the field's own ink at
/// this alpha, so the dimmed scene keeps its hue instead of going gray.
pub const SCRIM: Rgb = FIELD;
pub const SCRIM_ALPHA: f32 = 0.95;

// -- the player's objects -------------------------------------------------

/// The ball: the brightest object on screen, warm white so it is not
/// mistaken for any capsule or for the paddle's cool metal.
pub const BALL: Rgb = hex(0xFFF2CF);
/// Every ball's color: its own warm white, or while a ball power lasts,
/// that power's capsule hue.
pub fn ball(mode: Option<BallMode>) -> Rgb {
    match mode {
        None => BALL,
        Some(BallMode::Explosive) => CAPSULE_EXPLOSIVE_BALL,
        Some(BallMode::Metal) => CAPSULE_METAL_BALL,
    }
}
/// How often a power about to run out pulses its hue: under DESIGN.md's
/// 3 Hz limit for anything that flashes.
const EXPIRY_PULSE_HZ: f32 = 2.0;

/// How strongly a timed power's hue shows with `secs_left`: fully until its
/// last [`POWER_WARNING_SECS`], then a 2 Hz pulse (the warning its sound
/// gives, seen), or with reduced motion a steady fade to nothing.
pub fn expiry_strength(secs_left: f32, reduced_motion: bool) -> f32 {
    if secs_left >= POWER_WARNING_SECS {
        1.0
    } else if reduced_motion {
        (secs_left / POWER_WARNING_SECS).clamp(0.0, 1.0)
    } else {
        let t = POWER_WARNING_SECS - secs_left;
        0.5 + 0.5 * (std::f32::consts::TAU * EXPIRY_PULSE_HZ * t).cos()
    }
}

/// The paddle's warning, if a power it carries (Widen, Tiny, Glue, a gun)
/// is about to run out: that power's capsule hue, and how strongly it shows
/// now. The one that runs out soonest when several do.
pub fn paddle_expiry(game: &Game, reduced_motion: bool) -> Option<(Rgb, f32)> {
    game.expiring_powers()
        .filter(|&(kind, _)| !matches!(kind, PowerUpKind::ExplosiveBall | PowerUpKind::MetalBall))
        .min_by(|a, b| a.1.total_cmp(&b.1))
        .map(|(kind, secs)| (capsule(kind), expiry_strength(secs, reduced_motion)))
}

/// How much of its power's hue the ball shows: 1 while a ball power runs,
/// pulsing in its warning (`expiry_strength`).
pub fn ball_power_strength(game: &Game, reduced_motion: bool) -> f32 {
    game.ball_power
        .map_or(1.0, |power| expiry_strength(power.timer, reduced_motion))
}

/// Brushed-metal paddle base (the sprite recipe adds grain and gloss).
pub const PADDLE: Rgb = hex(0xC6CEDC);
/// One-frame flash on a brick that was hit.
pub const HIT_FLASH: Rgb = hex(0xFFFFFF);

// -- bricks: hue plus pattern (assets.rs draws the pattern) ---------------

/// Normal: plain beveled panel.
pub const BRICK_NORMAL: Rgb = hex(0xE0533F);
/// Armored, intact: steel blue, an inset plate with four rivets.
pub const BRICK_ARMORED: Rgb = hex(0x4D7FD6);
/// Armored after its first hit: amber, the same plate split by a crack.
pub const BRICK_ARMORED_HIT: Rgb = hex(0xDE9A2C);
/// Indestructible: graphite with diagonal hatching.
pub const BRICK_INDESTRUCTIBLE: Rgb = hex(0x4A5160);

/// Support, intact: violet, fluted like a column.
pub const BRICK_SUPPORT: Rgb = hex(0x8F6BE0);
/// Support after its first hit: paler and pinker, the flutes cracked. It
/// now carries half as much.
pub const BRICK_SUPPORT_HIT: Rgb = hex(0xC77DD9);
/// Beam and tie: pale steel, thin.
pub const BRICK_BEAM: Rgb = hex(0xAEB9C9);
/// Keystone: gold, with a diamond.
pub const BRICK_KEYSTONE: Rgb = hex(0xF5C542);

/// A brick's color for its kind and remaining hits.
pub fn brick(kind: BrickKind, hits_remaining: u8) -> Rgb {
    match kind {
        BrickKind::Normal => BRICK_NORMAL,
        BrickKind::Armored if hits_remaining >= 2 => BRICK_ARMORED,
        BrickKind::Armored => BRICK_ARMORED_HIT,
        BrickKind::Indestructible => BRICK_INDESTRUCTIBLE,
        BrickKind::Support if hits_remaining >= 2 => BRICK_SUPPORT,
        BrickKind::Support => BRICK_SUPPORT_HIT,
        BrickKind::Beam | BrickKind::Tie => BRICK_BEAM,
        BrickKind::Keystone => BRICK_KEYSTONE,
    }
}

// -- structure: what a brick's state looks like ------------------------------

/// A brick carrying more than it can, cracking: hot orange, the warning
/// before it gives way.
pub const STRUCT_OVERLOAD: Rgb = hex(0xFF7A1A);
/// A brick nothing holds any more, about to fall: white-hot.
pub const STRUCT_UNSTABLE: Rgb = hex(0xFFF1D6);
/// The mark on a brick that carries others: the weak point.
pub const STRUCT_BEARING: Rgb = hex(0xFFD9A0);

/// The paddle while it is held by a falling brick's strike: the overload's
/// orange, strongest at the strike and easing as it comes free. `None`
/// when it is free.
pub fn paddle_stun(game: &Game) -> Option<(Rgb, f32)> {
    (game.paddle_stun > 0.0).then(|| {
        let left = (game.paddle_stun / RULES.stun_secs).clamp(0.0, 1.0);
        (STRUCT_OVERLOAD, 0.45 + 0.4 * left)
    })
}

/// How far ahead a landing mark starts to strengthen: it is faint for
/// anything further off.
const FALL_ZONE_HORIZON_SECS: f32 = 1.2;

/// The mark where a brick that is coming down will land (`Game::fall_zones`),
/// on the paddle's line or on the brick it will hit: its color and how
/// much of it.
/// - Before the keystone falls it is a threat: pale while the brick still
///   shakes in place, the overload's orange once it is in the air.
/// - After, on the paddle's line, it is something to catch: the keystone's
///   gold.
/// - It strengthens as the landing nears, so "now" reads apart from "in a
///   second".
pub fn fall_zone(zone: &FallZone) -> (Rgb, f32) {
    let soon = (1.0 - zone.secs / FALL_ZONE_HORIZON_SECS).clamp(0.0, 1.0);
    match (zone.on, zone.falling) {
        (Landing::Catch, _) => (BRICK_KEYSTONE, 0.45 + 0.5 * soon),
        (_, true) => (STRUCT_OVERLOAD, 0.4 + 0.55 * soon),
        (_, false) => (STRUCT_UNSTABLE, 0.3 + 0.25 * soon),
    }
}

/// A falling brick's overlay: none while it threatens, the keystone's gold
/// once the keystone is down and it is there to be caught.
pub fn debris_glow(game: &Game) -> Option<(Rgb, f32)> {
    game.finale.then_some((BRICK_KEYSTONE, 0.55))
}

/// `a` toward `b` by `t`, 0 to 1, channel by channel in sRGB.
fn mix(a: Rgb, b: Rgb, t: f32) -> Rgb {
    let t = t.clamp(0.0, 1.0);
    let ch = |x: u8, y: u8| (f32::from(x) + (f32::from(y) - f32::from(x)) * t).round() as u8;
    Rgb(ch(a.0, b.0), ch(a.1, b.1), ch(a.2, b.2))
}

/// How a brick's structural state shows: one reading of the simulation,
/// drawn by both renderers, so they say the same thing.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct StructureLook {
    /// Where the brick is drawn from its place: an unstable brick shakes.
    pub offset: [f32; 2],
    /// A color laid over the brick and how much of it, 0 to 1: hot orange
    /// on an overloaded brick, stronger and beating faster as it nears
    /// giving way; white-hot on one about to fall.
    pub glow: Option<(Rgb, f32)>,
    /// The load mark: a bar along the brick's lower edge, as long as the
    /// share of its limit it carries (0: it carries nothing, no bar; 1: at
    /// its limit or past it).
    pub bearing: f32,
    /// The load mark's color: pale up to half its limit, then warming to
    /// the overload's orange as it nears it. What is close to failure
    /// reads before it fails.
    pub bearing_color: Rgb,
}

impl Default for Rgb {
    fn default() -> Self {
        STRUCT_BEARING
    }
}

/// An unstable brick's shake, in px, and how fast.
const UNSTABLE_SHAKE_PX: f32 = 1.5;
const UNSTABLE_SHAKE_HZ: f32 = 11.0;

/// The look of `brick`'s structural state now. With reduced motion nothing
/// shakes or beats: the glow alone says it, steady.
pub fn structure_look(game: &Game, brick: &Brick, reduced_motion: bool) -> StructureLook {
    let state = game.structure.state(brick);
    let share = f32::from(state.load) / f32::from(state.capacity.max(1));
    let bearing = if state.load == 0 {
        0.0
    } else {
        share.clamp(0.25, 1.0)
    };
    let bearing_color = mix(STRUCT_BEARING, STRUCT_OVERLOAD, (share - 0.5) * 2.0);
    match state.phase {
        Phase::Intact | Phase::Stressed => StructureLook {
            bearing,
            bearing_color,
            ..StructureLook::default()
        },
        Phase::Overloaded => {
            // Seconds it has been cracking; its beat goes from 3 to 9 Hz
            // over the time it has.
            let t = state.progress * RULES.give_way_secs;
            let beat = if reduced_motion {
                1.0
            } else {
                let turns = 3.0 * t + 3.75 * t * t;
                0.55 + 0.45 * (0.5 + 0.5 * (std::f32::consts::TAU * turns).cos())
            };
            StructureLook {
                glow: Some((STRUCT_OVERLOAD, (0.3 + 0.45 * state.progress) * beat)),
                bearing: 1.0,
                bearing_color: STRUCT_OVERLOAD,
                ..StructureLook::default()
            }
        }
        Phase::Unstable => {
            let secs = game.run_ticks as f32 / TICK_RATE;
            // Each brick out of step with its neighbors.
            let phase = brick.x * 0.37 + brick.y * 0.61;
            let dx = if reduced_motion {
                0.0
            } else {
                (std::f32::consts::TAU * UNSTABLE_SHAKE_HZ * secs + phase).sin() * UNSTABLE_SHAKE_PX
            };
            StructureLook {
                offset: [dx, 0.0],
                glow: Some((STRUCT_UNSTABLE, 0.4 + 0.25 * state.progress)),
                bearing: 0.0,
                bearing_color,
            }
        }
    }
}

// -- capsules: hue plus letter ---------------------------------------------

pub const CAPSULE_WIDEN: Rgb = hex(0x3DC46B);
pub const CAPSULE_SLOW: Rgb = hex(0x3F8EF5);
pub const CAPSULE_MULTIBALL: Rgb = hex(0x9C66F2);
pub const CAPSULE_GLUE: Rgb = hex(0xEE62A0);
pub const CAPSULE_LASER: Rgb = hex(0xEE463B);
pub const CAPSULE_SUPER_LASER: Rgb = hex(0x2BCFD8);
pub const CAPSULE_TINY: Rgb = hex(0x9BA5B9);
pub const CAPSULE_PLAYER: Rgb = hex(0xF0C232);
pub const CAPSULE_BREAK: Rgb = hex(0xFF8A2B);
/// Chartreuse, not the explosion's amber: amber sits within a few steps of
/// Player's gold and Break's orange, chartreuse has the green-yellow gap
/// to itself. Every ball wears it while the power lasts.
pub const CAPSULE_EXPLOSIVE_BALL: Rgb = hex(0xC5F03C);
/// Polished steel, the lightest capsule: Tiny's gray is the dull one.
/// Every ball wears it while the power lasts.
pub const CAPSULE_METAL_BALL: Rgb = hex(0xE6E9F0);
/// Magenta, the one hue between Multiball's purple and Glue's pink that
/// no capsule had: the rare capsule reads as none of the others.
pub const CAPSULE_METEOR_RAIN: Rgb = hex(0xE05BE8);

/// A capsule's hue. The open Break gate, the laser bolts and a powered ball
/// reuse their capsule's hue, so the cause and the effect read as one
/// thing.
pub fn capsule(kind: PowerUpKind) -> Rgb {
    match kind {
        PowerUpKind::Widen => CAPSULE_WIDEN,
        PowerUpKind::Slow => CAPSULE_SLOW,
        PowerUpKind::Multiball => CAPSULE_MULTIBALL,
        PowerUpKind::Glue => CAPSULE_GLUE,
        PowerUpKind::Laser => CAPSULE_LASER,
        PowerUpKind::SuperLaser => CAPSULE_SUPER_LASER,
        PowerUpKind::Tiny => CAPSULE_TINY,
        PowerUpKind::Player => CAPSULE_PLAYER,
        PowerUpKind::Break => CAPSULE_BREAK,
        PowerUpKind::ExplosiveBall => CAPSULE_EXPLOSIVE_BALL,
        PowerUpKind::MetalBall => CAPSULE_METAL_BALL,
        PowerUpKind::MeteorRain => CAPSULE_METEOR_RAIN,
    }
}

/// The letter on a capsule: the color-blind-safe half of its identity.
/// Unique across the twelve; Super laser takes X because S is Slow's,
/// Metal ball takes I (iron) because M is Multiball's, and Meteor rain
/// takes R (rain) for the same reason.
pub fn capsule_letter(kind: PowerUpKind) -> &'static str {
    match kind {
        PowerUpKind::Widen => "W",
        PowerUpKind::Slow => "S",
        PowerUpKind::Multiball => "M",
        PowerUpKind::Glue => "G",
        PowerUpKind::Laser => "L",
        PowerUpKind::SuperLaser => "X",
        PowerUpKind::Tiny => "T",
        PowerUpKind::Player => "P",
        PowerUpKind::Break => "B",
        PowerUpKind::ExplosiveBall => "E",
        PowerUpKind::MetalBall => "I",
        PowerUpKind::MeteorRain => "R",
    }
}

/// The ink a capsule's letter is set in: the field's own dark, which
/// clears 4.5:1 on all twelve hues (`capsule_letters_clear_contrast`).
pub const CAPSULE_INK: Rgb = FIELD;

// -- hazards and effects ---------------------------------------------------

/// Asteroids: cratered stone, cool and desaturated, so the one warm thing
/// about them is the explosion they become.
pub const ASTEROID: Rgb = hex(0x8B7F9A);
/// A meteorite of the Meteor rain: rock heated to the explosion's amber,
/// so it reads as the thing about to strike, not as an asteroid to hit.
pub const METEOR: Rgb = hex(0xFFB55E);
/// An asteroid's explosion.
pub const EXPLOSION: Rgb = hex(0xFFB55E);

// -- kinetic effects: light (src/fx/, docs/design/kinetic-fx.md) ----------
// Light, not objects: these color brief emissive effects (flashes, rings,
// glints, sparks) drawn above 1.0, where the bloom takes them. Nothing the
// player tracks wears them.

/// The rally's light, one hue per step of the pentatonic scale the brick
/// notes climb (`rally.rs`), root first: warm at the root, cooling as the
/// run climbs. The upper octave reuses them, with a ring added.
pub const FX_SCALE: [Rgb; 5] = [
    hex(0xFFD58A),
    hex(0xFFA474),
    hex(0xFF7FAE),
    hex(0xB59BFF),
    hex(0x7CCBFF),
];
/// The light inside an armored brick's crack: hot, like the amber plate.
pub const FX_CRACK: Rgb = hex(0xFFB347);
/// A crack's own line, after its light has gone: a fissure.
pub const FX_FISSURE: Rgb = hex(0x1A100A);
/// An indestructible brick's rebound: cold, hard light.
pub const FX_DEFLECT: Rgb = hex(0xC9E2FF);
/// The white-hot core of a blast, before it turns to `EXPLOSION`.
pub const FX_BLAST_CORE: Rgb = hex(0xFFF1D6);
/// Sparks off a metal ball.
pub const FX_SPARK: Rgb = hex(0xFFF4DC);

/// The rally color for a step on the scale.
pub fn rally(step: usize) -> Rgb {
    FX_SCALE[step % FX_SCALE.len()]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::{BallPower, Gun, GunKind};

    #[test]
    fn a_brick_shows_what_it_carries_and_what_is_coming() {
        use crate::game::{bricks_of_grid, Features};
        use crate::structure::Structure;
        // An anchor, a brick, and five more hung under it: the first one
        // carries one too many; the rest carry less down the chain.
        let grid = &["X", "1", "1", "1", "1", "1", "1"];
        let mut game = Game::new()
            .with_structures()
            .with_features(Features::STRUCTURES);
        let (bricks, origin) = bricks_of_grid(grid);
        game.bricks = bricks;
        game.structure = Structure::on_grid(origin);
        game.structure.survey(&game.bricks, |_| {});

        let look = |game: &Game, i: usize, reduced| structure_look(game, &game.bricks[i], reduced);
        // Steel carries everything and shows nothing.
        assert_eq!(look(&game, 0, false), StructureLook::default());
        // The last carries nothing: no mark. The one over it carries one
        // of its four: a short mark. Higher, a longer one.
        assert_eq!(look(&game, 6, false).bearing, 0.0);
        let (short, long) = (look(&game, 5, false).bearing, look(&game, 3, false).bearing);
        assert!(0.0 < short && short < long && long < 1.0, "{short} {long}");
        // Overloaded: a full mark and the orange glow, stronger as it
        // nears giving way, and steady with reduced motion.
        let first = look(&game, 1, true);
        assert_eq!(first.bearing, 1.0);
        let (color, early) = first.glow.unwrap();
        assert_eq!(color, STRUCT_OVERLOAD);
        for _ in 0..60 {
            game.structure
                .advance(&game.bricks, 1.0 / TICK_RATE, |_, _| {});
        }
        let (_, late) = look(&game, 1, true).glow.unwrap();
        assert!(late > early, "{early} -> {late}");
        assert_eq!(look(&game, 1, true).offset, [0.0, 0.0]);

        // Cut loose: white-hot, and shaking unless motion is reduced.
        game.bricks.remove(1);
        game.structure.survey(&game.bricks, |_| {});
        game.run_ticks = 7;
        let loose = look(&game, 1, false);
        assert_eq!(loose.glow.map(|g| g.0), Some(STRUCT_UNSTABLE));
        assert_ne!(loose.offset[0], 0.0);
        assert!(loose.offset[0].abs() <= UNSTABLE_SHAKE_PX);
        assert_eq!(look(&game, 1, true).offset, [0.0, 0.0]);
    }

    #[test]
    fn a_struck_paddle_shows_it_and_eases_as_it_comes_free() {
        let mut game = Game::new();
        assert_eq!(paddle_stun(&game), None);
        game.paddle_stun = RULES.stun_secs;
        let (color, full) = paddle_stun(&game).unwrap();
        assert_eq!(color, STRUCT_OVERLOAD);
        game.paddle_stun = RULES.stun_secs / 10.0;
        let (_, nearly_free) = paddle_stun(&game).unwrap();
        assert!(
            nearly_free < full && nearly_free > 0.4,
            "{nearly_free} {full}"
        );
        // In the air is hotter and stronger than still shaking in place;
        // nearer is stronger; and after the keystone it is gold.
        let zone = |falling, secs, on| FallZone {
            x: 400.0,
            width: 52.0,
            y: 554.0,
            on,
            falling,
            secs,
        };
        let (air, shake) = (
            fall_zone(&zone(true, 0.6, Landing::Paddle)),
            fall_zone(&zone(false, 0.6, Landing::Paddle)),
        );
        assert!(air.1 > shake.1);
        assert_ne!(air.0, shake.0);
        let (near, far) = (
            fall_zone(&zone(true, 0.1, Landing::Paddle)),
            fall_zone(&zone(true, 1.5, Landing::Paddle)),
        );
        assert!(near.1 > far.1, "{near:?} {far:?}");
        assert_eq!(
            fall_zone(&zone(true, 0.5, Landing::Catch)).0,
            BRICK_KEYSTONE
        );
        assert_eq!(
            fall_zone(&zone(true, 0.5, Landing::Brick)).0,
            STRUCT_OVERLOAD
        );
    }

    #[test]
    fn a_power_shows_fully_until_its_warning_then_pulses_under_3_hz() {
        assert_eq!(expiry_strength(15.0, false), 1.0);
        assert_eq!(expiry_strength(POWER_WARNING_SECS, false), 1.0);
        let samples: Vec<f32> = (1..400)
            .map(|i| expiry_strength(POWER_WARNING_SECS * (1.0 - i as f32 / 400.0), false))
            .collect();
        assert!(samples.iter().all(|k| (0.0..=1.0).contains(k)));
        let troughs = samples
            .windows(3)
            .filter(|w| w[1] < w[0] && w[1] <= w[2])
            .count();
        let hz = troughs as f32 / POWER_WARNING_SECS;
        assert!((1.0..3.0).contains(&hz), "{hz} Hz");
    }

    #[test]
    fn with_reduced_motion_a_power_fades_out_steadily() {
        let samples: Vec<f32> = (0..=40)
            .map(|i| expiry_strength(POWER_WARNING_SECS * (1.0 - i as f32 / 40.0), true))
            .collect();
        assert!(samples.windows(2).all(|w| w[1] <= w[0]), "never brightens");
        assert_eq!(samples[40], 0.0);
    }

    #[test]
    fn the_paddle_warns_of_its_soonest_power_and_the_ball_of_its_own() {
        let mut game = Game::new();
        assert!(paddle_expiry(&game, true).is_none());
        assert_eq!(ball_power_strength(&game, true), 1.0);

        game.widen_timer = 1.5;
        game.gun = Some(Gun {
            kind: GunKind::Laser,
            timer: 0.5,
        });
        game.ball_power = Some(BallPower {
            mode: BallMode::Metal,
            timer: 0.2,
        });
        let (hue, k) = paddle_expiry(&game, true).expect("two paddle powers warning");
        assert_eq!(hue, CAPSULE_LASER, "the gun runs out first");
        assert!((k - 0.25).abs() < 1e-6);
        assert!((ball_power_strength(&game, true) - 0.1).abs() < 1e-6);

        game.gun = None;
        game.widen_timer = 9.0;
        assert!(
            paddle_expiry(&game, true).is_none(),
            "not in its warning yet"
        );
    }

    const ALL_CAPSULES: [PowerUpKind; 12] = [
        PowerUpKind::Widen,
        PowerUpKind::Slow,
        PowerUpKind::Multiball,
        PowerUpKind::Glue,
        PowerUpKind::Laser,
        PowerUpKind::SuperLaser,
        PowerUpKind::Tiny,
        PowerUpKind::Player,
        PowerUpKind::Break,
        PowerUpKind::ExplosiveBall,
        PowerUpKind::MetalBall,
        PowerUpKind::MeteorRain,
    ];

    /// WCAG relative luminance of an sRGB color.
    fn luminance(c: Rgb) -> f32 {
        let [r, g, b, _] = c.linear();
        0.2126 * r + 0.7152 * g + 0.0722 * b
    }

    fn contrast(a: Rgb, b: Rgb) -> f32 {
        let (la, lb) = (luminance(a), luminance(b));
        (la.max(lb) + 0.05) / (la.min(lb) + 0.05)
    }

    #[test]
    fn linear_round_trips_through_srgb() {
        // The two ends and the classic mid-gray: 0.5 linear is about 188.
        assert_eq!(hex(0x000000).linear(), [0.0, 0.0, 0.0, 1.0]);
        assert_eq!(hex(0xFFFFFF).linear(), [1.0, 1.0, 1.0, 1.0]);
        let mid = hex(0xBCBCBC).linear()[0];
        assert!((mid - 0.5).abs() < 0.01, "#BC decodes to {mid}");
    }

    #[test]
    fn capsule_letters_are_unique() {
        let mut letters: Vec<_> = ALL_CAPSULES.iter().map(|&k| capsule_letter(k)).collect();
        letters.sort_unstable();
        letters.dedup();
        assert_eq!(letters.len(), ALL_CAPSULES.len());
    }

    #[test]
    fn capsule_hues_are_distinct() {
        for (i, &a) in ALL_CAPSULES.iter().enumerate() {
            for &b in &ALL_CAPSULES[i + 1..] {
                assert_ne!(capsule(a), capsule(b), "{a:?} vs {b:?}");
            }
        }
    }

    #[test]
    fn capsule_letters_clear_contrast() {
        for kind in ALL_CAPSULES {
            let ratio = contrast(capsule(kind), CAPSULE_INK);
            assert!(ratio >= 4.5, "{kind:?}: {ratio:.2}:1");
        }
    }

    #[test]
    fn text_clears_contrast_on_its_grounds() {
        for ground in [BAND, FIELD] {
            assert!(contrast(TEXT, ground) >= 7.0);
            assert!(contrast(TEXT_DIM, ground) >= 4.5);
        }
    }

    #[test]
    fn the_ball_is_the_brightest_moving_object() {
        let ball = luminance(BALL);
        for kind in ALL_CAPSULES {
            assert!(luminance(capsule(kind)) < ball, "{kind:?}");
        }
        assert!(luminance(ASTEROID) < ball && luminance(PADDLE) < ball);
    }
}
