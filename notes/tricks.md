# Tricks: MX vs. ATV Unleashed and Untamed against MX Unleashed (M19)

What the two later games add to MX Unleashed's tricks, how each is driven, and what openmx needs for it.
The tables and clips are packed as trick packs (standard §6, `tools/import-tricks`, built by `build.sh`
into `out/mods/tricks/`); the ELF parts are from the Ghidra projects in `extracted/mxvatv/` (SLUS_211.04
plus its `GAME_REL.BIN` overlay at 0x45c580) and `extracted/untamed/` (SLUS_217.01, overlay at 0x48ea80).
Addresses below are in those images.

**In short.** MX vs. ATV adds five stunts by name — **Varial**, **wheelie**, **stoppie**, **threesixtyL**,
**threesixtyR** — and one new air move, **the 360**, driven by its own torque from each bike's `.phy`
(`[Three Sixty]`). The backflip changes: it now arms only with **R1 held alone**, plays a rider clip, and a
landing mid-clip is a wreck. The 360 is armed the same way, so R1 alone is the rotation button.
**Neither game has a front flip, and a whip is no trick** — it is the stick in the air, MX Unleashed's
air control with weaker auto-levelling. Untamed keeps all of this, the PS2 controls unchanged
(its table's `Classic` mapping), and adds Wii motion paths to the same two steps.

## The tables

MX Unleashed: `appstart.res:rider\anim\stunts.ini`, 57 sections, read by openmx's `trick::Stunts::parse`
(`Name`, `DisplayName`, `SafeTime`, `HoldTime`, `Duration`, `UsedForBike`), the 24 gesture slots an
identity map of the first 24.

| | MX vs. ATV Unleashed | Untamed |
|---|---|---|
| file the loader opens | `StuntInfo.res`: `rider\anim\{mx,mxf,atv}_rider_stunts.ini` (also loose in the PSR, byte for byte) | `res\StuntInfo.res`: `rider\anim\MX_Rider_Stunts.ini.bpb` only (BPB; also loose) |
| loader | `0x2688e0` (per rider set, stride 0x1088) | `0x251e60` |
| sections | `[Stunt01]`…`[Stunt46]` (`Stunt%02d`) | 54 (`Stunt%02d`; BPB titles them `Stunt1`…) |
| record | 0x5c bytes: Name, DisplayName (0x20 each), SafeTime +0x40, HoldTime +0x44, Duration +0x48, LinkModTime +0x4c, PointValue +0x50, HoldPointValue +0x54, index +0x58, UsedForBike +0x59, HasAnimation +0x5a | 0x50 bytes, names 0x1b |
| defaults | clocks 0, **PointValue and HoldPointValue 1000** | the same |
| gesture key | `DefaultKeyMapping` (decoded by `0x267da0`) | `DefaultKeyMappingClassic` (the PS2's; `Style1`–`Style3` are the Wii's) |
| tables | MX, MXF (female rider), ATV; MXF differs only in backflip HoldTime 0.20 (MX 0.60), ATV only in saranwrap SafeTime 1.33 (1.43) and adds `[Depreciation]` (0.8, 4, 4) | one table for every rider set (the ELF names no other) |

Untamed's first 41 rows are MX vs. ATV's (same names, gestures and clocks to one ulp: the text is
truncated to a float by the loader's `strtod` + EE conversion, the BPB holds the nearest float). Rows
42–49 repeat eight earlier stunts with no Classic mapping (the Wii styles' extra gestures); 50–54 are
backflip, wheelie, stoppie, threesixtyL, threesixtyR as on MX vs. ATV.

Scores are new: every stunt has `PointValue` (1000; backflip and the two 360s 2000) and five have
`HoldPointValue` (backflip 10000, threesixtyL/R 15000, wheelie and stoppie 1000). How the HUD turns them
into points (per second held, the `[Depreciation]` repeat penalty) was not traced.

openmx's `Stunts::parse` reads 57 sections, so it refuses all four tables: `stunts.ini has no [stunt47]`
(MX vs. ATV), `no [stunt01]` (Untamed's BPB titles). `tools/import-tricks` reads them as the games'
loaders do (`Table::parse`); `tests/disc.rs` reads each back.

## The tricks MX Unleashed lacks

| stunt | MX vs. ATV / Untamed row | H / S / D (s) | points / hold | gesture | how it moves | clips (MX rider set) |
|---|---|---|---|---|---|---|
| **Varial** | 37 / 37 | 1.10 / 1.72 / 1.84 | 1000 / – | none (remap only) | a pose, no torque | `Rider\Anim\MX_Varial.mot`, `Veh\Anim\bike_Varial.mot` |
| **wheelie** | 43 / 51 | 0 / 0 / 0 | – / 1000 | none | on the ground, no clip (`HasAnimation=F`) | – |
| **stoppie** | 44 / 52 | 0 / 0 / 0 | – / 1000 | none | on the ground, no clip | – |
| **threesixtyL** | 45 / 53 | 0.72 / 1.26 / 1.50 | 2000 / 15000 | R1 + stick (below) | **rotation**: the 360 | `MX_ThreeSixtyL.mot`, `Bike_ThreeSixtyL.mot` |
| **threesixtyR** | 46 / 54 | 0.72 / 1.26 / 1.50 | 2000 / 15000 | R1 + stick | **rotation**: the 360 | `MX_ThreeSixtyR.mot`, `Bike_ThreeSixtyR.mot` |

MXF and ATV play `MXF_…`/`ATV_…` rider clips and `MXF_Bike_…`/`ATV_Bike_…` bike clips; every rider set's
stunt `.mcf` names its clips `<P>_rider_<name>`, and the bike's are found by name, `<P>bike_<name>.mot`,
as on MX Unleashed. Wheelie and stoppie are trick ids 25 and 26 on MX Unleashed too ("Wheelie",
"Stoppie" in the HUD), but with no row and no score; the rows are what is new.

Changed rather than new: **backflip**. MXU row 57: H 1.00, S 1.33, D 1.67, `UsedForBike=T`. MX vs. ATV
row 42: H 0.60 (MXF 0.20), S 0.80, D 1.00, `UsedForBike=F`, 2000 / 10000 points, R1 to arm, a rider clip
(`MX_backflip.mot`) and a landing test (below). Three of the 24 default gestures name other stunts than
MXU's: slot 4 `nothing` (MXU `catwalk`), 16 `9oclockindianair` (`stalelin`), 23 `9oclocknac`
(`supercanindianair`). The 16 stunts MXU has and MX vs. ATV dropped are absent from its table:
catwalk, stalelin, supercanindianair, doublesaranwrap, throwaway, doublecancan, onehandedswitchblade,
onehandedstalefish, deadbodyindianair, scorpion, upcan, lazycan, starfish, squid, scorpionnac, bardova
(so the throw-away, `trick::Set::throwaway`, is gone).

## The gesture map

The pause screen's chart (`UI\MXStuntMapping.lyr`/`.lyt`, Untamed `.bpb`, and `_16x9.lyt`) is a
24-row `MXUIDynamicList` filled from the table; it holds no mapping itself. The mapping is each row's
key string, decoded by `0x267da0` into the same byte the rider holds its live buttons in (`+0x14ec`):

| letter | `C` | `T` | `1` | `2` | `L` | `R` | `U` | `D` |
|---|---|---|---|---|---|---|---|---|
| bit | 0x01 circle (trick 1) | 0x02 triangle (trick 2) | 0x04 R1 (trick 3) | 0x08 R2 (unused) | 0x10 | 0x20 | 0x40 | 0x80 |

(`3 4 S W B` are skipped.) The 24 defaults are MX Unleashed's grid, now data: C, T, C+T, R1+C, R1+T,
R1+C+T, each with up, down, left, right, in that row order:

| | up | down | left | right |
|---|---|---|---|---|
| Circle | Taunt | Lazyboy | CanCan | Nothing |
| Triangle | No Hander | Pendulum | NacNac | Flatliner |
| Circle+Triangle | Hart Attack | Indian Air | Barhop | Supercan |
| R1+Circle | KOD | Superman | Saranwrap | 9 O'Clock Indian Air |
| R1+Triangle | Tsunami | Seatgrab Indian Air | Shaolin | Cordova |
| R1+Circle+Triangle | Deadbody | Rigor Mortis | 9 O'Clock Nac | Stripper |

The trick buttons are commands 0x11–0x13 (openmx's `bindings::Command::TRICK_1..3`, circle, triangle,
R1 on the shipped preset). **R1 alone is in no gesture** — on MX Unleashed it makes group 0, nothing — and
on MX vs. ATV it is the button the backflip and the 360 arm with. Only the first 41 rows get a default
map entry (`0x2688e0`: `index < 0x29`); rows 25–41 have no key and are reached by remapping.

## The 360 (SLUS_211.04 `0x4e53d0`; Untamed `0x4fd630`)

Tuning, per bike, `.phy` `[Three Sixty]` (read by `0x4df740` into the bike object): `MaxThreeSixtyTorqueX`
+0x11f0, `MinThreeSixtyTorqueX` +0x11ec, `MaxThreeSixtyTorqueY` +0x11f8, `MinThreeSixtyTorqueY` +0x11f4,
`ThreeSixtyTorqueRateIncrease` +0x11fc (read, never used by the step), `ThreeSixtyTorqueRateDecrease`
+0x1200, `MinThreeSixtyTime` +0x11e0, `MaxThreeSixtyTime` +0x11e4. All 24 MX and ATV `.phy` on each disc:
X 600–1100, Y 200–1100, rates 480 / 1960, times 0–60. MX Unleashed's `.phy` has no such section.

One substep (the step clears "applied" +0x11d0 first):

1. **Idle** (+0x11d8 = 0). Off a lip it disarms. A lip is: a wheel on the ground (+0x324), the body
   rising faster than 5 (body +0x84), speed over 30 (+0xc38), no brake (+0x310 = 0), and no backflip
   armed or turning (+0x1194, +0x119c). On a lip:
   - armed (+0x11dc) and |stick x| > 0.7 with sign(x) = the armed direction (+0x1224): **fire**. With
     `t = (|x flick| − MinTime) / (MaxTime − MinTime)` (the flick speed at +0x12b4, as the backflip's),
     yaw torque (+0x1214) = dir × lerp(t, MinY, MaxY), pitch torque (+0x1210) = −lerp(t, MinX, MaxX),
     each clamped to its range; grace 0.9 s (+0x11e8); turning.
   - else with the buttons exactly R1 (`+0x14ec & 0xf == 4`): stick x < −0.7 arms direction +1,
     x > 0.7 direction −1. So **left then right** spins one way, right then left the other.
2. **Fired, still on the ground** (armed flag clear): the grace counts down and ends the move at 0. The
   frame the wheels leave the ground the step records the heading (+0x1220, from the node's forward
   row, acos table) and marks itself airborne.
3. **In the air**: the heading turned is `min(|h0 − h|, |h0 + 2π − h|)`. The move **ends** on landing
   (+0x324), on touching anything (+0xc8 → +0x34), or when |yaw torque| < MinY. Otherwise: with **no
   trick button held** and the stick pushed against the spin (sign(x) ≠ dir), yaw torque −= dt × RateDecrease
   × dir × |x| and pitch torque −= dt × x × RateDecrease — the only decay; held, or with the stick idle,
   the torque stays. The torque goes into the body (`vtable+0x78`, the same moment call as the
   backflip's) and "applied" and "ramp" (+0x11d4) are set.
4. **The rider's clip** (+0x1758): once the bike is more than **35°** (0.6108652) off its take-off
   heading the rider plays `threesixtyR`/`threesixtyL` (anim ids +0x13c0 / +0x13bc by direction),
   state 1; past it, state 2; back under 35° in state 2 the clip is stopped, state 0.
5. **The landing test**: when the move ends with the clip state not 0 — the bike still more than 35°
   off its heading, or the clip not stopped — the rider is stopped and the bike **wrecks** (`vtable+0x368`
   reason 0xb, or 0xf when `0x4e8370` finds the contact past the bike's wreck limits). A 360 counts when
   the bike lands within 35° of where it took off.
6. As the backflip: with "ramp" set and the wheels down, the control-authority ramp starts (+0x1188 =
   +0x118c), and the backflip's upright test (roll < π/2, pitch inside `AirbornePitch{Down,Up}WreckAngle`)
   gives control back.

The pitch torque is the same sign as the backflip's, so a 360 tips the nose up while it turns.

**Untamed** (`0x4fd630`, the same step at other offsets): the arm test reads the key set's "stunt
modifier" (`+0x14f4`, byte +0x18: `MAPKEY_STUNT_MOD`) instead of R1-alone, and a second path for a mapped
motion axis (`MAPAXIS_STUNT_360`, the Wii) fires straight away with fixed torques (`0x53d3e8`, `0x53d3f0`).
Which PS2 button is the modifier was not traced (Classic "mirrors the PS2", so R1 is expected).

## The backflip (SLUS_211.04 `0x4e4f20`; MX Unleashed boot `0x256b50`, openmx `air::BACKFLIP`)

The same lip, the same arm-on −0.8 / fire-on +0.8 stick y (+0x12ac), the same 1.25 floor on the minimum
torque, the same 0.6 s grace and the same upright ending as MX Unleashed. New:

- it arms only with **R1 alone** (`+0x14ec & 0xf == 4`); MX Unleashed's needs no button;
- decay: no trick button held and stick y ≤ 0 → torque −= dt × y × RateDecrease (+0x11bc);
- the rider's backflip clip (anim +0x13b8, state +0x1754): started when the bike's up row points down
  (node +0x84 ≤ 0) and its forward row up (+0x94 > 0), state 2 once forward points down, stopped when up
  is up again and forward > −0.79; ending the move with the state not 0 is the same wreck (0xb / 0xf).

Untamed (`0x4fd0d0`): the modifier instead of R1-alone, and a motion-axis path that fires with a fixed
hold of 0.8 (`MAPAXIS_STUNT_BACKFLIP`).

## The whip: the air control, not a trick

The discs' own word for it is a loading tip (`UI\LoadTips.txt`, both games): "You can whip the bike side
to side and front to back while in the air. You can even clear some jumps by laying the bike completely
flat in the air." A whip is the left stick in the air with no trick button: the airborne control, the
same step as MX Unleashed's (openmx `air::airborne_control`; MX vs. ATV's tail is at `0x4e6720`): the
stick against `AirborneControlMoment` (−400, 500, −150 on both games' Pro 250), then either plain damping
(`AirborneDamping` 450) when |+0x12e8| ≥ 0.1 (the stick, as far as traced), |+0xbe4| ≥ cos 20° (not
identified), a backflip or 360 is ramping (+0x1198, +0x11d4) or control authority is ramping back
(+0x1188 > 0), or else the auto-levelling `UnrollTorque × (UnrollGain × roll − ω_z)` and
`UnyawTorque × (UnyawGain × crossup − ω_y)`, then a gyro term on the pitch lane. What MX vs. ATV changes
is in the `.phy`, not the step:

| `[Airborne Control]` (Pro 250) | MX Unleashed | MX vs. ATV |
|---|---|---|
| `UnrollGain` | 3.5 | **1.9** |
| `UnyawGain` | 3.5 | **2.3** |
| `GasGyroTorque` | – | **175** (new: subtracted from the pitch lane under one flag, as `BrakeGyroTorque` 400 is added under another) |

Weaker auto-levelling holds a whip longer. openmx needs `GasGyroTorque` and the gyro's two conditions
(+0x1290 / +0x1294 and +0x318; which input each is was not traced) and nothing else for whips.

## Front flip: not in either game

- No string of SLUS_211.04, SLUS_217.01 or either `GAME_REL.BIN` says "whip", "frontflip" or "front
  flip", nor do the discs' UI and text members (MX vs. ATV's 121, Untamed's first 150, searched
  whole); no `.phy` section tunes either (`[Backflip]` and `[Three Sixty]` are the only rotation
  sections, and all 24 MX and ATV `.phy` of each disc hold the same `[Three Sixty]`); no stunt row
  names either.
- The backflip step fires one way only, stick y past +0.8 after −0.8, with a negative pitch torque.
  Captured: the gesture reversed on the same lip arms the backflip and never fires; the bike turns 11°
  in 1.57 s of air, as with no trick at all (`front.csv` against `drive1.csv`, 9°).
- Pushing the stick forward in the air (no trick button) tips the nose down through the air control
  only: 40° over a 1.6 s flight (captured), nowhere near a flip.

So a front flip in openmx would be an invented rotation (openmx's `air::INVENTED`, behind its flag), not
a port of either game. The same goes for "R1 + stick" whips: on both discs R1 alone only arms the backflip
and the 360 on a lip; in the air it does nothing (the whip capture held no button).

## Captured in PCSX2 (T7)

MX vs. ATV Unleashed, kit copy 9, save state 4 (a Free Ride in progress; the map was not identified),
`kit record` with `extracted/mxvatv/watch/air.toml` and one pad script each: gas from the state, R1 and the
stick on the first lip, about 2 s in. The player's bike object is at 0xe714e0 in that state (found by its
`[Three Sixty]` numbers); the spec samples the body's quaternion (+0xb8 → +0x170), its position, the
object's pitch/heading/roll (+0x360..+0x368), the 360's and the backflip's flags and torques, the
buttons and the stick, every other vsync. Rotations below are the frame-to-frame body-frame rotations
of the quaternion summed (x pitch, y yaw, z roll). CSVs in `extracted/mxvatv/capture/` (not in git).

| capture | pad | fired | torque | air | rotation in the air |
|---|---|---|---|---|---|
| `drive1` | gas only | – | – | 1.60 s | x 9°, y 1°, z 6° |
| `spin2` | R1, left, right (fast) | lip, row 63 | X −1100, Y +1100 | 0.87 s | x −108°, **y +120°**, z 8°; landed 190° off its heading on its side (a wreck) |
| `slowspin` | R1, left, right in 5 steps | lip, row 66 | X −600, Y +200 | 0.40 s clean, then hit something | first 0.4 s: x −26°, y +11° (fast: −44°, +48°) |
| `flip1` | R1, forward, back | lip, row 64 | X −1400 | 1.53 s | **x −259°**, y 0°, z 0°; landed nose-down mid-clip |
| `front` | R1, back, forward | armed, never fired | – | 1.57 s | x 11°, y 0°, z 6° |
| `whip` | stick left 0.6 s in the air, no button | – | – | 1.63 s | heading +38° and lean −13° at the end of the push, back to +15° by touchdown (auto-levelled) |
| `nosedown` | stick forward in the air, no button | – | – | 1.57 s | pitch +11° → −29°: 40° nose-down, landed |

What they prove: the arm needs R1 alone and the lip (the first try pushed the stick before the climb and
nothing armed; arming came only once the body rose); the fire takes the flick: the fast flick gave the
maxima, the stepped one the minima; the torque holds through the air with R1 held; the reversed backflip
does nothing; a whip is the stick alone and springs back. Not captured: a clean landing of either (the jump is too short for a full 360, and both
wrecks were not scored or traced), Untamed (its steps are read from the ELF only; M20).

## What openmx needs, per trick

In `notes/openmx-asks.md` (M19). In short:

1. **The tables**: `trick::Stunts::parse` for a count other than 57, `Stunt%02d` and BPB's `Stunt%d`
   titles, the new keys (PointValue, HoldPointValue, LinkModTime, HasAnimation) and a per-row default
   gesture instead of the identity map.
2. **The 360**: a real `air::Rotation` row replacing the invented `FLAT_SPIN` — two lanes (yaw and
   pitch), the stick-x gesture with R1 alone, the flick-scaled torque from `[Three Sixty]`, the 0.9 s
   grace, the decay rule, the heading tracking, the 35° clip rule and its landing wreck.
3. **The backflip, MX vs. ATV's way**: R1 alone to arm, its decay, its clip and landing wreck.
4. **The clips**: `skin::Clip::parse` for `.mot` versions 6 and 7 (every rider clip and the bike clips
   of the new stunts; MX Unleashed's are 3).
5. **Scores**: PointValue, HoldPointValue and `[Depreciation]` (how they are applied: not traced).
6. **Varial**: a pose stunt like the 24; it needs only the table, its clips and a remap to reach it.
7. **Wheelie and stoppie**: their hold scores.
8. **Whips**: `GasGyroTorque` in the air control (the `.phy` numbers do the rest).

## Both at once: no 360 backflip (M21)

Asked for openmx's MD20 (a 360 backflip and a 360 front flip, with R1 and the stick on a diagonal): do
MX vs. ATV's two moves run together? **No, in either game.** On a lip the two steps lock each other out:
whichever fires first wins the jump, and if both fire on the same frame the backflip does. Nothing in the
air or at the landing combines them. What the game has instead is the 360 itself, which is already a
corked spin: its pitch torque turns the bike about an axis halfway between its pitch and yaw axes (at
the full flick). There is no front flip to combine with anything (above).

**The two steps, side by side** (SLUS_211.04; Untamed's `0x4fd0d0` and `0x4fd630` are the same at
other offsets, with the stunt modifier in place of R1-alone):

| | backflip `0x4e4f20` | 360 `0x4e53d0` |
|---|---|---|
| order | first: both callers (`0x4b95f0`, `0x4d917c`; Untamed `0x4dbc20`, `0x4f115c`) call it before the 360 | second (`0x4b95fc`, `0x4d9188`; Untamed `0x4dbc2c`, `0x4f1168`) |
| clears on entry | applied +0x1194 | applied +0x11d0 |
| lip test refuses if the other has | **applied +0x11d0 or active +0x11d8** | **applied +0x1194 or active +0x119c** |
| on a refused lip | its arm +0x11a0 and active cleared | its arm +0x11dc and active cleared |
| arms on | R1 alone, stick y < −0.8 | R1 alone, stick \|x\| > 0.7 (direction by sign) |
| fires on | armed and y > +0.8 | armed and x past 0.7 the armed way |
| torque into the body (`vtable+0x78`) | its own vector +0x11c0 (pitch) | its own vector +0x1210 (pitch, yaw) |
| ends on | wheels down, a touch, \|torque\| < floor | wheels down, a touch, \|yaw torque\| < `MinThreeSixtyTorqueY` |
| landing test | its clip state +0x1754 ≠ 0 → wreck | its clip state +0x1758 ≠ 0 (still past 35°) → wreck |

(M19's line above, "no backflip armed or turning (+0x1194, +0x119c)", should read "applied or active":
the backflip's arm, +0x11a0, is not tested, so both can be armed together.)

So:

- **Arming does not exclude.** A diagonal (R1, stick up and to one side) arms both on the same lip frame.
  The game's stick reaches (−1, −1) at a corner: x and y are not normalised to a circle.
- **Firing does.** The first step to fire sets its "active" flag. From the next step call on, the other
  step's lip test fails, and that failure clears its arm, every frame, for as long as the first move
  lasts: the 0.6 / 0.9 s grace on the ground, then the whole flight. Off the lip (no wheel down) neither can
  arm anyway, so the second move cannot start in the air either.
- **A tie goes to the backflip.** With both armed and the stick flicked straight to the opposite
  diagonal, y and x cross their fire lines on the same frame. The backflip's step runs first and fires,
  and the 360's step, called next with the backflip active, clears its own arm. If x crosses first (flick
  sideways, then down), the 360 fires and the backflip never can.
- **One torque per frame.** Only the active move's step calls the body's moment function. The two
  "applied" flags are never set on the same frame.
- **The landing tests never combine.** Each move's ending checks only its own clip. The other move is not
  running, so its test does not apply. The 360's landing is judged by heading alone (within 35° of
  take-off, not upright); the backflip's by its clip (the bike over and back). The shared tail differs as
  well: after a backflip, control comes back once the bike is upright in the air (roll < π/2, pitch inside
  `AirbornePitch{Down,Up}WreckAngle`); after a 360, only at touchdown.
- **No combination row.** None of the 46 (MX vs. ATV) or 54 (Untamed) stunt rows combines a 360 and a
  flip. The ELFs and overlays (`SLUS_211.04`, `SLUS_217.01`, each `GAME_REL.BIN` and `UI_REL.BIN`) name
  the two only apart: `Left 360`/`Right 360`, `Backflip`, the stats `NumThreeSixty`, `NumBackFlips`,
  `STR_STAT_MOST_360_LANDED`, `STR_STAT_MOST_BACKFLIPS_LANDED`. Their "combo" strings (`Combo Score`,
  `NumStuntsInCombo`, `combo challenge`) belong to the HUD's score for a chain of stunts, not to a
  combined rotation; how that chain is scored was not traced.

**The 360 is already part backflip.** The 360 fires with pitch −lerp(flick, 600, 1100) as well as yaw
dir × lerp(flick, 200, 1100), both held through the air in the body frame. At the full flick (X −1100,
Y +1100) the bike turns about one body axis tilted roughly 45° between its pitch and yaw axes. The nose
comes up and over the shoulder, the bike lies near flat on its side halfway (up.y about 0), and a full
turn brings it back to its take-off attitude: a cork, not a flat spin. The slow flick (600 / 200) is
mostly pitch (M19 `slowspin`; M20 `step30`/`step60` went over on its back). Holding the stick back in
the air, with R1 still down (the 360 does not decay while a trick button is held), adds the air
control's pitch on top. That steepens the cork; it does not make a second, separate flip.

**Captured** (PCSX2 kit copy 9; MX vs. ATV state 4, its first lip, bike 0xe714e0; Untamed state 8,
Rocksbury's first big jump, bike 0x1be47f0; both with gas and R1 held from the state). The driver,
`extracted/mxvatv/watch/both.py`, is M20's closed-loop pattern for either game. It puts the stick on the
first lip frame, flicks once the step reports armed and centres once a move fires (so the input is
frame-exact on the game's own flags), and logs the bike every frame it sees (every other vsync).
`both_sum.py` gives the summaries below. CSVs `m21-*.csv` in `extracted/{mxvatv,untamed}/capture/` (not
in git). Rotations are the body-frame steps of the quaternion summed (x pitch, y yaw). "Path" is the
total turned; "net" is the angle between the take-off and touchdown attitudes. Wrecks are counted from
take-off to 1 s after touchdown (MX vs. ATV: +0x6e4, incremented by the wreck call `0x4e2260`; Untamed:
+0x708).

| run | stick, R1 held | MX vs. ATV (1.6 s of air) | Untamed (1.9 s of air) |
|---|---|---|---|
| `diag` | up-left (both arm, frame 120 / 280), then down-right | **backflip fires** (−1400), the 360's arm cleared the same frame; x −265°, y 0°; wreck (under-rotated, mid-clip) | **backflip fires**, 360 arm cleared; x −332°, y 0°, landed upright, clean |
| `diagx` | up-left, then right, then down-right a frame later | **360 fires** (−1100 / +1100); the backflip, armed, is cleared and never fires; path 306°, landed 37° off heading mid-clip: wreck | **360 fires**, backflip never; path 365°, 9° off, clean |
| `spinflip` | left, right (360 fires), then up, down on the ground | 360 only; up then down never arms the backflip; as `spin` | 360 only; as `spin` |
| `flipspin` | up, down (backflip fires), then left, right on the ground | backflip only; left then right never arms the 360; as `diag` | backflip only; as `diag` |
| `spin` | left, right | x −212°, y +217°, path 306°, up.y down to +0.07; landed 37° off heading with the clip running: wreck | x −258°, y +259°, **path 366°, net 11°**, up.y down to −0.04; landed 9° off: clean |
| `spinpull` | left, right, then the stick held back from take-off | x −289°, y +217°, **path 365°, net 10°**, up.y down to −0.22; landed 9° off: **clean** | x −350°, y +259°, path 436°: over-rotated, landed 101° off: wreck |
| `flip` | up, down | x −254°: under-rotated, wreck | x −321°, landed upright: clean |

In all 14 runs, no frame had both moves active or both applied. The `flipspin` and `spinflip` stick
moves came while the first move's grace was still running on the ground (frames 128–134 / 288–294, take-off at
156 / 302), the one window the arm could have opened, and nothing armed.

**Does the game do a 360 backflip?** Not as two moves. Its lip tests make the 360 and the backflip
exclusive per jump, and the diagonal gives a plain backflip. What it does have is the 360 as a cork
(about 45° of pitch share at the full flick), which the stick held back steepens. That is as close as
either disc comes. For openmx (MD20) this means: a "360 backflip" with both torques and both landing
tests at once is the port's own invention, like the front flip (`air::INVENTED`, named constants). The
accurate port keeps the two exclusive, with the backflip winning a tie and the 360's pitch share as the
`.phy` gives it.
