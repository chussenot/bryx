# Structural gameplay

How structural destruction works in BRYX: the model, the rules, what the
simulation emits and what the player sees and hears. The idea behind it is
in [`../concept.md`](../concept.md).

Status: playable, second version ([`structures-v2.md`](structures-v2.md)
is the review that led to it). 32 levels in four acts (`levels::STRUCTURES`), their own
menu row (Structures) and flag (`--structures`), both renderers, the
sound director, the HUD, the autopilot and the playtest log. The campaign
is not converted yet (see "Next").

![First fall: one beam cut, the other side cracking, the block unstable, the block falling](structures.png)

*First fall, played by the autopilot. One beam is cut (top left: the
beams and the brick on the other side glow as they crack); they give way
and the block turns white, about to fall; it lets go row by row from the
bottom; the keystone's fall ends the level.*

## 1. The model

A structure is a graph on the level's grid (`src/structure.rs`). Nothing
in it is a physics body.

| Term | Meaning |
|---|---|
| Anchor | An indestructible brick (`X`). Fixed to the world. |
| Bond | Two bricks whose cells share an edge hold each other. A beam bonds only left and right, a tie only up and down. |
| Held | A chain of bonds leads from the brick to an anchor. |
| Load | The number of bricks that would stop being held if this brick alone were gone. |
| Capacity | How many bricks a brick can carry: by kind, and by the hits it has left. |

Two consequences worth stating, because levels are built on them:

- **Two holds share, and neither carries.** A block held from both sides
  has no brick whose loss drops it, so every load in it is zero. Cut one
  side and the other side's bricks each carry the whole block.
- **A neck forms wherever the ball makes one.** Any brick left as the only
  link to a section carries that section. A normal brick carries four, so
  a section of five or more hanging by one normal brick comes down by
  itself. Collapses are not limited to the bricks a level's author marked.

### Brick kinds

| Kind | Char | Hits | Carries | Notes |
|---|---|---|---|---|
| Normal | `1` | 1 | 4 | |
| Armored | `2` | 2 | 3 per hit left | Tough, not strong. |
| Support | `S` | 2 | 6 per hit left | A cracked support holds half. |
| Beam | `=` | 1 | 6 | 52×8 px. Bonds left and right only. |
| Tie | `\|` | 1 | 6 | 10×22 px. Bonds up and down only. |
| Keystone | `K` | none | no limit | Nothing that strikes it breaks it. |
| Anchor | `X` | none | no limit | Steel. |

The numbers live in one table, `structure::RULES`, with the timings
below. Balancing is a change to that table.

## 2. The lifecycle

```
Intact ─▶ Stressed ─▶ Overloaded ─▶ gives way ─▶ destroyed
                          │
   (what it held) ────────┴─▶ Unstable ─▶ falling ─▶ lands or leaves
```

| State | When | What happens | Shown as |
|---|---|---|---|
| Intact | Held, carrying half its capacity or less | Nothing | A load mark if it carries anything |
| Stressed | Held, carrying more than half | Nothing yet | A longer load mark |
| Overloaded | Carrying more than its capacity | Cracks for 0.8 s, then gives way (breaks where it stands) | A full mark and an orange glow that strengthens and beats faster |
| Unstable | No chain to an anchor | Shakes for 0.3 s, then lets go; each row above the lowest 0.05 s later | White-hot, shaking |
| Falling | Let go | Falls straight down at 1500 px/s², up to 640 px/s | The brick, dimmed |
| Landed | It meets a standing brick | It shatters; the brick it hit takes one hit | Dust, and the hit brick's crack or break |
| On the paddle | It meets the paddle, before the keystone is down | It shatters; the paddle cannot move for 0.75 s | Sparks, the paddle in the overload's orange |

The rows of a section let go bottom-up, so it peels away and no falling
brick overtakes another.

### What is deterministic, and what bounds it

- The graph is surveyed only when the number of bricks changes (bricks
  only ever leave a level), never per tick or per frame.
- One survey floods the grid once for the held set and once per brick
  with two bonds or more for its load: at most 196 floods of 196 cells,
  on arrays on the stack.
- A cascade advances one step a tick. A brick that goes this tick is seen
  by the next tick's survey. There is no recursion and no event that
  triggers another within a tick.
- No randomness: a brick brought down rolls no power-up, and nothing in
  the structure reads a generator.
- Timings accumulate the fixed tick, so they are the same on every
  machine and at every frame rate.

`game.rs` has the tests: the same cut gives the same events and score; a
vault broken at random, forty times, never rests with a brick hanging on
nothing or over its limit.

## 3. The keystone and the end of a level

- A ball, a bolt, a blast, a meteorite and a falling brick all leave the
  keystone whole. A ball bounces off it, metal or not.
- It is not an anchor. A level may not put one beside an anchor (tested):
  it could never fall.
- When it lets go, `KeystoneFell` is emitted, the level is won and every
  brick still standing lets go after it.
- While that finale runs the balls hold still, so the level's last second
  cannot cost a life. The level ends when nothing is left standing or
  falling.
- A level is over, in general, when no brick but steel remains and
  nothing is falling. Breaking everything around the keystone therefore
  also wins.

## 4. Interactions with the rest of the game

| Case | Behaviour |
|---|---|
| Multiball | Every ball breaks bricks as usual; the structure reacts to the bricks, not to who broke them. |
| Explosive ball | The blast breaks what it reaches; the keystone and steel are untouched. A blast can cut several holds at once. |
| Metal ball | Flies through breakable bricks; turned back by steel and by the keystone. |
| Lasers, meteorites | Break bricks as elsewhere; stopped by steel and by the keystone. |
| Simultaneous impacts | Bricks broken in one tick are surveyed together on the next. |
| A life lost during a collapse | The collapse goes on while the next ball waits on the paddle. |
| The level ends during a cascade | It ends when the last debris has landed or left, not before. |
| Break gate | Off in this mode (the capsule never drops): a structure is a problem to solve, and a way around it is not a solution. |
| Asteroids | Off in this mode: the field's moving parts are the bricks coming down. |
| Stall rule | Unchanged. A keystone hit is not progress, so a ball caught between steel and a keystone is still turned out. |

### The paddle under a collapse

Before the keystone falls, a falling brick that lands on the paddle
shatters on it and holds the paddle where it stands for `stun_secs`
(0.75 s). A second one starts the time over; it is not added. The ball is
still in flight, so the cost is whatever the paddle could not reach
meanwhile.

After the keystone falls, the level is won and the paddle **catches**
what lands on it instead. Each caught brick scores again what it scored
when it came down, and the finale's falling bricks are drawn in the
keystone's gold. The same skill, reading where things land, pays in the
finale instead of costing. (An earlier note proposed a bonus for staying
clear of the finale. A structure 13 columns wide leaves 62 px on each
side for a 100 px paddle, so on the widest levels it could not be won.)

- **Every landing is announced.** Both renderers draw a bar where each
  brick that is coming down will land (`Game::fall_zones`): on the
  paddle's line, or on the top of the brick it will crack or break.
  Pale while the brick still shakes in place, orange once it is in the
  air, gold when it is there to catch, and stronger as the landing
  nears. A brick that steel or the keystone will stop shows none.
- **It does not score against you.** No points are lost; time and the
  ball are.
- A lost life and a new level free a held paddle.

So the hazard bites on the collapses that are steps: a floor dropped to
get in (the Trapdoor), a load dropped on a mast (the Crane), a slab's
overhang (the Bridge).

## 5. Scoring

- A brick the ball breaks scores as with rally scoring: its row's value
  times its place in the rally, ×1 to ×10.
- **A collapse belongs to the shot that set it off.** It is one run: it
  starts at the rally of that shot and climbs one step for each brick the
  structure brings down (fallen, crushed, given way), to ×10. The paddle
  touching the ball while it falls does not reset it. Each brick scores
  `collapse_factor` (2) times its value at the run's multiplier.
- The keystone is worth 250 before the multipliers.
- **What the keystone's fall releases scores its plain value.** That fall
  is the level ending, not the player's work. Without this, leaving the
  structure standing for the finale paid more than bringing it down on
  purpose: measured on the Bridge, chipping beside the keystone scored
  27,160 against 21,900 for the planned drop. A test now holds the
  planned cut ahead (`one_cut_outscores_chipping_the_same_structure_down`).
- A brick caught in the finale scores again what it scored coming down.
- No score accrues with time. The mode has no asteroids, so nothing pays
  for keeping the ball up.

The HUD shows the tally of the level (broken, brought down), each
collapse of two bricks or more with the multiplier its run reached, as it
runs and for 1.6 s after, and the finale's catch.

This mode always scores by the rally and keeps one record, apart from the
campaign's.

## 6. Events

Added to `GameEvent`; the simulation emits them, and nothing reads them
back.

| Event | When |
|---|---|
| `BrickOverloaded { rect, load, capacity }` | A brick starts carrying more than it can |
| `BrickGaveWay { rect }` | It breaks under it; its `BrickDestroyedAt` follows |
| `SupportLost { x, y, dependents }` | A brick that carried others is gone |
| `SectionUnstable { count, x, y }` | Bricks lost their hold |
| `BrickFell { rect, kind, nth }` | A brick let go; scored now, no `BrickDestroyedAt` |
| `DebrisLanded { x, y }` | A falling brick landed; the hit brick's `BrickStruck` (by `Striker::Debris`) follows |
| `PaddleStruck { x, y }` | A falling brick landed on the paddle |
| `KeystoneFell { x, y }` | The level is won |
| `DebrisCaught { x, y, nth, points }` | A falling brick landed on the paddle after the keystone fell: caught |
| `CollapseEnded { bricks, score, waves }` | Nothing cracks, shakes or falls any more; `waves` is how many sections let go |

`rally.rs` counts a `BrickFell` as it counts a `BrickDestroyedAt`, so the
scored rally, the note and the light still agree (a property test holds
them together over the structures too).

## 7. Sound

Played on the existing cues, every pitch a step of the playing track's
rally scale (`audio/director.rs`; a test holds every note of a collapse in
the key). No new sample. A collapse is a phrase: tension rising with each
wave, resolving on the root when it is over.

| Event | Sound | Role |
|---|---|---|
| A brick broken by the ball | The rally note, one step up the scale (as before) | The shot |
| Overloaded | The armor's crack an octave down, quiet | Tension |
| A section lets go | A bass note: the fifth, then the seventh, then the seventh an octave up, wave by wave | Rising tension |
| Gives way | The leading tone: the track's seventh, under the root | Wants to resolve |
| A row lets go | One note, down the scale from its top | The fall |
| A falling brick lands | The steel knock, lower | |
| A falling brick lands on the paddle | The steel knock an octave down, loud, with the armor's crack | |
| A brick caught in the finale | Up the scale from the root, one step a catch | The answer |
| The collapse ends | The root, low; for eight bricks or more, the low boom under it | Resolution |
| Keystone falls | The root and its octave; the level fanfare follows | Final resolution |

Each sounds once a tick at most, however many bricks do it together. A
support lost is not heard by itself: the section it lets go is.

Known limit: dedicated voices belong with the studio score (PR #34),
which replaces the cue set this is written against.

## 8. Look

One function, `palette::structure_look`, reads a brick's state and both
renderers draw what it returns, so 2D and 3D say the same thing.

- **Load mark:** a bar along the brick's lower edge (its near edge in
  3D), as long as its share of its limit, and warming from pale to the
  overload's orange past half of it: what is close to failure reads
  before it fails. No bar: it carries nothing.
- **Overloaded:** orange over the brick, strengthening, beating from 3 to
  9 Hz.
- **Unstable:** white-hot, shaking 1.5 px.
- **Falling:** the brick at four fifths of its light; in 3D it tips
  forward as it gathers speed.
- **Where it will land:** a bar on the paddle's line or on the brick it
  will hit, pale, then orange, then gold in the finale, stronger as the
  landing nears (section 4).
- **The finale:** falling bricks in the keystone's gold.
- **A struck paddle:** the overload's orange, easing as it comes free.
- **Reduced motion:** nothing shakes, beats or tips. The glow alone says
  it, steady.

Effects are restrained: a fall has none (the brick is the effect), a
landing throws four sparks, and only the keystone gets the blast's core
and ring.

## 9. The 32 levels

As many as the campaign, in four acts of eight, each act titled and
grounded as the campaign's is (I · Foundations, II · Structures, III ·
Machines, IV · The Vault). Each act ends on one of the larger
structures: the Crane, the Bridge, the Gantry, the Citadel.

**The first act is a curriculum,** one idea a level:

| # | Level | What it teaches |
|---|---|---|
| 1 | First fall | A thin hold carries a heavy block. Won by an unaimed serve. |
| 2 | Table | A support takes two hits; the one left takes it all. |
| 3 | Chandelier | Several holds: it stands on two of three. |
| 4 | Tower | Where you cut decides how much hangs. |
| 5 | Trapdoor | A floor drops across the field: read the warning, then go in. |
| 6 | Hammer | Drop a weight on a support: it cracks, and the load breaks it. Knock off the jib's hanging bricks first and it is too light to. |
| 7 | Arch | A sequence: take one pier, then crack the other. |
| 8 | Crane | All of it: a stay to cut, a load to drop on the mast, its end falling to the field first. |

Three of them teach a single cut (First fall, Table, Hammer). In every
other level no single brick brings the keystone down, so a level is a
sequence of cuts.

**The Bridge** (Act II's finale) is the full loop: cut a tie and the slab
tears loose; its ends fall to the field (a threat), the rest cracks every
deck brick it lands on; a cracked deck cannot hang from one pier, so one
pier then brings it all down. Three hits, where the deck alone takes
four, in either order. Chip the slab where it would have cracked the
deck and that end holds.

Eight levels of acts II to IV are on the review's revise list
([`structures-v2.md`](structures-v2.md), section 6): slow because one
target is hard to reach, or repeating another's idea.

All are original. The shapes some take (an eye, a flower, a face, a
lattice) are the kind brick-breaker levels have always drawn; none is
copied from another game or from a fetched levelset.

| # | Act | Level | Its lesson, as the serve prompt says it | Hold (hits) |
|---|---|---|---|---|
| 1 | I | First fall | Cut a beam | 2 |
| 2 | I | Table | Break a leg | 2 |
| 3 | I | Chandelier | Two of three holds | 3 |
| 4 | I | Tower | Cut it low | 4 |
| 5 | I | Trapdoor | Stand clear, then go in | 2 |
| 6 | I | Hammer | Drop the hammer | 2 |
| 7 | I | Arch | Weaken what is left | 4 |
| 8 | I | Crane | Let the load do it | 4 |
| 9 | II | Portcullis | Two hangers | 3 |
| 10 | II | Kite | Cut the strings | 3 |
| 11 | II | Aqueduct | Free the middle span | 5 |
| 12 | II | Mobile | Under the ties | 2 |
| 13 | II | Eye | Two of three | 3 |
| 14 | II | Hourglass | Clear the waist | 4 |
| 15 | II | Scales | One hold, then crack it | 4 |
| 16 | II | Bridge | Crack the deck, then a pier | 4 |
| 17 | III | Pagoda | Both sides of it | 4 |
| 18 | III | Flower | Stem, then crack | 4 |
| 19 | III | Windmill | Three of four | 4 |
| 20 | III | Ladder | The middle rung | 4 |
| 21 | III | Scaffold | Cut it low | 4 |
| 22 | III | Spider | Its belly, then its back | 4 |
| 23 | III | Vault | Keep it heavy | 6 |
| 24 | III | Gantry | Two of three | 6 |
| 25 | IV | Crown | Two of three | 6 |
| 26 | IV | Mask | Through the mouth | 6 |
| 27 | IV | Saucer | Two hangers | 6 |
| 28 | IV | Lattice | Four threads | 4 |
| 29 | IV | Drawbridge | The chain and the deck | 4 |
| 30 | IV | Keep | Breach the curtain | 6 |
| 31 | IV | Reactor | In through the slots | 8 |
| 32 | IV | Citadel | Bring it all down | 6 |

Each is checked by `game.rs`: all of it holds at the start, nothing starts
over its limit, one keystone, no anchor beside it. Each of the eight
levels above has tests for its intended solution and for the cut that
does not work.

### How hard, measured

Two numbers a test computes for every level
(`the_structures_get_harder_and_only_the_lessons_fall_to_one_brick`),
and one from the autopilot:

- **Hold:** the hits it takes to cut every chain from the keystone to an
  anchor (`Structure::keystone_holds`, a minimum cut). An upper bound on
  the work: a structure often comes down for less, when a cut leaves a
  hold carrying more than it can. Its mean by act: 2.9, 3.5, 4.5,
  5.8.
- **One brick wins:** whether breaking a single brick, however many hits
  it takes, ends the level. True of the three single-cut lessons only,
  found by playing every brick of every level out.
- **Autopilot:** it brings all 32 down on each of three seeds, in 27 to
  32 minutes of play, struck by falling bricks 8 to 16 times
  (`the_heuristic_brings_down_every_structure`, ignored by default: a few
  seconds in release). A pacing probe, not a measure of how hard a person
  finds them.

Writing the levels against these checks caught six that were wrong as
first drawn. Five fell to one brick: a neck of plain bricks tore under
what hung from it, or a post carried more than it could. One had a jaw
attached to nothing. And one of the five, once fixed, took the autopilot
sixteen minutes to reach its hanger, and was redrawn.

What makes a level harder, in order of effect:

1. **More holds than one cut removes.** Three ties, three hangers, four
   holds on a cage: a structure that stands on any two.
2. **Holds strong enough to carry it alone.** A support carries twelve.
   Where the last hold can carry the structure, the player has to crack
   it, or leave more weight hanging on it.
3. **Steel in the way.** A deck no ball reaches from below; hangers in a
   gap entered from the sides; a chamber entered over its walls or
   through two slots in its floor.
4. **No way around.** The Break gate is off in this mode.

## 10. Decisions, and what I did not do

- **The keystone cannot be struck.** The brief asked for a keystone
  "exposed" by destruction. An unbreakable one is the stronger form of
  that: a level cannot be finished without bringing something down, and
  never ends on a hunt.
- **Supports are a kind, load is computed.** The brief listed four
  authored roles. Any brick can end up carrying others, so load is worked
  out from the graph and drawn; the support is the kind built to carry it.
- **No physics engine.** Falling straight down, one hit on landing. It is
  enough for a cascade to cross from one structure to another, and it
  stays exactly reproducible.
- **Stress does not spread from an impact.** A hit weakens what it hits
  (a cracked support carries half); load then does the rest. A second
  channel of "stress" travelling through bonds would be harder to predict
  and to draw, for no decision it adds.
- **A new mode, not a converted campaign.** The campaign's 32 flat walls
  have no anchors; converting them one for one would make 32 identical
  dependency graphs.

## 11. Known limits

- **Difficulty is measured, not felt.** The numbers above say the levels
  ask for more as the acts go on; the order inside an act is a first
  guess. Whether a level is hard or merely slow is for the playtest.
- **The first act and the Bridge are new and unplayed.** They are checked
  by tests for each intended solution, and looked at on screen; no
  person has played them.
- **The autopilot does not find the drops.** It aims a straight line from
  the paddle, so a tie hidden above a hammer or a load is out of its
  reach: on the Hammer and the Crane it brute-forces the mast instead. It
  still wins every level; the tests, not the autopilot, prove the drops.
  One run of the Chandelier took it 562 s (25 s on the other seeds).
- **The hazard is occasional for a precise player** outside the levels
  built for it (the Trapdoor, the Crane, the Bridge, the Scaffold).
- **Capsules can shortcut a level.** A Super laser, a Metal ball or a
  Meteor rain cuts holds faster than any shot. They drop only from bricks
  the ball breaks, so they are rare here; they are not tuned for this
  mode.
- **The autopilot's ball forecast ignores collapse.** It predicts the
  ball against the bricks as they stand; after a section falls the
  forecast is recomputed, so it is late by one bounce, never wrong for
  long.
- **The collapse readout sits mid-field,** so falling bricks pass behind
  it for a moment.
- **Score scale.** A structure pays tens of thousands; the campaign pays
  about two thousand a level. The records are separate, and the numbers
  are untuned.
- **Sound** uses the existing cues (above).

## 12. Next

1. **Human playtest** of the first act and the Bridge (`docs/playtest.md`,
   round C and hypotheses S1 to S6).
2. **The revise list** of acts II to IV, with the playtest's evidence.
3. **Act III as machines:** moving parts designed with the levels that
   need them. A shutter (a steel door on a cycle) was prototyped and set
   aside for it.
4. **Dedicated collapse voices** once the studio score has merged.
