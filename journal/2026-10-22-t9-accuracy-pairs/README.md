# T9: the accuracy pairs' openmx half

## First session: blocked

### What was found

The task starts "once openmx has a track from a mod folder (MD2) and --shot-pose (MD10)". Neither is
there. openmx's main is at `b63486a`, and MD2 and MD10 are both `[ ]` in its `plan/TODO.md` (lines
175, 183). No branch and none of its worktrees (`openmx-slots/s1..s5`: P100, P102, S99, P101, P94)
works on them. `openmx/target/release/openmx --help` lists `--shot PATH` and `--shot-frame N`, with no
`--shot-pose` and no mod flag. So the default flags can't be set to "their real flags", the fov's axis
can't be settled against MD10, and Lago's eight pairs can't be taken.

### What was done

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

### Also

- The gate (`./check.sh`) failed before any of this, in `tools/renderprobe` (M9): openmx's V49
  (`5127066`) gave `game_render::eco::Grove::load` two new arguments, the scene's `TerrainLayout` and
  the `Terrain`, to stand plants on the ground. `main` here has no fix. The smallest change that
  compiles: keep the terrain's bytes (the padding-free view where survey makes one), and at the grove,
  parse `game_sim::track::Scene` for its layout and pass both. If either is missing, the grove line
  reports that instead. renderprobe's output on a real map after this change was not re-run (see
  below).
- `notes/openmx-asks.md`: MD2 and MD10 need a command line for the pairs (a mod track by flag, both
  flags in `--help`).

## Second session: openmx landed MD2 and MD10

openmx's main is at `3b11fd6` (MD2 `a811ee8`, MD10 merged at `94c62d0`). Its `target/release/openmx`
was older than both (08:54, no `--shot-pose`), so it was built into this slot's
`extracted/openmx-target/` (`CARGO_TARGET_DIR`, nothing written in openmx's checkout).

- **The flags.** MD2 is `--mods DIR --track <id>`, DIR holding `tracks/<id>/`. MD10 is `--shot-pose
  x,y,z,yaw,pitch,fov`, with yaw, pitch and the fov as the harness has them (its `shot_pose.rs` quotes
  `tools/accuracy/src/pose.rs`), the fov the horizontal one.
- **First try, failed:** a headless `--shot` run with no `--frames` writes nothing to stdout and does not
  end. `timeout 120` killed it. With `--frames 2` it writes the shot and exits in 1.5 s.
- **`--graphics original`:** openmx's `full` puts the fog at 768..8908 ft. `original` uses the disc's
  141..768, which is what the original shows, so the default uses it.
- `look::OPENMX_ARGS` is now `--mods {mods} --track {id} --headless --no-sound --frames 2 --size 640x480
  --overlay none --graphics original --shot {shot} --shot-pose {pose}`. `openmx_args` fills `{mods}`
  (the folder over `tracks/`) and `{id}` (the folder's name, which MD2 requires to be mod.json's id),
  and `{mod}` still fills for a template of the caller's own. `in_a_mods_folder` refuses a mod outside
  a `tracks/` folder (openmx could not find it by `--mods`). `openmx_missing` names `--mods`.
- `cmd_pair` runs openmx in `--openmx-dir` (default `openmx`), since openmx finds the disc through the
  nearest `kit.toml` above its working directory. The mod folder, the out folder and the binary are
  made absolute first. openmx's stdout goes to /dev/null, and its stderr still shows.
- Tests: `the_openmx_command_line_built_from_a_pose` checks every flag's word, a space in the mods
  folder, the pose read back to 1e-3, no placeholder left, and the tracks-folder check.
  `openmx_is_ready_only_with_every_flag_as_a_word` checks that `--mod` is neither in `--mods` nor in
  `--modifiers`.
- **Lago.** This slot had neither the mod nor the original's shots. `import-mxvatv --map nat05` from
  the main checkout's `iso/` (read-only) gave `out/mods/tracks/mxvatv_nat05`: 447 files, 31 todo, read
  back. The original's eight shots, `poses.txt` and `heights.txt` were copied from slot s3's
  `out/accuracy/nat05` (T7's eight). Slot s2 has a later set with M12's centre (2880 rather than
  2877.75) and landmark poses, which were not taken here. Then `accuracy pair --mod
  out/mods/tracks/mxvatv_nat05 --out out/accuracy/nat05 --openmx extracted/openmx-target/release/openmx`
  took all eight openmx shots and pairs in 12 s, and `index.md` lists them.
- **Reading them** (`notes/accuracy.md`, *Lago, the eight pairs*): the framing is the same (which
  settles the fov as the width, and shows the props and the terrain's shape agree). The ground is flat
  tan in openmx and textured dirt in the original (openmx MD4, open). Grey slabs come from the waypoint
  race's props, which the port draws though the scene hides them (already asked). Plants, sky and fog
  are close.
- `notes/openmx-asks.md`: the two asks about the harness's command line are marked answered.

### The gate, broken by openmx's MD5

`./check.sh` failed before the commit, outside T9's tools. openmx's MD5 (`c3f6942`) gave
`game_sim::track::Collision` a third arm, `Hierarchy`, and made `Collision::parse` read class 1.
origin/main had no fix, and no task in the plan names it. The smallest changes, made here:

- `tools/survey/src/lib.rs` `col`: a `Hierarchy` arm (`hierarchy v<n> <parts> parts`). Before it,
  survey did not compile.
- Three tests expected openmx's old words, "collision class 1 is not understood". Their fixtures are
  still refused under MD5 (survey's `Swing.col` is cut short, and modcheck's and import-mxvatv's
  `Blade.col` have one word left over), so each test now takes the expected refusal from
  `Collision::parse` of the same bytes: `survey/tests/row.rs`, `modcheck/tests/disc.rs`
  (`a_refused_or_missing_shape_must_be_in_todo`), and `import-mxvatv/tests/disc.rs`. The last keeps
  its "openmx MD5" check while the reader says "class 1", as the MD1 case beside it does.

The gate then passed (`check: ok, 117 passed, 0 failed`, with `DISCX_ISO` at the main checkout's `iso/`).

## Not verified

- The pairs were read by eye from half-size contact sheets. No pixel metric was computed. The fov's
  axis rests on the props landing at the same pixels, to about 1 % of the width, against the 33 % a
  vertical fov would make.
- The original's shots are T7's from slot s3 (copied, not retaken). The mod they are compared with was
  imported now, with today's importer. A change between the two in the scene or the props would show in
  the pairs as a difference that is not openmx's.
- Which of the scene keys (`Hide=`, `Invisible=`, `IsTrigger=`, `Activity=`) hides the waypoint slabs
  in the original was not looked up in SLUS_211.04.
- The plants' count, density and far fade were not compared. Only where a plant stands was compared.
- The openmx binary is one built from openmx `3b11fd6` into this slot, not openmx's own
  `target/release` (which is older than MD10). A later openmx may draw differently.
- `renderprobe` on a disc map after the first session's `Grove::load` fix: still not re-run.
- The MD5 fixes: only the generated tests were run. The importer's and modcheck's output on real
  MX vs. ATV maps that hold a class-1 `.col` (the windmill blade) was not re-run against MD5's reader.
  Lago imported and read back clean.
