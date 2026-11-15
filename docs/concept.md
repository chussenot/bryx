# BRYX: the concept

> Don't clear the wall. Bring it down.

BRYX is a brick-breaker in which the bricks hold each other up. A level
is a structure, hung from steel anchors or standing on them. The ball is
still the only thing the player has, and the paddle still aims it. What
changes is the question each shot answers: no longer "which brick can I
reach?" but "which brick is this standing on?".

This page is the idea. The rules are in
[`design/structural-gameplay.md`](design/structural-gameplay.md); the
contract is [`spec.md`](spec.md).

## The one rule

**A brick stands while a chain of touching bricks leads from it to an
anchor.** Everything else follows from that:

- Cut the last chain and what it held falls.
- A brick that is the only thing holding others carries them. Each kind
  carries so many; one carrying more cracks for a moment, then gives way,
  and what it held falls in turn.
- What falls lands on what stands, and breaks it.

So one shot can do three things, each started by the one before: a
support goes, the next one cannot take the weight, and the section that
comes down breaks the one under it.

## What the player does

1. **Reads the structure.** Steel is fixed. A brick that carries others
   wears a mark along its lower edge, longer the nearer it is to its
   limit: the weak point is drawn, not guessed.
2. **Picks the cut.** The brick whose loss brings the most down, not the
   nearest one.
3. **Makes the shot.** The paddle's aim is unchanged: where the ball
   lands on it sets the angle. Thin beams and ties are small targets on
   purpose.
4. **Watches it go.** Orange where it cracks, white where it is about to
   fall, row by row from the bottom. The level ends when the keystone
   comes down.

## The keystone

Every structure holds one keystone, and nothing that strikes it breaks
it. It falls when nothing holds it, and its fall wins the level: the balls
hold still and what is left of the structure comes down after it.

This is the rule that makes the game about structure. A level cannot be
finished by chipping at its objective, so it cannot end on a hunt for a
last brick; and a player who breaks every brick around the keystone has
still freed it, so no level can be left unwinnable.

## Why it is still a brick-breaker

- The paddle, the ball, the angle from the hit position, the speed that
  climbs with each return: unchanged, and tested as before.
- A level takes a minute, not ten. Collapse is fast: under a second from
  a crack to a fall.
- Nothing is simulated that the player cannot predict. There is no
  physics engine and no chance in what falls: the same cut always gives
  the same collapse.

## The rally, the score and the music

A rally is what comes down between two touches of the paddle. A brick
brought down scores twice a brick the ball broke, and counts in the rally
like one, so a cut that drops twenty bricks runs the multiplier to its top
in a second: the short, exact shot beats the long, careful one.

The music already climbed a scale with the rally. A collapse comes back
down it: one step for each row that lets go, from the top of the scale,
resolving on the root when the keystone falls. The player plays the phrase
up, and the structure plays it down.

## What this is not

- Not a physics sandbox. Bricks fall straight down; nothing tumbles,
  stacks or rolls.
- Not an engineering game. There is one number to learn (how much a brick
  can carry), and the game draws it.
- Not a reskin. The campaign's flat walls have no structure and still
  play by their own rules, as does Classic; the structural levels are
  their own mode until the campaign is rebuilt on them, act by act.
