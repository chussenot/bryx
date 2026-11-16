//! Structural destruction: which bricks hold which, what each one carries,
//! and what lets go (docs/design/structural-gameplay.md).
//!
//! The model is a graph on the level's grid, and nothing else:
//!
//! - **Anchors.** Indestructible bricks are fixed to the world.
//! - **Bonds.** Two bricks in cells that share an edge hold each other. A
//!   beam bonds only to its left and right, a tie only above and below.
//! - **Held.** A brick stands while a chain of bonds leads from it to an
//!   anchor. One that loses every chain is *unstable*: it shakes for a
//!   moment, then lets go and falls.
//! - **Load.** What a held brick carries is the number of bricks that
//!   would lose their hold if it alone were gone. A brick carrying more
//!   than its kind and its remaining hits allow is *overloaded*: it cracks
//!   for a fixed time, then gives way.
//!
//! No physics engine, no randomness, no frame time: the same bricks give
//! the same loads, and the same ticks give the same collapse. The graph is
//! surveyed only when the number of bricks changes, never per frame, and
//! one survey is bounded by the grid (196 cells, each flooded once per
//! cell at worst).
//!
//! Pure, like `game.rs`, which owns what falls: this module only says what
//! holds, what is overloaded and when a brick lets go.

use crate::game::Brick;
use crate::levels::{BrickKind, BRICK_HEIGHT, BRICK_WIDTH};

/// The largest structural grid.
pub const MAX_COLS: usize = 14;
pub const MAX_ROWS: usize = 14;
const CELLS: usize = MAX_COLS * MAX_ROWS;
/// No cell: a neighbour off the grid, or nothing blocked in a flood.
const NONE: usize = usize::MAX;

const LEFT: u8 = 1;
const RIGHT: u8 = 2;
const UP: u8 = 4;
const DOWN: u8 = 8;

/// Every number the structural rules use, in one place: balancing is a
/// change to this table, not to the code that reads it.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rules {
    /// Bricks a normal brick can carry.
    pub carries_normal: u16,
    /// Bricks an armored brick can carry, per hit it has left.
    pub carries_armored: u16,
    /// Bricks a support can carry, per hit it has left: a cracked support
    /// holds half as much, so the weight can finish what one hit started.
    pub carries_support: u16,
    /// Bricks a beam or a tie can carry.
    pub carries_beam: u16,
    /// How long an overloaded brick cracks before it gives way.
    pub give_way_secs: f32,
    /// How long a brick that lost its hold shakes before it lets go.
    pub unstable_secs: f32,
    /// The delay between one row of a falling section and the row above:
    /// the lowest row lets go first, so a section peels off bottom-up and
    /// no falling brick overtakes another.
    pub row_stagger_secs: f32,
    /// Falling bricks: acceleration and top speed, in px/s² and px/s.
    pub gravity: f32,
    pub terminal_speed: f32,
    /// What a brick brought down scores, times what it scores broken by
    /// the ball: structure is worth more than chipping.
    pub collapse_factor: u32,
    /// The keystone's value (before the multipliers).
    pub keystone_score: u32,
    /// How long the paddle cannot move after a falling brick lands on it.
    pub stun_secs: f32,
}

pub const RULES: Rules = Rules {
    carries_normal: 4,
    carries_armored: 3,
    carries_support: 6,
    carries_beam: 6,
    give_way_secs: 0.8,
    unstable_secs: 0.3,
    row_stagger_secs: 0.05,
    gravity: 1500.0,
    terminal_speed: 640.0,
    collapse_factor: 2,
    keystone_score: 250,
    stun_secs: 0.75,
};

// A falling brick moves less than a third of a brick a tick, so it lands
// on the first brick under it and never skips one.
const _: () = assert!(RULES.terminal_speed / crate::game::TICK_RATE < BRICK_HEIGHT / 3.0);

impl Rules {
    /// How many bricks a brick of this kind, with `hits` left, can carry.
    pub fn capacity(&self, kind: BrickKind, hits: u8) -> u16 {
        let hits = u16::from(hits.max(1));
        match kind {
            BrickKind::Normal => self.carries_normal,
            BrickKind::Armored => self.carries_armored * hits,
            BrickKind::Support => self.carries_support * hits,
            BrickKind::Beam | BrickKind::Tie => self.carries_beam,
            BrickKind::Indestructible | BrickKind::Keystone => u16::MAX,
        }
    }
}

/// The sides a brick bonds on.
fn sides(kind: BrickKind) -> u8 {
    match kind {
        BrickKind::Beam => LEFT | RIGHT,
        BrickKind::Tie => UP | DOWN,
        _ => LEFT | RIGHT | UP | DOWN,
    }
}

/// A cell's four neighbours, each with the side it is on and the side it
/// is reached through; `NONE` off the grid.
fn neighbours(i: usize) -> [(u8, u8, usize); 4] {
    let (row, col) = (i / MAX_COLS, i % MAX_COLS);
    [
        (LEFT, RIGHT, if col > 0 { i - 1 } else { NONE }),
        (RIGHT, LEFT, if col + 1 < MAX_COLS { i + 1 } else { NONE }),
        (UP, DOWN, if row > 0 { i - MAX_COLS } else { NONE }),
        (
            DOWN,
            UP,
            if row + 1 < MAX_ROWS {
                i + MAX_COLS
            } else {
                NONE
            },
        ),
    ]
}

/// Marks every cell a chain of bonds reaches from an anchor, going around
/// `blocked`, and counts the ones that are not anchors themselves.
fn flood(
    link: &[u8; CELLS],
    anchor: &[bool; CELLS],
    blocked: usize,
    reached: &mut [bool; CELLS],
) -> u16 {
    // Each cell is pushed once at most, so the stack never outgrows the
    // grid; its 196 cells index in a byte.
    let mut stack = [0u8; CELLS];
    let mut top = 0;
    let mut count = 0;
    for i in 0..CELLS {
        if anchor[i] && i != blocked {
            reached[i] = true;
            stack[top] = i as u8;
            top += 1;
        }
    }
    while top > 0 {
        top -= 1;
        let i = stack[top] as usize;
        for (side, back, j) in neighbours(i) {
            if j == NONE || j == blocked || reached[j] {
                continue;
            }
            if link[i] & side != 0 && link[j] & back != 0 {
                reached[j] = true;
                if !anchor[j] {
                    count += 1;
                }
                stack[top] = j as u8;
                top += 1;
            }
        }
    }
    count
}

/// Where a brick is in its structural life. A brick that let go is no
/// longer a brick: `game.rs` holds it as falling debris.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Phase {
    /// Held, and carrying half of what it can or less.
    #[default]
    Intact,
    /// Held, and carrying more than half of what it can.
    Stressed,
    /// Carrying more than it can: cracking, about to give way.
    Overloaded,
    /// Nothing holds it: shaking, about to fall.
    Unstable,
}

/// One brick's structural state, for the renderers and the HUD.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct BrickState {
    pub phase: Phase,
    /// Bricks that would fall if this one alone were gone.
    pub load: u16,
    /// Bricks it can carry as it is now.
    pub capacity: u16,
    /// From 0 to 1: how far an overloaded brick is from giving way, or an
    /// unstable one from letting go. 0 otherwise.
    pub progress: f32,
}

/// What a survey found changed.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Change {
    /// A brick that carried `dependents` others is gone.
    SupportLost { at: [f32; 2], dependents: u16 },
    /// `count` bricks lost their hold, centered on `at`.
    Unstable { count: u16, at: [f32; 2] },
}

/// What one tick decided for a brick.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Verdict {
    /// It now carries more than it can.
    Overloaded { load: u16, capacity: u16 },
    /// It cracked through: it breaks where it stands.
    GaveWay,
    /// It was unstable long enough: it falls.
    LetGo,
    /// It falls because the keystone fell ([`Structure::release`]), not
    /// because of anything that happened to it.
    Released,
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
struct Cell {
    occupied: bool,
    held: bool,
    load: u16,
    /// Seconds spent overloaded.
    strain: f32,
    overloaded: bool,
    letting_go: bool,
    /// Let go by the keystone's fall, not by losing its hold.
    released: bool,
    /// Seconds until it lets go, once `letting_go`.
    fall_in: f32,
}

/// A level's structure: the grid it stands on and each cell's state.
#[derive(Debug, Clone)]
pub struct Structure {
    active: bool,
    /// The grid's top-left corner on the field.
    origin: [f32; 2],
    /// How many bricks the last survey saw: bricks only ever leave a
    /// level, so a different count is a different structure.
    seen: usize,
    /// A brick is cracking or about to fall.
    pending: bool,
    cells: [Cell; CELLS],
}

impl Structure {
    /// No structure: a level that plays by the plain rules.
    pub fn off() -> Self {
        Self {
            active: false,
            origin: [0.0; 2],
            seen: usize::MAX,
            pending: false,
            cells: [Cell::default(); CELLS],
        }
    }

    /// The structure of a level whose grid has its top-left corner at
    /// `origin` on the field.
    pub fn on_grid(origin: [f32; 2]) -> Self {
        Self {
            active: true,
            origin,
            ..Self::off()
        }
    }

    pub fn is_active(&self) -> bool {
        self.active
    }

    /// The cell a brick stands in, if it is on the grid.
    fn cell_of(&self, brick: &Brick) -> Option<usize> {
        let col = ((brick.x - self.origin[0]) / BRICK_WIDTH).floor();
        let row = ((brick.y - self.origin[1]) / BRICK_HEIGHT).floor();
        let on_grid =
            (0.0..MAX_COLS as f32).contains(&col) && (0.0..MAX_ROWS as f32).contains(&row);
        on_grid.then(|| row as usize * MAX_COLS + col as usize)
    }

    fn center(&self, i: usize) -> [f32; 2] {
        [
            self.origin[0] + ((i % MAX_COLS) as f32 + 0.5) * BRICK_WIDTH,
            self.origin[1] + ((i / MAX_COLS) as f32 + 0.5) * BRICK_HEIGHT,
        ]
    }

    /// Looks at the bricks again if any left since the last look: which are
    /// still held, what each one now carries, and which just lost their
    /// hold. `on` hears what changed.
    pub fn survey(&mut self, bricks: &[Brick], mut on: impl FnMut(Change)) {
        if !self.active || bricks.len() == self.seen {
            return;
        }
        self.seen = bricks.len();

        let mut link = [0u8; CELLS];
        let mut anchor = [false; CELLS];
        for brick in bricks {
            if let Some(i) = self.cell_of(brick) {
                link[i] = sides(brick.kind);
                anchor[i] = brick.kind == BrickKind::Indestructible;
            }
        }

        // A brick that was carrying others and is gone: a support lost.
        for (i, &sides) in link.iter().enumerate() {
            let cell = self.cells[i];
            if cell.occupied && sides == 0 {
                if cell.held && cell.load > 0 {
                    on(Change::SupportLost {
                        at: self.center(i),
                        dependents: cell.load,
                    });
                }
                self.cells[i] = Cell::default();
            }
            self.cells[i].occupied = sides != 0;
        }

        let mut held = [false; CELLS];
        let standing = flood(&link, &anchor, NONE, &mut held);

        // What no anchor reaches any more lets go, lowest row first.
        let loose =
            |cells: &[Cell; CELLS], i: usize| link[i] != 0 && !held[i] && !cells[i].letting_go;
        let (mut count, mut lowest, mut sum) = (0u16, 0, [0.0f32; 2]);
        for i in 0..CELLS {
            if loose(&self.cells, i) {
                count += 1;
                lowest = lowest.max(i / MAX_COLS);
                let at = self.center(i);
                sum = [sum[0] + at[0], sum[1] + at[1]];
            }
        }
        if count > 0 {
            for i in 0..CELLS {
                if loose(&self.cells, i) {
                    let rows_up = (lowest - i / MAX_COLS) as f32;
                    self.cells[i].letting_go = true;
                    self.cells[i].fall_in = RULES.unstable_secs + rows_up * RULES.row_stagger_secs;
                }
            }
            let n = f32::from(count);
            on(Change::Unstable {
                count,
                at: [sum[0] / n, sum[1] / n],
            });
        }

        // What each held brick carries: the bricks that stand only through
        // it. A brick bonded to fewer than two others carries nothing.
        for i in 0..CELLS {
            self.cells[i].held = held[i];
            self.cells[i].load = 0;
            if !held[i] || anchor[i] {
                continue;
            }
            let bonds = neighbours(i)
                .iter()
                .filter(|&&(side, back, j)| j != NONE && link[i] & side != 0 && link[j] & back != 0)
                .count();
            if bonds < 2 {
                continue;
            }
            let mut without = [false; CELLS];
            let still = flood(&link, &anchor, i, &mut without);
            self.cells[i].load = standing - 1 - still;
        }
    }

    /// One tick of cracking and shaking. `on` hears, with the brick's index
    /// in `bricks`, each brick that became overloaded, gave way or let go;
    /// the caller removes the last two.
    pub fn advance(&mut self, bricks: &[Brick], dt: f32, mut on: impl FnMut(usize, Verdict)) {
        self.pending = false;
        if !self.active {
            return;
        }
        for (index, brick) in bricks.iter().enumerate() {
            let Some(i) = self.cell_of(brick) else {
                continue;
            };
            let cell = &mut self.cells[i];
            if cell.letting_go {
                cell.fall_in -= dt;
                if cell.fall_in <= 0.0 {
                    let verdict = if cell.released {
                        Verdict::Released
                    } else {
                        Verdict::LetGo
                    };
                    on(index, verdict);
                } else {
                    self.pending = true;
                }
                continue;
            }
            let capacity = RULES.capacity(brick.kind, brick.hits_remaining);
            if cell.load > capacity {
                if !cell.overloaded {
                    cell.overloaded = true;
                    on(
                        index,
                        Verdict::Overloaded {
                            load: cell.load,
                            capacity,
                        },
                    );
                }
                cell.strain += dt;
                if cell.strain >= RULES.give_way_secs {
                    on(index, Verdict::GaveWay);
                } else {
                    self.pending = true;
                }
            } else {
                cell.overloaded = false;
                cell.strain = (cell.strain - dt).max(0.0);
            }
        }
    }

    /// Lets every breakable brick go, lowest row first: what the keystone's
    /// fall does to what is left of the level.
    pub fn release(&mut self, bricks: &[Brick]) {
        let cells: Vec<usize> = bricks
            .iter()
            .filter(|b| b.kind.breakable())
            .filter_map(|b| self.cell_of(b))
            .filter(|&i| !self.cells[i].letting_go)
            .collect();
        let lowest = cells.iter().map(|i| i / MAX_COLS).max().unwrap_or(0);
        for i in cells {
            let rows_up = (lowest - i / MAX_COLS) as f32;
            self.cells[i].letting_go = true;
            self.cells[i].released = true;
            self.cells[i].fall_in = RULES.unstable_secs + rows_up * RULES.row_stagger_secs;
            self.pending = true;
        }
    }

    /// Nothing is cracking, shaking or unsurveyed: the structure is at
    /// rest.
    pub fn at_rest(&self, bricks: &[Brick]) -> bool {
        !self.active || (!self.pending && bricks.len() == self.seen)
    }

    /// What would come down if `bricks[index]` broke now, by the graph
    /// alone: how many bricks would give way or fall after it, and whether
    /// the keystone is one of them. It leaves out what the falling bricks
    /// would land on, so it can only undercount. For the autopilot, and for
    /// anything else that wants to know where the weak point is.
    pub fn what_falls(&self, bricks: &[Brick], index: usize) -> (u16, bool) {
        let mut left = bricks.to_vec();
        left.remove(index);
        let mut after = Self::on_grid(self.origin);
        let (mut count, mut keystone) = (0, false);
        // Each pass takes at least one brick or stops.
        loop {
            after.survey(&left, |_| {});
            let before = left.len();
            left.retain(|brick| {
                let phase = after.state(brick).phase;
                let goes = matches!(phase, Phase::Overloaded | Phase::Unstable);
                keystone |= goes && brick.kind == BrickKind::Keystone;
                !goes
            });
            if left.len() == before {
                return (count, keystone);
            }
            count += (before - left.len()) as u16;
        }
    }

    /// What holds the keystone up: the cheapest set of bricks to break,
    /// counted in the hits they have left, that leaves it with no chain to
    /// an anchor. Indices into `bricks`; empty when there is no keystone,
    /// or an anchor touches it.
    ///
    /// It counts chains, not weight: a structure often comes down for
    /// less, when a cut leaves a hold carrying more than it can
    /// (`what_falls` sees that). So it is an upper bound on the work a
    /// level asks for, and what to aim at when no single brick brings
    /// anything down.
    ///
    /// A minimum vertex cut, by augmenting paths on the grid with each
    /// brick split into an in and an out side joined by its hits: the
    /// flow is at most the hits of the keystone's four neighbors, so it
    /// takes eight searches of the grid at most.
    pub fn keystone_holds(&self, bricks: &[Brick]) -> Vec<usize> {
        const FREE: u8 = u8::MAX;
        let mut link = [0u8; CELLS];
        let mut anchor = [false; CELLS];
        let mut hits = [0u8; CELLS];
        let mut index_of = [NONE; CELLS];
        let mut source = NONE;
        for (index, brick) in bricks.iter().enumerate() {
            let Some(i) = self.cell_of(brick) else {
                continue;
            };
            link[i] = sides(brick.kind);
            anchor[i] = brick.kind == BrickKind::Indestructible;
            hits[i] = if brick.kind.breakable() {
                brick.hits_remaining.max(1)
            } else {
                FREE
            };
            index_of[i] = index;
            if brick.kind == BrickKind::Keystone {
                source = i;
            }
        }
        if source == NONE {
            return Vec::new();
        }

        // Flow through each brick, and from each brick's out side to each
        // neighbor's in side. A search state is `2 * cell` for a brick's
        // in side, `2 * cell + 1` for its out side.
        let mut through = [0u8; CELLS];
        let mut across = [[0u8; 4]; CELLS];
        let bonded = |i: usize, d: usize| {
            let (side, back, j) = neighbours(i)[d];
            (j != NONE && link[i] & side != 0 && link[j] & back != 0).then_some(j)
        };
        // Eight units of flow at most, and one last search that finds none.
        for _ in 0..=8 {
            let mut from = [NONE; 2 * CELLS];
            let mut queue = [0usize; 2 * CELLS];
            let (mut head, mut tail) = (0, 0);
            let mut reached = [false; 2 * CELLS];
            let start = 2 * source + 1;
            reached[start] = true;
            queue[tail] = start;
            tail += 1;
            let mut sink = NONE;
            while head < tail && sink == NONE {
                let state = queue[head];
                head += 1;
                let (i, out) = (state / 2, state % 2 == 1);
                let mut visit = |next: usize| {
                    if !reached[next] {
                        reached[next] = true;
                        from[next] = state;
                        queue[tail] = next;
                        tail += 1;
                    }
                };
                if out {
                    // On to a neighbor, or back through this brick.
                    for d in 0..4 {
                        if let Some(j) = bonded(i, d) {
                            visit(2 * j);
                        }
                    }
                    if through[i] > 0 {
                        visit(2 * i);
                    }
                } else if anchor[i] {
                    sink = state;
                } else {
                    // Through this brick, or back along a neighbor's flow.
                    if hits[i] == FREE || through[i] < hits[i] {
                        visit(2 * i + 1);
                    }
                    for d in 0..4 {
                        if let Some(j) = bonded(i, d) {
                            if across[j][d ^ 1] > 0 {
                                visit(2 * j + 1);
                            }
                        }
                    }
                }
            }
            if sink == NONE {
                // The cut: bricks the search entered and could not leave.
                return (0..CELLS)
                    .filter(|&i| reached[2 * i] && !reached[2 * i + 1] && hits[i] != FREE)
                    .map(|i| index_of[i])
                    .collect();
            }
            // One more unit along the path found.
            let mut state = sink;
            while from[state] != NONE {
                let prev = from[state];
                let (i, j) = (prev / 2, state / 2);
                match (prev % 2 == 1, state % 2 == 1) {
                    (false, true) if i == j => through[i] += 1,
                    (true, false) if i == j => through[i] -= 1,
                    (true, false) => {
                        let d = (0..4).find(|&d| bonded(i, d) == Some(j)).unwrap_or(0);
                        across[i][d] += 1;
                    }
                    _ => {
                        let d = (0..4).find(|&d| bonded(j, d) == Some(i)).unwrap_or(0);
                        across[j][d] -= 1;
                    }
                }
                state = prev;
            }
        }
        // An anchor touches the keystone, or something like it: no cut.
        Vec::new()
    }

    /// A brick's structural state.
    pub fn state(&self, brick: &Brick) -> BrickState {
        let Some(cell) = self
            .active
            .then(|| self.cell_of(brick))
            .flatten()
            .map(|i| self.cells[i])
        else {
            return BrickState::default();
        };
        let capacity = RULES.capacity(brick.kind, brick.hits_remaining);
        let (phase, progress) = if cell.letting_go {
            (
                Phase::Unstable,
                1.0 - (cell.fall_in / RULES.unstable_secs).clamp(0.0, 1.0),
            )
        } else if cell.load > capacity {
            (
                Phase::Overloaded,
                (cell.strain / RULES.give_way_secs).clamp(0.0, 1.0),
            )
        } else if u32::from(cell.load) * 2 > u32::from(capacity) {
            (Phase::Stressed, 0.0)
        } else {
            (Phase::Intact, 0.0)
        };
        BrickState {
            phase,
            load: cell.load,
            capacity,
            progress,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::bricks_of_grid;

    const DT: f32 = 1.0 / 120.0;

    /// A grid's bricks and their surveyed structure.
    fn built(grid: &[&str]) -> (Vec<Brick>, Structure) {
        let (bricks, origin) = bricks_of_grid(grid);
        let mut structure = Structure::on_grid(origin);
        structure.survey(&bricks, |_| {});
        (bricks, structure)
    }

    /// The brick in `grid`'s cell, by column and row.
    fn at(bricks: &[Brick], structure: &Structure, col: usize, row: usize) -> usize {
        bricks
            .iter()
            .position(|b| structure.cell_of(b) == Some(row * MAX_COLS + col))
            .unwrap_or_else(|| panic!("no brick at column {col}, row {row}"))
    }

    fn load(bricks: &[Brick], structure: &Structure, col: usize, row: usize) -> u16 {
        structure
            .state(&bricks[at(bricks, structure, col, row)])
            .load
    }

    #[test]
    fn a_chain_carries_what_hangs_from_it_link_by_link() {
        let (bricks, structure) = built(&["X", "1", "1", "1"]);
        assert_eq!(load(&bricks, &structure, 0, 1), 2);
        assert_eq!(load(&bricks, &structure, 0, 2), 1);
        assert_eq!(load(&bricks, &structure, 0, 3), 0);
        assert!(bricks
            .iter()
            .all(|b| structure.state(b).phase != Phase::Unstable));
    }

    #[test]
    fn two_holds_share_the_weight_and_neither_carries_it() {
        // Held from both ends: no one brick's loss drops another.
        let (bricks, structure) = built(&["X111X"]);
        for col in 1..4 {
            assert_eq!(load(&bricks, &structure, col, 0), 0);
        }
    }

    #[test]
    fn a_brick_no_anchor_reaches_is_unstable_and_lets_go_lowest_row_first() {
        let (mut bricks, mut structure) = built(&["X..", "1..", "1..", "1.."]);
        let cut = at(&bricks, &structure, 0, 1);
        bricks.remove(cut);
        let mut changes = Vec::new();
        structure.survey(&bricks, |c| changes.push(c));
        assert!(matches!(
            changes[..],
            [
                Change::SupportLost { dependents: 2, .. },
                Change::Unstable { count: 2, .. }
            ]
        ));
        let mut order = Vec::new();
        for tick in 0..120 {
            let mut gone = Vec::new();
            structure.advance(&bricks, DT, |index, verdict| {
                assert_eq!(verdict, Verdict::LetGo);
                gone.push(index);
            });
            for index in gone.into_iter().rev() {
                order.push((tick, bricks.remove(index).y));
            }
        }
        assert_eq!(order.len(), 2);
        assert!(order[0].0 < order[1].0, "the lower one first: {order:?}");
        assert!(order[0].1 > order[1].1, "the lower one first: {order:?}");
        let first = RULES.unstable_secs / DT;
        assert!((order[0].0 as f32 - first).abs() <= 1.0, "{order:?}");
    }

    #[test]
    fn a_beam_holds_sideways_and_a_tie_up_and_down_only() {
        // The brick over the beam and the brick beside the tie touch them,
        // and neither is held by them.
        let (bricks, structure) = built(&[".1.X", "X=.|", "...1"]);
        let over_beam = &bricks[at(&bricks, &structure, 1, 0)];
        assert_eq!(structure.state(over_beam).phase, Phase::Unstable);
        let beam = &bricks[at(&bricks, &structure, 1, 1)];
        assert_ne!(structure.state(beam).phase, Phase::Unstable);
        let under_tie = &bricks[at(&bricks, &structure, 3, 2)];
        assert_ne!(structure.state(under_tie).phase, Phase::Unstable);
        assert_eq!(load(&bricks, &structure, 3, 1), 1);
    }

    #[test]
    fn an_overloaded_brick_cracks_for_a_fixed_time_then_gives_way() {
        // One normal brick holding five: one more than it can.
        let (bricks, mut structure) = built(&["X", "1", "1", "1", "1", "1", "1"]);
        let top = at(&bricks, &structure, 0, 1);
        let state = structure.state(&bricks[top]);
        assert_eq!((state.load, state.capacity), (5, RULES.carries_normal));
        assert_eq!(state.phase, Phase::Overloaded);

        let mut heard = Vec::new();
        let mut ticks = 0;
        while !heard.contains(&(top, Verdict::GaveWay)) {
            structure.advance(&bricks, DT, |index, verdict| heard.push((index, verdict)));
            ticks += 1;
            assert!(ticks < 1000);
        }
        assert_eq!(
            heard[0],
            (
                top,
                Verdict::Overloaded {
                    load: 5,
                    capacity: 4
                }
            ),
            "announced once, when it starts"
        );
        assert_eq!(
            heard
                .iter()
                .filter(|(_, v)| matches!(v, Verdict::Overloaded { .. }))
                .count(),
            1
        );
        let expected = (RULES.give_way_secs / DT).ceil() as i32;
        assert!((ticks - expected).abs() <= 1, "{ticks} ticks");
    }

    #[test]
    fn what_falls_follows_the_overloads_to_the_end() {
        // The first level: cut one beam and the other row gives way, then
        // the block and the keystone fall. Break a brick of the block and
        // nothing follows.
        let grid = crate::levels::STRUCTURES[0].grid;
        let (bricks, structure) = built(grid);
        let beam = at(&bricks, &structure, 2, 3);
        let (count, keystone) = structure.what_falls(&bricks, beam);
        assert!(keystone);
        assert!(count >= 26, "{count}");
        let corner = at(&bricks, &structure, 5, 1);
        assert_eq!(structure.what_falls(&bricks, corner), (0, false));
        assert_eq!(bricks.len(), built(grid).0.len(), "a question, not a move");
    }

    /// The hits it takes to cut every chain from the keystone to an anchor.
    fn hold(grid: &[&str]) -> u32 {
        let (bricks, structure) = built(grid);
        structure
            .keystone_holds(&bricks)
            .iter()
            .map(|&i| u32::from(bricks[i].hits_remaining))
            .sum()
    }

    #[test]
    fn the_keystones_holds_are_the_cheapest_cut_in_hits() {
        // One chain of normal bricks: one hit, anywhere along it.
        assert_eq!(hold(&["X11K"]), 1);
        // Two chains: one hit each.
        assert_eq!(hold(&["X1K1X"]), 2);
        // A support costs two hits, so the cut goes through the normal
        // brick beside it; two supports in a row are still one chain.
        assert_eq!(hold(&["XS1K"]), 1);
        assert_eq!(hold(&["XSSK"]), 2);
        // A wide wall narrows to the keystone's own neighbors.
        assert_eq!(hold(&["XXXXX", "11111", "11111", "..K.."]), 1);
        assert_eq!(hold(&["XXXXX", "11111", ".1K1."]), 3);
        // A beam does not hold what is over it.
        assert_eq!(hold(&["X=K", "..1", "..X"]), 2);
        assert_eq!(hold(&["X.K", "..=", "..X"]), 0);
        // No keystone, or one an anchor touches: nothing to cut.
        assert_eq!(hold(&["X11"]), 0);
        assert_eq!(hold(&["XK"]), 0);

        // The cut names real bricks, and breaking them frees the keystone.
        let grid = crate::levels::STRUCTURES
            .iter()
            .find(|level| level.name == "Vault")
            .expect("the Vault")
            .grid;
        let (mut bricks, mut structure) = built(grid);
        let mut cut = structure.keystone_holds(&bricks);
        assert!(!cut.is_empty());
        cut.sort_unstable();
        for index in cut.into_iter().rev() {
            assert!(bricks[index].kind.breakable());
            bricks.remove(index);
        }
        structure.survey(&bricks, |_| {});
        let keystone = bricks
            .iter()
            .find(|b| b.kind == BrickKind::Keystone)
            .unwrap();
        assert_eq!(structure.state(keystone).phase, Phase::Unstable);
    }

    #[test]
    fn a_cracked_support_carries_half() {
        assert_eq!(RULES.capacity(BrickKind::Support, 2), 12);
        assert_eq!(RULES.capacity(BrickKind::Support, 1), 6);
        assert_eq!(RULES.capacity(BrickKind::Keystone, 0), u16::MAX);
    }

    #[test]
    fn the_same_bricks_give_the_same_structure() {
        let grid = crate::levels::STRUCTURES[3].grid;
        let (bricks, a) = built(grid);
        let (_, b) = built(grid);
        for brick in &bricks {
            assert_eq!(a.state(brick), b.state(brick));
        }
    }

    #[test]
    fn a_level_with_no_structure_reports_every_brick_intact() {
        let (bricks, _) = bricks_of_grid(&["111"]);
        let structure = Structure::off();
        assert!(structure.at_rest(&bricks));
        assert_eq!(structure.state(&bricks[0]), BrickState::default());
    }
}
