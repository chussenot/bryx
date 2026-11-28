//! Events emitted by the simulation each tick, describing what happened
//! during that tick so other layers (render now, audio later) can react
//! without the simulation depending on them.
//!
//! Emission pattern (for `game.rs`, wired up in a later milestone): `Game`
//! owns a `Vec<GameEvent>` field that `tick()` pushes onto as things happen
//! during that tick. The caller (`main.rs`'s loop) drains it after every
//! tick — `std::mem::take(&mut game.events)` or `game.events.drain(..)` —
//! before the next tick runs, so events never accumulate across ticks and
//! never leak into the interpolated render frames between ticks.
//!
//! Events are notifications, not a state snapshot: a consumer reacting to
//! `LevelCleared` reads the new level number off `Game` itself rather than
//! finding it on the event. The only payload carried here is what a reactor
//! cannot otherwise recover — which power-up kind spawned or was caught.

/// The power-ups a brick can drop: v1's three, plus Glue, Laser, Super laser,
/// Tiny, Player, Break, Explosive ball and Metal ball (docs/spec.md,
/// "Additions after v1"; off under `game::Features::V1`).
///
/// ponytail: no code constructs these yet -- brick/power-up gameplay lands
/// in M3/M4. `#[allow(dead_code)]` until then; drop it once `game.rs` emits
/// real `PowerUpSpawned`/`PowerUpCaught` events.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PowerUpKind {
    Widen,
    Slow,
    Multiball,
    Glue,
    Laser,
    SuperLaser,
    Tiny,
    Player,
    Break,
    ExplosiveBall,
    MetalBall,
    /// The rare one: a rain of meteorites on the bricks.
    MeteorRain,
}

/// Something the simulation did this tick.
///
/// ponytail: `Game::tick` is still the M1 no-op skeleton, so nothing
/// constructs these variants yet -- `#[allow(dead_code)]` until paddle/
/// ball/brick gameplay (M2+) starts pushing onto `Game::events`.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GameEvent {
    /// A brick was destroyed (last hit on a normal or armored brick).
    BrickDestroyed,
    /// A ball bounced off the paddle. The autopilot (`src/autopilot/`)
    /// counts these as rallies: each one is a point where a new shot is
    /// planned.
    PaddleHit,
    /// The paddle slid out through the open Break gate: the level is left
    /// (`LevelCleared` follows, or `Victory` on the last one).
    Warped,
    /// The paddle's gun fired (one Laser pair, or one Super laser bolt).
    GunFired(crate::game::GunKind),
    /// An asteroid drifted in from the top.
    AsteroidSpawned,
    /// An asteroid was blown up by a ball or a bolt, at its last
    /// position: it is gone from `Game::asteroids` by the time this fires,
    /// so the renderers' explosion effect reads its spot from here.
    AsteroidExploded { x: f32, y: f32 },
    /// An asteroid reached the paddle and blew up on it, costing points:
    /// drawn like `AsteroidExploded`, heard as its own sound.
    AsteroidHitPaddle { x: f32, y: f32 },
    /// An explosive ball blew up against a brick, centered on that brick:
    /// the renderers draw it like an asteroid's explosion, the director
    /// plays it a little quieter (there can be many in a row).
    BallExploded { x: f32, y: f32 },
    /// A meteorite of the Meteor rain landed on a brick, at the point of
    /// its top face it hit. The brick's own event follows in the same
    /// tick: `BrickDestroyedAt` if it broke, `BrickStruck` (by a blast) if
    /// it is indestructible.
    MeteorStruck { x: f32, y: f32 },
    /// A brick started carrying more than it can (structural levels): it
    /// cracks from now until it gives way. `rect` is its center and size.
    BrickOverloaded {
        rect: [f32; 4],
        load: u16,
        capacity: u16,
    },
    /// An overloaded brick broke under what it carried. Its
    /// `BrickDestroyedAt` follows in the same tick.
    BrickGaveWay { rect: [f32; 4] },
    /// A brick that carried `dependents` others is gone, however it went.
    SupportLost { x: f32, y: f32, dependents: u16 },
    /// `count` bricks lost every hold and are about to fall, centered on
    /// (`x`, `y`).
    SectionUnstable { count: u16, x: f32, y: f32 },
    /// A brick let go and is falling: the `nth` brought down in this
    /// collapse (1 for the first). It is scored now and no longer a brick;
    /// no `BrickDestroyedAt` follows.
    BrickFell {
        rect: [f32; 4],
        kind: crate::levels::BrickKind,
        nth: u32,
    },
    /// A falling brick landed on a standing one, at the middle of the top
    /// face it hit, and shattered there.
    DebrisLanded { x: f32, y: f32 },
    /// A falling brick landed on the paddle, at the middle of the brick on
    /// the paddle's top, and shattered there: the paddle cannot move for a
    /// moment.
    PaddleStruck { x: f32, y: f32 },
    /// The keystone came down: the level is won, and what is left of it
    /// lets go.
    KeystoneFell { x: f32, y: f32 },
    /// A collapse is over (nothing cracking, shaking or falling any more):
    /// how many bricks it brought down, what they scored, and in how many
    /// waves (sections that let go one after the other).
    CollapseEnded { bricks: u32, score: u32, waves: u32 },
    /// A falling brick landed on the paddle after the keystone fell: it is
    /// caught, the `nth` of the level's finale, and scores `points` again.
    DebrisCaught {
        x: f32,
        y: f32,
        nth: u32,
        points: u32,
    },
    /// A ball fell below the bottom edge. Fires once per ball, including
    /// during multiball — see `LifeLost` for the life-loss rule.
    BallLost,
    /// A life was actually spent: the *last* ball on the field was lost.
    LifeLost,
    /// All destructible bricks on the current level are gone.
    LevelCleared,
    /// A power-up began falling from a destroyed brick.
    PowerUpSpawned(PowerUpKind),
    /// The paddle caught a falling power-up.
    PowerUpCaught(PowerUpKind),
    /// A timed power has `game::POWER_WARNING_SECS` left: the warning the
    /// player hears before it runs out. Once per running, when its timer
    /// crosses the mark; catching it again refills the timer and warns again
    /// later.
    PowerUpExpiring(PowerUpKind),
    /// The last level was cleared: the player won.
    Victory,
    /// Lives reached zero.
    GameOver,
    /// Same moment as `BrickDestroyed`, carrying the destroyed brick's
    /// world geometry and kind -- data `BrickDestroyed` itself doesn't
    /// carry. Added for the 3D renderer's destroy-tumble effect
    /// (arkanoid-v2-c3): by the time the event fires the brick is already
    /// gone from `Game::bricks` (removed just before this pushes), so a
    /// render-side consumer has nowhere else to read its position/size/
    /// kind from in order to spawn a tumbling cube in the right place.
    /// Pushed alongside `BrickDestroyed` at the same call site, same
    /// tick -- not a replacement for it.
    BrickDestroyedAt {
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        kind: crate::levels::BrickKind,
    },
    /// Something struck a brick: a ball, or a gun's bolt. The contact
    /// itself, for presentation (`fx/`): the brick's rect (center x, y,
    /// width, height), the striker's center at contact, the outward
    /// normal of the face it came through, and the hits the brick has left
    /// after it (0: it broke, and `BrickDestroyedAt` follows in the same
    /// tick). Pushed for indestructible bricks too: their rebound is an
    /// event with no other trace. No rule reads it.
    BrickStruck {
        rect: [f32; 4],
        at: [f32; 2],
        normal: [f32; 2],
        kind: crate::levels::BrickKind,
        hits_left: u8,
        by: Striker,
    },
}

/// What struck a brick (`GameEvent::BrickStruck`): each one's impact reads
/// differently.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Striker {
    Ball,
    MetalBall,
    ExplosiveBall,
    Laser,
    SuperLaser,
    /// An explosive ball's or an asteroid's blast, on a brick it hit
    /// without breaking: the contact is the brick's point nearest the
    /// blast. (Bricks a blast breaks are `BrickDestroyedAt` only.)
    Blast,
    /// A falling brick landing on it (structural levels).
    Debris,
}
