# C11: the combined and ultra maps in openmx

openmx landed MD8 (worlds), MD9 (the look, World sky) and MD17 (streaming). This task loads
world_mxu, world_mxvatv, world_untamed and world_ultra as openmx does and checks the ground, the rides
across joins, the far corners and the look at every C9 gate.

## Log

- Re-imported the three discs into `target/c11/mods/tracks` (96 maps) and `target/c11/mxu` (27),
  built every spec with `stitch --mods target/c11/mods --out target/c11/stage` (1 min 37 s; the ultra
  map widened to 4.768, 116,073 × 33,849 ft, 281 new pieces). openmx daf98721 built into
  `extracted/openmx-target` (release).
- `tools/stitch/src/openmx.rs` (new): a world read as openmx's `worlds::find` reads it (its steps
  repeated on the same readers, since `crates/openmx` is a binary), into game-sim's own `Tiled`; then
  every tile against its imported map and every join against its file and stitch's field, through
  `Tiled::at` and `contact_search::ground`, bit for bit.
- **Found: openmx leaves a 4.5 ft strip with no ground at every join boundary.** A terrain leaf is
  17×17 samples, so a 16-leaf join (257 samples) has 256 cells; game-sim's `Tile::samples()` gives
  256 and `Tile::local` drops the last cell (`i + 1 < n`), the same off-by-one as T10's `Ground::at`
  ask. The next join starts at sample 256, so x 1147.5–1152 (and every 1152 ft, both axes) belongs to
  no one: 3.6 M of 1.3 G one-foot points on world_mxu, 3.4 M on world_mxvatv, 4.2 M on world_untamed.
  `Tiled::off_world` probes ±150 ft for holes, so a rider is shot back 150 ft before each strip:
  ridden on world_untamed from (5600, 3000) at 90°, thrown at x 6760.7, the strip at 6907.5. This is
  the user's "still shoot the rider off inside them" (openmx MD9 note).
- **Found: stitch's partial pieces were clamped padding to openmx.** A piece of 213 samples is 14
  leaves to the reader (224 cells); trnwrite fills the last leaf by repeating the edge sample, and
  openmx owns all of it ahead of the next join (precedence). A collar's padding stood over lattice
  ground 0.07–0.13 ft off stitch's field on the generated world; on the ultra map edge pieces of 17,
  83, 99 samples did the same.
- Fix in stitch (`build::cuts`): every piece is whole leaves of real ground, pieces start a leaf
  before the previous one ends (chunk 256 → stride 240), the last on an axis ends on the world's or
  collar's edge (moved back over the one before); the ultra map's new pieces start two cells inside
  the rectangle before them and run past its end (onto the next piece's ground, or a world, which
  comes first), with a point deep in a world taking the rectangle's nearest.
- Generated tests (`tests/openmx_worlds.rs`): a 3-tile world with a collar and a 2-world ultra map,
  checked as above plus every foot of the world owned. On the old cutter the 3-tile world had 104,545
  unowned points a foot apart and 1,987 join samples off the field; now none.
- The ultra map's new pieces now start a cell inside the world before them. The first attempt gave
  that column the wrong look: `UltraPlan::at`'s foot assumes a point outside or on a world's
  rectangle, so one step in took the z1 edge's look (92,894 `.blend steps by 255` in `stitch check`).
  Now such a sample takes the look and surface of its rectangle's nearest point, and the world's own
  height (the band, or `ultra::World::ground` off its joins deeper in; clamped stand-ins first failed
  the check's cross-join comparison by 0.4 ft on the generated ultra map).
- `stitch check` then failed the real ultra map on 19 border samples, ≤ 0.0083 ft: a new piece holds
  a world's edge as its first join has it, rounded again, and a second, overlapping world join differs
  from the first by up to its quantum. The comparison is now to the sum of the two quanta, not the
  larger. `a_tampered_world_fails_the_check` and `a_step_at_a_worlds_edge_fails_the_check` still fail
  what they should (the second now finds the piece a whole foot inside world_one's edge).
- `tests/openmx_worlds.rs`, ignored: `every_world_of_the_three_discs_in_openmx` (imports the three
  discs, builds the four worlds from their specs with stitch's `build`/`ultra::build`, checks the ground,
  then `rides`), `rides_in_openmx`, `shots_at_every_gate`. Run with `C11_ROOT=target/c11/real`
  (`tracks/` linked to this session's imports), DISCX_ISO at the main checkout's iso/, openmx from
  `extracted/openmx-target`. Results in notes/stitch.md *In openmx*. The build, checks and rides take
  2.5 min; the shots 8 min.
- Rides: 135 across joins, 0 thrown (before the fix, the one test ride on join ground was thrown).
  world_untamed rides tick for tick the same in the ultra map; world_mxu 76,032 ft out parts from
  itself alone on tick 0 (11.4 ft by tick 400). A stopped, braked bike creeps 2 ulps a tick far out
  (0.47 ft/s at x 113,040). Asked of openmx.
- Shots: 206 pairs at every C9 gate. The first run drew no near ground at all: openmx streams ground round
  the rider, who was at the spawn; `--trace-at` under the pose fixes it (`--world-ground all` instead
  left the near tile blank; asked). By eye, 10 of 27 sheets: no seam at the joins. world_untamed's
  tiles are dirt-coloured in the ultra map (`WorldPaint::pack` past the GPU's limits; asked).
- Asks filed in notes/openmx-asks.md: the last-cell off-by-one, one-level tiles refused by `Tiled`,
  the far creep, the paint pack, `--world-ground all`.

## Not verified

- The ultra map's u-pieces against stitch's field: `UltraPlan::at` is private, so only their files
  (bit-exact), the strips and `stitch check`'s world seams were checked, not their heights against the
  blend.
- The shots were read by eye on 10 of the 27 sheets (160 of 412 pictures), at 320 × 240 in the sheet;
  the rest were written and not looked at. Nothing reads a seam in look by machine.
- Which ultra-map tiles exactly are dirt: seen on untamed_sm_hs (and nat_md looks as sandy in both);
  the device's `max_texture_array_layers` here and each tile's page count were not read.
- Rides that did not reach the far tile (24 of 135) were not looked into one by one; none was thrown.
  Their wrecks could hide a join problem a straight gas ride cannot show (steep join ground is
  `stitch check`'s).
- The camera far from the origin: `--trace-camera` printed nothing with `--trace-feel`, so camera
  jitter was not measured; only the bike's trace was.
- Per map sky (MD9's other setting) was not shot; only Global.
- The main checkout's out/ and openmx's installed mods still hold the old worlds (with the strips)
  until `./build.sh` runs there; not touched.
