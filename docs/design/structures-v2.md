# Structures v2: design review

A review of the Structures mode as it stands on `master`, and what changes
next. The rules this builds on are in
[`structural-gameplay.md`](structural-gameplay.md); the idea in
[`../concept.md`](../concept.md).

## 1. How this was checked

Not from the docs. The autopilot played all 32 levels on three seeds, and
a probe counted what happened in each level before the keystone fell. Seed
1, which the others match within a few seconds:

| What | Levels where it happened |
|---|---|
| A falling brick broke part of the structure before the keystone fell | **1 of 32** (Bridge, 3 bricks) |
| A collapse finished before the keystone fell (an intermediate collapse) | 13 of 32, never more than 3 in a level |
| A falling brick hit the paddle | 6 of 32, 8 times in all |
| Most of the level's bricks fell in the finale (after the keystone) | 17 of 32 |
| The level took more than 90 s | 6 of 32 (Lantern, Scales, Flower, Spider, Drawbridge, Reactor) |

The slow six are slow because one target is hard to reach (a tie behind a
cage, a hanger between two posts), not because they ask for a plan.

## 2. What works, and stays

- **The model.** Anchors, directional bonds, load against capacity, a
  cracked support carrying half, unheld sections letting go bottom-up,
  falling bricks dealing one hit to what they land on, the keystone. All
  deterministic, bounded and tested. No rule here changes.
- **The basic loop.** The one playtest so far, of the first four
  levels, found the mode fun. The load mark, the orange crack and the
  white shake have not been reported unreadable; they have not been
  tested on someone new either.
- **The paddle hazard as a rule.** A falling brick holds the paddle for
  0.75 s, the floor shows where, the finale is exempt.
- **The checks.** Every level holds untouched, has one keystone, and past
  the lessons cannot be won by one brick.
- Untouched: the ball, the paddle, the campaign, Classic, levelsets, the
  pinned replay, records, both renderers' pipelines, the audio engine.

## 3. What limits it

1. **The levels use one idea.** Nearly every level is "cut the chains
   that hold the keystone". The mechanic that lets a cut have a second
   consequence, a falling section breaking another, is in the engine and
   used by one level.
2. **There is one collapse per level.** The keystone usually falls in the
   same collapse as the first section that comes down, so there is
   nothing to plan around and nothing to survive.
3. **The hazard has nothing to fall from.** Debris before the keystone is
   rare, so the warning and the stun rarely matter.
4. **A collapse is scored by accident.** Its bricks count in the rally of
   whatever shot is in flight when they land. If the ball returns to the
   paddle while a section is still falling, the rest of it starts over at
   ×1: the payoff depends on timing the player did not choose.
5. **The music answers bricks, not the shape of a collapse.** Every
   collapse sounds the same whether it was one wave or three, and the
   "gave way" accent is a semitone outside the key.
6. **The acts are invisible.** The 32 levels are four acts, but the act
   titles, grounds and accents are drawn for the campaign only.
7. **The autopilot cannot see a second consequence.** `what_falls` follows
   overloads and lost holds, not what a falling section breaks, so it
   never plans a drop.
8. **Act III is not about machines.** Its levels are static. That act
   needs moving parts designed for it, not added to it (section 7).

## 4. What changes, and why

A few changes to how the existing loop is read, scored and heard, and a
rebuilt first act. No new brick kind, no new power-up, no rule change to
the structure itself.

### A. Levels built on second consequences

The engine already lets a falling section crack or break what it lands
on, and a cracked support carries half. Levels now use that on purpose.
Three patterns:

| Pattern | The decision it asks for |
|---|---|
| **Drop to crack.** A weight hangs over a support. Dropped, it cracks it; the load then does the rest. | Which to hit: the support twice, or the weight's hanger once. |
| **Clear the path.** A weight hangs over a support, with something in between. Dropped too early, it lands on the shield, not the support. | The order of two cuts. A premature drop is wasted. |
| **Weaken, then cut.** A slab dropped on a deck cracks every brick it lands on. The deck then fails where it would have held. | Combine a drop with one more cut, instead of four direct ones. |

The first act becomes a curriculum (section 6), and the Bridge, the end
of Act II, is rebuilt as the full loop: two cuts, an intermediate
collapse that threatens the paddle, a crack that turns into a give-way, a
keystone collapse.

### B. Where it lands, and when

- **Every landing is shown,** not only the paddle's. The bar under a
  falling brick is drawn where it will land: on the paddle's line, or on
  the top of the brick that will catch it. So a drop shows, while it
  still shakes, which support it is about to crack.
- **The bar says how soon.** It strengthens as the brick nears, so the
  player can tell "now" from "in a second" and time a return around it.

### C. A collapse belongs to the shot that caused it

A collapse is scored as one run, started at the rally of the shot that
set it off and climbing one step per brick to ×10. The paddle no longer
resets it halfway. This is the rally's own rule applied to collapses, not
a new multiplier, and it means:

- one decisive cut scores a whole collapse at its full run;
- a cascade that keeps going (a drop, then a give-way) stays in the same
  run;
- chipping still scores ×1 per brick, so it stays the worst way to play.

The HUD shows the collapse's multiplier beside its count.

### D. The finale is a harvest, not a test

The previous note proposed a bonus for staying clear of the winning
collapse. Measured, that is unfair. A structure 13 columns wide leaves
62 px on each side, and the paddle is 100 px wide. On the widest levels
no clear spot exists, so the bonus would be a punishment by omission.

Instead, once the keystone falls, every falling brick turns the
keystone's gold and can be **caught**: each one that lands on the paddle
scores again what it scored when it fell, and rings the next note up the
scale. It is the same skill, reading where things land. The finale stays
a reward with something to do, and the two kinds of falling brick never
look alike: orange threatens, gold pays.

### E. A collapse is a phrase

The collapse's music, in the track's key only:

| Moment | Sound | Role |
|---|---|---|
| A brick overloads | The crack, an octave down, quiet | Tension |
| A section lets go (each wave) | A bass note: the fifth, then the seventh, then the seventh an octave up | Rising tension, one step a wave |
| A brick gives way | The leading tone (the track's seventh, below the root) | Wants to resolve |
| Rows fall | Down the scale, one note a row | The fall |
| The collapse ends | The root, low: the cadence. A large one adds the low boom | Resolution |
| The keystone falls | Root and octave, then the fanfare | Final resolution |
| A gold brick is caught | Up the scale from the root | The answer |

The semitone accent goes: every note is now in the track's key.

### F. The acts show

The four act titles, grounds and accents now play on the structures as
on the campaign. Act I opens under "I · FOUNDATIONS".

### G. Measuring it

The playtest log gains what the design needs to be judged:

- support cuts before the keystone;
- intermediate collapses;
- bricks the structure broke on itself before the keystone (crushes);
- bricks caught in the finale;
- the longest stretch with nothing broken or brought down.

The autopilot forecasts a cut by playing it out on a copy of the game,
with no ball, so it sees drops and their consequences. It stays a
regression and pacing tool.

## 5. What was not done, and why

- **No clean-collapse bonus.** See D.
- **No new brick kind.** A shutter (a steel door on a cycle) was
  prototyped and set aside. It belongs to Act III designed as machines,
  with levels built around timing. Added to existing levels it only made
  them slower.
- **No moving anchors.** Same reason.
- **No damage scaling with height.** A falling brick already deals one
  hit, and a taller section delivers more of them down the same column.
  Weight already matters; a second rule for it would not be readable.
- **No chain multiplier.** The run in C already pays a longer cascade.

## 6. The levels

Verdicts, from the probe and from reading each level:

| # | Level | Verdict | Why |
|---|---|---|---|
| 1 | First fall | Keep | Lesson: a thin hold carries a heavy block. Won by an unaimed serve. |
| 2 | Table | Keep | Lesson: a support takes two hits, the other gives way. |
| 3 | Pendulum | **Cut** | A second "both holds" lesson; two bricks fall; nothing to remember. |
| 4 | Chandelier | Keep, then **replaced** by the Lintel | Lesson: redundancy, three holds, stands on two. Its third hold was a tie under the ceiling, out of the ball's reach: one run took 562 s, and a player could not bring the keystone down. |
| 5 | Tower | Keep | Lesson: where you cut decides the weight. |
| 6 | Portcullis | **Move** to Act II | The third "three hangers, stands on two"; it opens Act II. |
| 7 | Arch | Keep | Lesson 7: a sequence. |
| 8 | Crane | **Rebuild** | Act I's finale: combine the lessons. |
| 9 | Lantern | **Cut** | 96 s from one hidden tie: a reach problem. |
| 10 | Kite | Keep | |
| 11 | Aqueduct | Keep | 29 of 30 bricks fall in one finale: a spectacle level. |
| 12 | Mobile | Keep | Has an intermediate collapse. |
| 13 | Eye | Keep | |
| 14 | Hourglass | Keep | Has an intermediate collapse. |
| 15 | Scales | Revise | 131 s: reach problem. |
| 16 | Bridge | **Rebuild** | The vertical slice. |
| 17 | Pagoda | Keep | |
| 18 | Flower | Revise | 141 s: reach problem. |
| 19 | Windmill | Keep | |
| 20 | Ladder | Keep | |
| 21 | Scaffold | Keep | Most of it falls before the keystone, onto the paddle's side: the hazard works here. |
| 22 | Spider | Revise | 123 s: reach problem. |
| 23 | Vault | Keep | Three intermediate collapses; the weight lesson. |
| 24 | Gantry | Keep | |
| 25 | Crown | Revise | Same idea as Gantry. |
| 26 | Mask | Keep | |
| 27 | Saucer | Revise | Same idea as Gantry. |
| 28 | Lattice | Keep | |
| 29 | Drawbridge | Revise | 94 s. |
| 30 | Keep | Keep | |
| 31 | Reactor | Revise | 106 s: reach problem. |
| 32 | Citadel | Keep | Its roof drop is a second consequence the autopilot never used. |

**Act I, rebuilt as a curriculum**, one lesson a level:

1. First fall: a thin hold carries a heavy block.
2. Table: a support; the one left takes it all.
3. Lintel (was the Chandelier): several holds, all in the open; it stands on three of four.
4. Tower: where you cut decides how much hangs.
5. **Trapdoor** (new): a floor drops toward the paddle; read the warning, then go in.
6. **Hammer** (new): drop a weight on a support; the crack and the load do the rest.
7. Arch: a sequence, take one pier, then crack the other.
8. **Crane** (rebuilt): the finale, using a drop, a crack and a cut.

To keep 32: Pendulum and Lantern go, and Portcullis opens Act II.

The "revise" levels are the next pass. They are not wrong, just slow or
repetitive; they need a person's play to say which way to take them.

## 7. Next, after this

- A human playtest of Act I and the Bridge (`docs/playtest.md`, round C).
- Act III as machines: moving parts designed with the levels that need
  them. The shutter prototype is the starting point.
- The revise list above.
- Dedicated collapse voices, once the studio score (PR #34) has merged.

## 8. Outcome

What was built, and where it departed from the plan above.

- **All of section 4 is in,** with tests for each rule: the collapse's
  run, the landing marks with their timing, the finale's catch, the
  phrase in key, the acts on the structures, the new log columns, the
  autopilot's playout forecast.
- **A scoring finding changed one rule.** With the run in place, chipping
  beside the Bridge's keystone still outscored the planned drop (27,160
  against 21,900): what the keystone's fall released was scored as
  structural work at x10, so leaving the structure standing paid. Bricks
  the keystone releases now score their plain value, and a test keeps
  the planned cut ahead.
- **Anvil was dropped.** Every version of "clear the path first" let the
  player hit the support directly for the same number of hits, so the
  order never mattered. Arch, which already teaches a sequence, stayed in
  Act I as lesson 7; Portcullis, not cut, opens Act II.
- **Hammer is a single-cut lesson,** like First fall and Table: one
  precise hit on its tie drops the hammer, and the crack and the load do
  the rest. The difficulty test names the three lessons explicitly.
- **Measured after,** autopilot on three seeds, all 32 won each time:
  27 to 32 minutes of play; falling bricks hit the paddle 8 to 16 times a
  run, against 8 before, mostly where the new levels put them (the
  Trapdoor 3 times a run, the Bridge 2). The autopilot still does not
  find the hidden drops (section 11 of the rules page), so in its runs
  falling bricks damage the structure before the keystone on three
  levels (the Tower, the Bridge, the Citadel), against one before; the
  level tests prove the drops it does not find.
