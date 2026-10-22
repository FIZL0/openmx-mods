# T9: the accuracy pairs' openmx half (blocked on openmx MD2, MD10)

## What was found

The task starts "once openmx has a track from a mod folder (MD2) and --shot-pose (MD10)". Neither is
there. openmx's main is at `b63486a`, and MD2 and MD10 are both `[ ]` in its `plan/TODO.md` (lines
175, 183). No branch and none of its worktrees (`openmx-slots/s1..s5`: P100, P102, S99, P101, P94)
works on them. `openmx/target/release/openmx --help` lists `--shot PATH` and `--shot-frame N`, with no
`--shot-pose` and no mod flag. So the default flags can't be set to "their real flags", the fov's axis
can't be settled against MD10, and Lago's eight pairs can't be taken.

## What was done

- `tools/accuracy/src/look.rs`: `OPENMX_ARGS` (the default template), `openmx_args` (the command line
  for one pose, which was inline in `main.rs`'s `cmd_pair`), `help_has_flag` and `openmx_missing`.
  `openmx_ready` now takes the template and checks each of its flags as a whole word in `--help`. Before,
  it looked for the substring `--mod`, which openmx's `--modifiers` contains. openmx's current `--help`
  usage block does not print `--modifiers`, so the old check said "no --mod" correctly today, by
  luck.
- `main.rs`'s `cmd_pair` uses them. Its behaviour is unchanged apart from the ready check.
- Tests (`tools/accuracy/tests/generated.rs`): `the_openmx_command_line_built_from_a_pose` (the brief's
  test: six words, a folder with a space stays one argument, the `--shot-pose` word parses back to the
  pose within 1e-3, a template of the caller's own with `--x={mod}` forms) and
  `openmx_is_ready_only_with_every_flag_as_a_word`.
- Ran `accuracy pair --mod out/mods/tracks/mxvatv_nat05 --openmx openmx/target/release/openmx`. It
  indexed the 8 original shots plus `spawn`, and `index.md` says "has no --mod … (openmx MD2) and no
  --shot-pose (openmx MD10)". No openmx shot was attempted.
- `notes/accuracy.md`: a new section, *openmx's half (T9, waiting)*, on what is ready and what is left.

## Also

- The gate (`./check.sh`) failed before any of this, in `tools/renderprobe` (M9): openmx's V49
  (`5127066`) gave `game_render::eco::Grove::load` two new arguments, the scene's `TerrainLayout` and
  the `Terrain`, to stand plants on the ground. `main` here has no fix. The smallest change that
  compiles: keep the terrain's bytes (the padding-free view where survey makes one), and at the grove,
  parse `game_sim::track::Scene` for its layout and pass both. If either is missing, the grove line
  reports that instead. renderprobe's output on a real map after this change was not re-run (see
  below).
- `notes/openmx-asks.md`: MD2 and MD10 need a command line for the pairs (a mod track by flag, both
  flags in `--help`).

## Not verified

- The openmx command line against a real MD2/MD10 build: there is none. MD2's flag name is a guess
  (`--mod`). The plan names MD10's (`--shot-pose x,y,z,yaw,pitch,fov`).
- The fov's axis (width or height): it needs MD10.
- Lago's eight pairs and what differs in them: none taken, because openmx can't load the mod or take a
  posed shot.
- `renderprobe` on a disc map after the Grove::load fix: it compiles and its tests pass, but no
  report was regenerated or compared.
