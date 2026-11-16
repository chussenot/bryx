//! Level grids as const `&[&str]` data.
//!
//! Chars: `.` empty, `1` normal, `2` armored, `X` indestructible. Adding a
//! level must require touching only this file. The structural levels
//! (`STRUCTURES`) add `S` support, `=` beam, `|` tie and `K` keystone; in
//! them `X` is an anchor (docs/design/structural-gameplay.md).
//!
//! ponytail: nothing in `game.rs` calls `parse_level`/reads `LEVELS` yet —
//! that lands with level load/progression (M3, arkanoid-5rs). Module-wide
//! `#[allow(dead_code)]` until then; delete it once M3 wires this module in.
#![allow(dead_code)]

/// Brick footprint in logical playfield pixels (spec: 52x22, grid up to
/// 14x8).
pub const BRICK_WIDTH: f32 = 52.0;
pub const BRICK_HEIGHT: f32 = 22.0;

/// What a brick is made of. Scoring (10-70 pts by row) and hit-count
/// behavior (armored takes 2 hits) belong to `game.rs`, not here.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BrickKind {
    Normal,
    Armored,
    /// Steel. Never breaks, and in a structural level it is an anchor:
    /// what everything else holds on to.
    Indestructible,
    /// A load-bearing brick (structural levels): two hits, and it carries
    /// more than any other.
    Support,
    /// A thin girder (structural levels): one hit, and it holds only the
    /// bricks to its left and right.
    Beam,
    /// A thin tie (structural levels): one hit, and it holds only the
    /// bricks above and below it.
    Tie,
    /// The level's objective (structural levels). Nothing that strikes it
    /// can break it: it comes down when nothing holds it any more.
    Keystone,
}

impl BrickKind {
    /// Whether a ball, a bolt, a blast or falling debris can damage it.
    pub fn breakable(self) -> bool {
        !matches!(self, Self::Indestructible | Self::Keystone)
    }
}

/// One brick to spawn: top-left corner in grid-local pixels (col *
/// BRICK_WIDTH, row * BRICK_HEIGHT — grid origin at (0, 0)) plus its kind.
/// The consumer (`game.rs`) is responsible for translating this into the
/// 800x600 playfield (e.g. centering the grid), since that placement is a
/// gameplay/layout concern, not a level-data concern.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BrickSpawn {
    pub x: f32,
    pub y: f32,
    pub kind: BrickKind,
}

/// Parse a level grid into brick spawns. `.` cells are skipped; any other
/// character maps to a `BrickKind` per the module doc. Row/column index
/// order matches the grid's reading order (row 0 = top).
pub fn parse_level(grid: &[&str]) -> Vec<BrickSpawn> {
    grid.iter()
        .enumerate()
        .flat_map(|(row, line)| {
            line.chars().enumerate().filter_map(move |(col, ch)| {
                let kind = match ch {
                    '1' => Some(BrickKind::Normal),
                    '2' => Some(BrickKind::Armored),
                    'X' => Some(BrickKind::Indestructible),
                    'S' => Some(BrickKind::Support),
                    '=' => Some(BrickKind::Beam),
                    '|' => Some(BrickKind::Tie),
                    'K' => Some(BrickKind::Keystone),
                    _ => None,
                };
                kind.map(|kind| BrickSpawn {
                    x: col as f32 * BRICK_WIDTH,
                    y: row as f32 * BRICK_HEIGHT,
                    kind,
                })
            })
        })
        .collect()
}

/// Level 1: simple rows, all normal bricks.
pub const LEVEL_1: &[&str] = &[
    "11111111111111",
    "11111111111111",
    "11111111111111",
    "11111111111111",
];

/// Level 2: introduces armored bricks (rows 0 and 3).
pub const LEVEL_2: &[&str] = &[
    "22222222222222",
    "11111111111111",
    "11111111111111",
    "22222222222222",
    "11111111111111",
];

/// Level 3: indestructible bricks shape a corridor — ceiling/floor walls,
/// side walls, and a clear lane (row 3) the ball can travel through. The
/// floor has a four-brick gate in the middle: the only way in, so every
/// destructible brick is reached through it.
pub const LEVEL_3: &[&str] = &[
    "XXXXXXXXXXXXXX",
    "X111111111111X",
    "X122222222221X",
    "X1..........1X",
    "X111111111111X",
    "XXXXX....XXXXX",
];

/// All built-in levels, in play order.
pub const LEVELS: &[&[&str]] = &[LEVEL_1, LEVEL_2, LEVEL_3];

// -- the campaign -------------------------------------------------------------
//
// What a normal game plays: 32 levels (staircases, frames, columns, walls of
// armored and indestructible bricks), all original designs: this repo never
// reproduces another game's layouts (see ATTRIBUTION). `LEVELS` above stays
// v1's three: the tests play them, and so does `--classic`.

/// The campaign, in play order, each with its name.
pub const CAMPAIGN: &[(&str, &[&str])] = &[
    (
        "Warm-up",
        &[
            "..............",
            ".111111111111.",
            ".111111111111.",
            ".111111111111.",
        ],
    ),
    (
        "Cascade",
        &[
            "..........1111",
            ".........11112",
            "........11112.",
            ".......11112..",
            "......11112...",
            ".....11112....",
            "....11112.....",
            "...11112......",
        ],
    ),
    (
        "Checkers",
        &[
            "1.1.1.1.1.1.1.",
            ".1.1.1.1.1.1.1",
            "2.2.2.2.2.2.2.",
            ".2.2.2.2.2.2.2",
            "1.1.1.1.1.1.1.",
            ".1.1.1.1.1.1.1",
        ],
    ),
    (
        "Twin towers",
        &[
            ".1111....1111.",
            ".1221....1221.",
            ".1221....1221.",
            ".1221....1221.",
            ".1221....1221.",
            ".1111....1111.",
        ],
    ),
    (
        "Arrow",
        &[
            "......11......",
            "......11......",
            "......11......",
            "......11......",
            "..1...11...1..",
            "...1..11..1...",
            "....1.11.1....",
            ".....1111.....",
        ],
    ),
    (
        "Capped columns",
        &[
            "X.X.X.X.X.X.X.",
            "1.1.1.1.1.1.1.",
            "1.1.1.1.1.1.1.",
            "2.2.2.2.2.2.2.",
            "1.1.1.1.1.1.1.",
            "1.1.1.1.1.1.1.",
        ],
    ),
    (
        "Diamond",
        &[
            "......22......",
            ".....2112.....",
            "....211112....",
            "...21111112...",
            "...21111112...",
            "....211112....",
            ".....2112.....",
            "......22......",
        ],
    ),
    (
        "Scatter",
        &[
            "1...2...1...2.",
            "..1...2...1...",
            "2...1...2...1.",
            "..2...1...2...",
            "1...2...1...2.",
            "..1...2...1...",
        ],
    ),
    (
        "Picture frame",
        &[
            "X111111111111X",
            "1............1",
            "1.2222222222.1",
            "1.2111111112.1",
            "1.2222222222.1",
            "1............1",
            "21111111111112",
        ],
    ),
    (
        "Heart",
        &[
            "..111....111..",
            ".11111..11111.",
            "11111111111111",
            "11111111111111",
            ".111111111111.",
            "..1111111111..",
            "....111111....",
            "......11......",
        ],
    ),
    (
        "Double W",
        &[
            "2............2",
            "12..........21",
            ".12...22...21.",
            "..12.2112.21..",
            "...121..121...",
            "....1....1....",
        ],
    ),
    (
        "Fortress",
        &[
            "XXXXX....XXXXX",
            "X111X....X111X",
            "X121X....X121X",
            "X111X....X111X",
            "X222X....X222X",
        ],
    ),
    (
        "Lattice",
        &[
            "1.1.1.1.1.1.1.",
            "11111111111111",
            "X1X1X1X1X1X1X1",
            "11111111111111",
            "1.1.1.1.1.1.1.",
        ],
    ),
    (
        "Wave",
        &[
            "1....11....1..",
            "11..1111..11..",
            "111111111111..",
            ".111111111111.",
            "..11..1111..11",
            "..1....11....1",
        ],
    ),
    (
        "Rocket",
        &[
            "......22......",
            ".....2112.....",
            ".....1111.....",
            ".....1111.....",
            "....111111....",
            "...11.11.11...",
            "..1...11...1..",
        ],
    ),
    (
        "Bridges",
        &[
            "22222222222222",
            "11111111111111",
            ".1X..2112..X1.",
            "..X........X..",
            "..X........X..",
        ],
    ),
    (
        "Spiral",
        &[
            "11111111111111",
            "1............1",
            "1.1111111111.1",
            "1.1........1.1",
            "1.1.111111.1.1",
            "1.1.1......1.1",
            "1.1.11111111.1",
            "1.1..........1",
        ],
    ),
    (
        "Pillars",
        &[
            "22.22.22.22.22",
            "11.11.11.11.11",
            "11.11.11.11.11",
            "11.11.11.11.11",
            "XX.XX.XX.XX.XX",
        ],
    ),
    (
        "Crown",
        &[
            "1..1..11..1..1",
            "11.11.11.11.11",
            "11111111111111",
            "12121212121212",
            "11111111111111",
        ],
    ),
    (
        "Hourglass",
        &[
            "11111111111111",
            ".111111111111.",
            "..1111111111..",
            "....222222....",
            "....222222....",
            "..1111111111..",
            ".111111111111.",
            "11111111111111",
        ],
    ),
    (
        "Shield",
        &[
            "..2222222222..",
            ".211111111112.",
            ".211111111112.",
            "..2111111112..",
            "...21111112...",
            "XXXX.2112.XXXX",
        ],
    ),
    (
        "Armored frame",
        &[
            "22222222222222",
            "2............2",
            "2.1111111111.2",
            "2.1111111111.2",
            "2............2",
            "22222222222222",
        ],
    ),
    (
        "Rain",
        &[
            "1.1.1.1.1.1.1.",
            "1...1...1...1.",
            "..1...1...1...",
            ".1.1.1.1.1.1.1",
            "2...2...2...2.",
            "..2...2...2...",
        ],
    ),
    (
        "Funnel",
        &[
            "22222222222222",
            ".111111111111.",
            "..1111111111..",
            "...11111111...",
            "....111111....",
            ".....1111.....",
            "......11......",
        ],
    ),
    (
        "Two rooms",
        &[
            "XXXXXXXXXXXXXX",
            "X11111XX11111X",
            "X12221XX12221X",
            "X11111XX11111X",
            "X.....XX.....X",
        ],
    ),
    (
        "Ring",
        &[
            "....111111....",
            "..11......11..",
            ".1..222222..1.",
            ".1..2....2..1.",
            ".1..222222..1.",
            "..11......11..",
            "....111111....",
        ],
    ),
    (
        "Baffles",
        &[
            "11111111111111",
            "22222222222222",
            "11111111111111",
            "..............",
            "XXX...XX...XXX",
            "..............",
            "...XX.....XX..",
        ],
    ),
    (
        "Chevron",
        &[
            "2............2",
            "12..........21",
            ".12........21.",
            "..12......21..",
            "...12....21...",
            "....12..21....",
            ".....1221.....",
        ],
    ),
    (
        "Split wall",
        &[
            "11111XXXX11111",
            "11111....11111",
            "22222....22222",
            "11111....11111",
            "11111XXXX11111",
        ],
    ),
    (
        "Bulwark",
        &[
            "22222222222222",
            "21111111111112",
            "21111111111112",
            "21111111111112",
            "21111111111112",
            "22222222222222",
        ],
    ),
    (
        "Minefield",
        &[
            "1.2.1.X.1.2.1.",
            ".X.1.2.1.2.1.X",
            "2.1.X.1.2.1.2.",
            ".1.2.1.X.1.2.1",
            "X.2.1.2.1.X.1.",
        ],
    ),
    (
        "Vault",
        &[
            "XXXXXXXXXXXXXX",
            "X212121212121X",
            "X121212121212X",
            "X212121212121X",
            "X121212121212X",
            "X............X",
        ],
    ),
];

// -- the structures -----------------------------------------------------------
//
// Levels built to be brought down, not cleared (docs/concept.md,
// docs/design/structural-gameplay.md). `X` is an anchor: every other brick
// stands only while a chain of touching bricks leads from it to one. `S`
// carries the most, `=` holds sideways and `|` up and down only, and `K`,
// the keystone, is the objective: no strike breaks it, it falls when
// nothing holds it. Up to 14 columns by 14 rows.
//
// `game.rs` checks every one of them: all of it holds at the start, nothing
// starts overloaded, and the keystone touches no anchor.

/// One structural level: its name, the line the serve prompt shows under
/// it, and its grid.
#[derive(Debug, Clone, Copy)]
pub struct StructureLevel {
    pub name: &'static str,
    pub hint: &'static str,
    pub grid: &'static [&'static str],
}

/// The structural levels, in play order: 32, in four acts of eight, as
/// many as the campaign. The first two are lessons. From the third on, no
/// single brick brings the keystone down (`game.rs` checks it), so a level
/// is a sequence of cuts, and which cut comes first matters. Each act ends
/// on one of the larger structures: the Crane, the Bridge, the Gantry, the
/// Citadel.
///
/// All of them are original. The shapes some take (an eye, a flower, a
/// face, a lattice) are the kind brick-breaker levels have always drawn;
/// none is copied from another game or from a fetched levelset.
pub const STRUCTURES: &[StructureLevel] = &[
    // -- act I ---------------------------------------------------------------
    // 1. Two rows of beams hold a block between two pillars. Cut either
    // row and the other cannot carry the block alone: it cracks, gives
    // way, and the block comes down with the keystone in it. An unaimed
    // first serve lands on the right-hand beams.
    StructureLevel {
        name: "First fall",
        hint: "CUT A BEAM",
        grid: &[
            "X............X",
            "X....1111....X",
            "X...111111...X",
            "X===11K111===X",
            "X...111111...X",
            "X....1111....X",
        ],
    },
    // 2. A table on two legs, the keystone set in its top. A leg takes
    // two hits; the other then gives way under the top.
    StructureLevel {
        name: "Table",
        hint: "BREAK A LEG",
        grid: &[
            "..............",
            "...11111111...",
            "...111K1111...",
            "...S......S...",
            "...S......S...",
            "...X......X...",
        ],
    },
    // 3. A bob on two ties. Either one holds it.
    StructureLevel {
        name: "Pendulum",
        hint: "BOTH TIES",
        grid: &[
            ".....X.X.....",
            ".....|.|.....",
            ".....1K1.....",
            "......2......",
        ],
    },
    // 4. Hung three ways: a tie above, a row of beams each side. On one, it is too heavy.
    StructureLevel {
        name: "Chandelier",
        hint: "TWO OF THREE HOLDS",
        grid: &[
            "......X......",
            "......|......",
            "X====1K1====X",
            ".....2.2.....",
        ],
    },
    // 5. A column with the keystone at its head, braced to two posts by a
    // beam each side. Cut the column and its top hangs from the braces;
    // cut one brace and the other carries what hangs. Cut the column low
    // and that is more than a beam can hold. Cut it high and both braces
    // have to go. The column is armored: a hanging chain of plain bricks
    // would tear under its own length.
    StructureLevel {
        name: "Tower",
        hint: "CUT IT LOW",
        grid: &[
            "....X.1.X....",
            "....X=K=X....",
            "....X.2.X....",
            "......2......",
            "......2......",
            "......2......",
            "......2......",
            "......S......",
            "......S......",
            "......X......",
        ],
    },
    // 6. A gate in a steel frame, on three hangers. It stays up on two.
    StructureLevel {
        name: "Portcullis",
        hint: "TWO HANGERS",
        grid: &[
            "XXXXXXXXXXXXX",
            "X.S...S...S.X",
            "X.222222222.X",
            "X.|.|.|.|.|.X",
            "X.1111K1111.X",
        ],
    },
    // 7. A lintel on two piers, all of supports, the keystone at its
    // middle. One pier gone, the other holds the lintel out over nothing:
    // a support carries twelve. Crack one that carries it and it holds
    // six.
    StructureLevel {
        name: "Arch",
        hint: "WEAKEN WHAT IS LEFT",
        grid: &[
            ".............",
            "...SSSKSSS...",
            "...S.....S...",
            "...S.....S...",
            "...S.....S...",
            "...X.....X...",
        ],
    },
    // 8. A girder on two hangers, the keystone slung under it. One hanger carries it all, just.
    StructureLevel {
        name: "Crane",
        hint: "CRACK THE LAST HANGER",
        grid: &[
            "...X.....X...",
            "...S.....S...",
            "...SSSSSSS...",
            ".....|.|.....",
            ".....1K1.....",
        ],
    },
    // -- act II ---------------------------------------------------------------
    // 9. A cage on three ties under a steel bar. The middle tie is behind the others.
    StructureLevel {
        name: "Lantern",
        hint: "CUT TWO TIES",
        grid: &[
            "....XXXXX....",
            "....|.|.|....",
            "....22222....",
            "....2.K.2....",
            "....22222....",
        ],
    },
    // 10. A diamond on a hanger and two rows of beams. Any two gone and it falls.
    StructureLevel {
        name: "Kite",
        hint: "CUT THE STRINGS",
        grid: &[
            "......X......",
            "......S......",
            ".....111.....",
            "X===11K11===X",
            ".....111.....",
            "......1......",
        ],
    },
    // 11. Three spans on four piers under a channel. The keystone is the middle span's.
    StructureLevel {
        name: "Aqueduct",
        hint: "FREE THE MIDDLE SPAN",
        grid: &[
            "1111111111111",
            "S222S2K2S222S",
            "S...S...S...S",
            "S...S...S...S",
            "X...X...X...X",
        ],
    },
    // 12. Two arms with a pan each, and the keystone slung between two ties.
    StructureLevel {
        name: "Mobile",
        hint: "UNDER THE TIES",
        grid: &[
            ".....XXX.....",
            ".....SSS.....",
            "..1==SSS==1..",
            "..|..|.|..|..",
            "..2..1K1..2..",
        ],
    },
    // 13. A ring pinned at both ends and hung by a tie, the keystone its pupil.
    StructureLevel {
        name: "Eye",
        hint: "TWO OF THREE",
        grid: &[
            "......X......",
            "......|......",
            "....11S11....",
            "..111.2.111..",
            "X1S..2K2..S1X",
            "..111.2.111..",
            "....11S11....",
        ],
    },
    // 14. Two heaps meeting at the keystone. It is held from every side.
    StructureLevel {
        name: "Hourglass",
        hint: "CLEAR THE WAIST",
        grid: &[
            "XX111111111XX",
            "...1111111...",
            "....11111....",
            ".....1K1.....",
            "....11111....",
            "...1111111...",
            "XX111111111XX",
        ],
    },
    // 15. A balance on a post, hung from above too. Either holds it, until it is cracked.
    StructureLevel {
        name: "Scales",
        hint: "ONE HOLD, THEN CRACK IT",
        grid: &[
            "......X......",
            "......S......",
            "......S......",
            "......S......",
            "....1=K=1....",
            "....2.S.2....",
            "......S......",
            "......X......",
        ],
    },
    // 16. A slab hung by three ties over a deck that carries the keystone.
    // The deck sits on steel: no ball reaches it from below, and the two
    // bricks that hold the keystone stand whatever else goes. The slab,
    // dropped whole, breaks them both. It stays up on any two ties.
    StructureLevel {
        name: "Bridge",
        hint: "DROP THE SLAB WHOLE",
        grid: &[
            "..X...X...X..",
            "..|...|...|..",
            "..111111111..",
            "..111111111..",
            ".............",
            ".............",
            ".XSSSSKSSSSX.",
            ".XXXXX.XXXXX.",
        ],
    },
    // -- act III ---------------------------------------------------------------
    // 17. Three roofs on two posts, the keystone between the posts.
    StructureLevel {
        name: "Pagoda",
        hint: "BOTH SIDES OF IT",
        grid: &[
            ".....222.....",
            ".....S.S.....",
            "....=S.S=....",
            ".....SKS.....",
            "X2222S.S2222X",
            ".....S.S.....",
            ".....X.X.....",
        ],
    },
    // 18. Hung from above and standing on its stem. Either carries it, until it is cracked.
    StructureLevel {
        name: "Flower",
        hint: "STEM, THEN CRACK",
        grid: &[
            "......X......",
            "......S......",
            ".....1S1.....",
            "....11K11....",
            ".....1S1.....",
            "......S......",
            "......S......",
            "......X......",
        ],
    },
    // 19. Four arms to four anchors. The last one cannot carry the hub.
    StructureLevel {
        name: "Windmill",
        hint: "THREE OF FOUR",
        grid: &[
            "......X......",
            "......|......",
            ".....111.....",
            "X====1K1====X",
            ".....111.....",
            "......|......",
            "......X......",
        ],
    },
    // 20. Two rails, three rungs, and the keystone in the middle one. The way in is up the shaft.
    StructureLevel {
        name: "Ladder",
        hint: "THE MIDDLE RUNG",
        grid: &[
            "....X...X....",
            "....S...S....",
            "....S===S....",
            "....S...S....",
            "....S2K2S....",
            "....S...S....",
            "....S===S....",
            "....S...S....",
            "....X...X....",
        ],
    },
    // 21. Three posts and their planks, the keystone at the top of the middle one.
    StructureLevel {
        name: "Scaffold",
        hint: "CUT IT LOW",
        grid: &[
            "..S===K===S..",
            "..S...S...S..",
            "..S===S===S..",
            "..S...S...S..",
            "..S===S===S..",
            "..S...S...S..",
            "..X...X...X..",
        ],
    },
    // 22. A body on four legs and a thread. The keystone is held above and below, inside.
    StructureLevel {
        name: "Spider",
        hint: "ITS BELLY, THEN ITS BACK",
        grid: &[
            "......X......",
            "......|......",
            "...2222222...",
            "...22.K.22...",
            "...2222222...",
            "...|.|.|.|...",
            "...X.X.X.X...",
        ],
    },
    // 23. The keystone in an armored cage, held four ways by supports: two
    // hangers above, a row on each side. Any one of them can carry the
    // cage (its top corners are supports too), unless what is left of the
    // others still hangs on it. A slab
    // closes the vault from below; the way in is over the walls.
    StructureLevel {
        name: "Vault",
        hint: "KEEP IT HEAVY",
        grid: &[
            "X....X.X....X",
            "X....S.S....X",
            "X....S2S....X",
            "XSSSS2K2SSSSX",
            "X....222....X",
            "X...........X",
            "XS111111111SX",
        ],
    },
    // 24. A box hung under a steel beam by three hangers, the keystone in
    // it. It stays up on two. The hangers stand in the gap between the
    // beam and the box: the way in is from the sides, and the middle one
    // is behind the other two.
    StructureLevel {
        name: "Gantry",
        hint: "TWO OF THREE",
        grid: &[
            "..XXXXXXXXX..",
            "...S..S..S...",
            "...S..S..S...",
            "..222222222..",
            "..2111K1112..",
            "..222222222..",
        ],
    },
    // -- act IV ---------------------------------------------------------------
    // 25. A heavy band on three hangers, the keystone its jewel.
    StructureLevel {
        name: "Crown",
        hint: "TWO OF THREE",
        grid: &[
            "..X...X...X..",
            "..S...S...S..",
            "..S...S...S..",
            "..222222222..",
            "..2222K2222..",
            "..222222222..",
        ],
    },
    // 26. A face pinned by its eyes. The keystone is behind the mouth.
    StructureLevel {
        name: "Mask",
        hint: "THROUGH THE MOUTH",
        grid: &[
            "..222222222..",
            "..2X22222X2..",
            "..2222K2222..",
            "..22.....22..",
            "..2.11111.2..",
            "..222222222..",
        ],
    },
    // 27. A disc on three hangers, the keystone under the middle one.
    StructureLevel {
        name: "Saucer",
        hint: "TWO HANGERS",
        grid: &[
            "...X..X..X...",
            "...S..S..S...",
            "...S22K22S...",
            "..111111111..",
            "...|..|..|...",
            "...1..1..1...",
        ],
    },
    // 28. A net of beams and ties. The keystone hangs on four of them.
    StructureLevel {
        name: "Lattice",
        hint: "FOUR THREADS",
        grid: &[
            "X=1=1=1=1=1=X",
            "..|.|.|.|.|..",
            "X=1=1=K=1=1=X",
            "..|.|.|.|.|..",
            "X=1=1=1=1=1=X",
        ],
    },
    // 29. A deck hinged on the wall and held by two chains, the keystone at its far end.
    StructureLevel {
        name: "Drawbridge",
        hint: "THE CHAIN AND THE DECK",
        grid: &[
            "X.....X....X.",
            "X.....|....S.",
            "X.....|....S.",
            "X2222222222K.",
        ],
    },
    // 30. A curtain wall between two towers, the keystone at its heart.
    StructureLevel {
        name: "Keep",
        hint: "BREACH THE CURTAIN",
        grid: &[
            "SS.........SS",
            "SS111121111SS",
            "SS1112K2111SS",
            "SS111121111SS",
            "SS.........SS",
            "XX.........XX",
        ],
    },
    // 31. The core on four arms in a steel shell. Two slots in the floor are the way in.
    StructureLevel {
        name: "Reactor",
        hint: "IN THROUGH THE SLOTS",
        grid: &[
            "XXXXXXXXXXXXX",
            "X.....S.....X",
            "X.....S.....X",
            "XSSSS2K2SSSSX",
            "X.....S.....X",
            "X.....S.....X",
            "XXX.XXXXX.XXX",
        ],
    },
    // 32. The cage again, on a pedestal, braced to the walls, under an
    // armored roof hung by two ties and over a curtain wall. The roof,
    // dropped, cracks everything it lands on. The pedestal (and the
    // support the cage sits on it by) carries the cage alone, and not the
    // cage with its braces still hanging on it.
    StructureLevel {
        name: "Citadel",
        hint: "BRING IT ALL DOWN",
        grid: &[
            "X.X.......X.X",
            "X.|.......|.X",
            "X.222222222.X",
            "X...........X",
            "X....222....X",
            "XSSSS2K2SSSSX",
            "X....2S2....X",
            "X.....S.....X",
            "X.....S.....X",
            "XS1111X1111SX",
        ],
    },
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn level_1_row_is_all_normal_bricks_at_expected_positions() {
        let spawns = parse_level(&[LEVEL_1[0]]);
        assert_eq!(spawns.len(), 14);
        assert!(spawns.iter().all(|b| b.kind == BrickKind::Normal));
        assert_eq!(
            spawns[0],
            BrickSpawn {
                x: 0.0,
                y: 0.0,
                kind: BrickKind::Normal
            }
        );
        assert_eq!(
            spawns[13],
            BrickSpawn {
                x: 13.0 * BRICK_WIDTH,
                y: 0.0,
                kind: BrickKind::Normal
            }
        );
    }

    #[test]
    fn level_2_top_row_is_all_armored() {
        let spawns = parse_level(&[LEVEL_2[0]]);
        assert_eq!(spawns.len(), 14);
        assert!(spawns.iter().all(|b| b.kind == BrickKind::Armored));
    }

    #[test]
    fn level_3_top_row_is_all_indestructible() {
        let spawns = parse_level(&[LEVEL_3[0]]);
        assert_eq!(spawns.len(), 14);
        assert!(spawns.iter().all(|b| b.kind == BrickKind::Indestructible));
    }

    #[test]
    fn level_3_floor_has_a_gate_into_the_box() {
        // Without it the box is closed on all four sides and its
        // destructible bricks can never be hit, so the level (and the game)
        // could not be won.
        let floor = parse_level(&[LEVEL_3[LEVEL_3.len() - 1]]);
        assert_eq!(floor.len(), 10);
        assert!(floor.iter().all(|b| b.kind == BrickKind::Indestructible));
        let gate: Vec<f32> = (5..9).map(|col| col as f32 * BRICK_WIDTH).collect();
        assert!(floor.iter().all(|b| !gate.contains(&b.x)));
    }

    #[test]
    fn level_3_corridor_row_mixes_kinds_and_skips_empty_cells() {
        // "X1..........1X" -> walls + normal at the ends, dots skipped.
        let spawns = parse_level(&[LEVEL_3[3]]);
        assert_eq!(spawns.len(), 4);
        assert_eq!(
            spawns[0],
            BrickSpawn {
                x: 0.0,
                y: 0.0,
                kind: BrickKind::Indestructible
            }
        );
        assert_eq!(
            spawns[1],
            BrickSpawn {
                x: BRICK_WIDTH,
                y: 0.0,
                kind: BrickKind::Normal
            }
        );
        assert_eq!(
            spawns[2],
            BrickSpawn {
                x: 12.0 * BRICK_WIDTH,
                y: 0.0,
                kind: BrickKind::Normal
            }
        );
        assert_eq!(
            spawns[3],
            BrickSpawn {
                x: 13.0 * BRICK_WIDTH,
                y: 0.0,
                kind: BrickKind::Indestructible
            }
        );
    }

    /// Every built-in grid: v1's three and the campaign.
    fn all_grids() -> impl Iterator<Item = &'static [&'static str]> {
        LEVELS
            .iter()
            .copied()
            .chain(CAMPAIGN.iter().map(|(_, grid)| *grid))
    }

    #[test]
    fn every_built_in_level_can_be_cleared() {
        // A ball can reach a brick if a path of cells leads to it from the
        // open field around the grid, through empty cells and through
        // destructible bricks (they break), never through an indestructible
        // one, moving between edge-sharing cells (two indestructible bricks
        // touching at a corner leave no gap a ball fits through). Level 3's
        // first, closed box failed exactly this.
        for (index, grid) in all_grids().enumerate() {
            let rows = grid.len();
            let cols = grid[0].chars().count();
            let cells: Vec<Vec<char>> = grid.iter().map(|r| r.chars().collect()).collect();
            // A ring of open field around the grid: indices shifted by one.
            let open = |r: usize, c: usize| {
                r == 0 || c == 0 || r > rows || c > cols || cells[r - 1][c - 1] != 'X'
            };
            let mut seen = vec![vec![false; cols + 2]; rows + 2];
            let mut stack: Vec<(usize, usize)> = vec![(0, 0)];
            seen[0][0] = true;
            while let Some((r, c)) = stack.pop() {
                let neighbours = [
                    (r.wrapping_sub(1), c),
                    (r + 1, c),
                    (r, c.wrapping_sub(1)),
                    (r, c + 1),
                ];
                for (nr, nc) in neighbours {
                    if nr <= rows + 1 && nc <= cols + 1 && !seen[nr][nc] && open(nr, nc) {
                        seen[nr][nc] = true;
                        stack.push((nr, nc));
                    }
                }
            }
            for (r, row) in cells.iter().enumerate() {
                for (c, &cell) in row.iter().enumerate() {
                    assert!(
                        !matches!(cell, '1' | '2') || seen[r + 1][c + 1],
                        "grid {index}: the brick at row {r}, column {c} cannot be reached"
                    );
                }
            }
        }
    }

    /// A breakable brick with an indestructible one above it and another
    /// below it in the same column can only be hit from the side, through
    /// one brick-high lane: the pacing probe found such bricks were always
    /// a level's last 10%, hunted for minutes (docs/design/director-review.md).
    /// The campaign only: v1's three levels are frozen with its replay hash.
    #[test]
    fn no_campaign_brick_sits_between_two_indestructible_ones_in_its_column() {
        for (name, grid) in CAMPAIGN {
            let cells: Vec<Vec<char>> = grid.iter().map(|r| r.chars().collect()).collect();
            for c in 0..cells[0].len() {
                let column: Vec<char> = cells.iter().map(|row| row[c]).collect();
                let (Some(top), Some(bottom)) = (
                    column.iter().position(|&k| k == 'X'),
                    column.iter().rposition(|&k| k == 'X'),
                ) else {
                    continue;
                };
                for (r, &cell) in column.iter().enumerate().take(bottom).skip(top + 1) {
                    assert!(
                        !matches!(cell, '1' | '2'),
                        "{name}: the brick at row {r}, column {c} is sealed between \
                         indestructible bricks above and below"
                    );
                }
            }
        }
    }

    #[test]
    fn the_campaign_has_32_named_levels_each_with_bricks_to_break() {
        assert_eq!(CAMPAIGN.len(), 32);
        for (name, grid) in CAMPAIGN {
            assert!(!name.is_empty());
            assert!(
                parse_level(grid)
                    .iter()
                    .any(|b| b.kind != BrickKind::Indestructible),
                "{name} has nothing to clear"
            );
        }
    }

    #[test]
    fn all_built_in_levels_have_rectangular_rows() {
        for level in all_grids() {
            let width = level[0].chars().count();
            assert!(level.iter().all(|row| row.chars().count() == width));
            assert!(width <= 14);
            assert!(level.len() <= 8);
        }
    }
}
