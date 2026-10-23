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
The modifier is **R1** on the PS2 (M20): the ELF loads `UI\PSXController_S%i_P%i.ini` (format string
`%s%sController_S%i_P%i.ini`; the disc has style 1, presets 1–6), and the first preset binds
`MAPKEY_STUNT_MOD = BUTTONID_R1` (presets 2 R1, 3 L1, 4–6 Square; preset 1 moves `MAPKEY_STUNT3` to
Select). Live, R1 alone sets byte +0x18 of the key set and nothing else (Circle sets +0x12, Triangle
+0x14), and the motion-axis flag the step tests first is 0 on the PS2, so the stick path runs. Unlike
MX vs. ATV the arm reads that one byte, not "R1 alone": R1 + Circle held arms and fires the same 360.
See "Untamed in PCSX2" below.

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
does nothing; a whip is the stick alone and springs back. Not captured on MX vs. ATV: a clean landing of either (the jump is too short for a full 360, and both
wrecks were not scored or traced). Untamed: next section (M20).

## Untamed in PCSX2 (M20)

Kit copy 9, Untamed's state 5 (the Free Ride list), Rocksbury (the first row, a supercross stadium),
saved at the gate as scratch state 8. Its first big jump comes ~4.7 s from the gate at full gas
(~92–100 mph) and gives 1.8–2.3 s of air, enough for a full turn. The player's bike object is at
0x1be47f0 in that state (one of six objects holding `[Three Sixty]` 600, 1100, 200, 1100, 480, 1960; the
one whose speed moved). Its fields as the two steps use them: ground +0x354, speed +0xc28, body +0xe8
(the quaternion at body +0x170, rising speed body +0x84), node +0x114 (up row +0xb0, forward +0xc0),
key set +0x14f4, stick x +0x12bc and y +0x12c4, the flicks +0x12b8 (x) and +0x12c0 (y), the decay's stick
+0x1298/+0x129c; backflip applied/ramp/active/armed +0x1184..+0x1190, grace +0x119c, torque +0x11b0,
clip state +0x1794; 360 applied/ramp/active/armed +0x11c0..+0x11cc, grace +0x11d8, torque x/y
+0x1200/+0x1204, take-off heading +0x1210, direction +0x1214, clip state +0x1798; the wreck call
(`vtable+0x368` → `0x4fa390` → `0x4cbc60`) stores its reason at +0xc70 and counts wrecks at +0x708.
Spec `extracted/untamed/watch/air.toml` (`kit record`, every other vsync, the integer vsync counter is
0x410318) with pad scripts there; and `extracted/untamed/watch/air360.py`, a closed-loop driver (ps2kit's
PINE client and the pad FIFO) that pushes the stick on the frame the game's own fields show the lip,
flicks it once the step reports armed and centres it once it fires, because wall-clock pad scripts
missed the lip by a frame or two from one run to the next. CSVs in `extracted/untamed/capture/`
(not in git). Rotations as above (body-frame, summed over the air).

| capture | input | fired | torque | air | what happened |
|---|---|---|---|---|---|
| `drive1` | gas only | – | – | 2.6 s | the first jump, heading steady |
| `clean360` (driver) | R1, left, right on the lip | frame 284 | X −1100, Y +1100 | 1.87 s | **x −249°, y +253°, z 1° (355° about a diagonal)**; clip state 2 at frame 322, stopped at 404 (back within 35°), landed at 416 **1.6° off the take-off heading**, upright (up.y 0.99), **no wreck**, rode on |
| `backflip` (driver) | R1, stick up, stick down on the lip | frame 284 | X −1400 | 1.83 s | **x −360°** at touchdown, y 0°, z 0°; clip 1 at 334 (bike over), 2 at 364, stopped at 408, landed at 412 upright, **no wreck** |
| `spin2` | R1, left, right (fast) | lip, 4.70 s | X −1100, Y +1100 | 2.27 s | heading back within 35° (clip stopped) at 6.63 s, past 35° again at 7.17 s; landed at 7.30 s with the clip in state 2: **wreck, reason 0xb** |
| `mid120` | the same with the stick 120 ms at centre | lip, 4.80 s | X −725, Y +426 | 2.63 s | landed mid-clip (state 2): **wreck 0xb** |
| `step30`, `step60` | the stick left to right in 5 steps of 30/60 ms | lip | X −600, Y +200 | ~2.5 s | mostly pitch: over on its back, the move ended on touching (no wheel down, up.y −0.85) with the clip in state 2: **wreck 0xb** |
| `spin1` | stick left 0.6 s before the lip | lip, 4.77 s | X −1100, Y +1100 | – | the stick steered the bike ~50° on the ground; a wheel touched 0.03 s after take-off and ended the move (clip still 0, no wreck); later hit something (reason 1) |
| `r1circle` (driver) | R1 **and** Circle held, left, right | frame 284 | X −1100, Y +1100 | – | armed and fired as `clean360`: the arm reads only the STUNT_MOD byte |
| `nomod` (driver) | no R1, left, right | never armed | – | – | the stick alone steers on the lip; nothing arms |

What they prove:

- **MAPKEY_STUNT_MOD is R1** (the disc's first PS2 preset, and live: R1 sets key-set byte +0x18 and
  nothing else); with it held, other buttons do not stop the arm (MX vs. ATV wants R1 alone).
- The arm and fire are MX vs. ATV's: the stick's sign on the first lip frame picks the direction (one run
  with the stick already right on that frame armed direction −1 and then never fired); the flick on the
  fire frame scales the torque between the `[Three Sixty]` minima and maxima (a jump from −1 to +1 gave
  the maxima, a stepped crossing the minima, a pause at centre between).
- The torque is constant while R1 is held, and both lanes are always on: at the full flick a 360 is a
  turn about an axis ~45° back from vertical (pitch and yaw about equal), not a flat spin; at the minima
  it is mostly a backflip.
- **A 360 lands when the bike comes back within 35° of its take-off heading before it touches**: the
  clip then stops (state 0) and the landing is clean. Untamed's Rocksbury first jump at full gas gives
  one full turn at the full flick with ~0.2 s to spare.
- **Landing mid-clip wrecks with reason 0xb** (four captures: `spin2`, `mid120`, `step30`, `step60`),
  the moment the move ends (a wheel down, or anything touched). Reason 0xf (`0x500880`: the contact's
  surface carries a flag and the bike's contact angles +0x1240/+0x1248 are past its limits +0x13f8..+0x1400,
  or +0xdd0 is set) never came up here.
- The backflip lands clean after a full −360° and the same clip rules; its −1400 came with a y flick
  of 662 on the fire frame (the fast end, as MX vs. ATV's `flip1`).

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
