# OpenMX Mods

Tools that pull free-roam terrain out of other off-road games, and out of real-world elevation data, and turn
it into freeride track mods for the MX Unleashed (PS2) port, [OpenMX](https://github.com/FIZL0/openmx).

The port has no mod loader yet: that is openmx's Phase 14 (MD1–MD6). `standard/` will define what it loads
(`plan/TODO.md`, T1).

| Source | Platform | What it gives | Status |
|---|---|---|---|
| Real elevation (USGS 3DEP, SRTM) | — | real OHV areas: Glamis, Ocotillo Wells, Little Sahara, Hatfield-McCoy | planned, first |
| MX vs ATV Unleashed | PS2 / Xbox | 56 maps in MX Unleashed's own formats (`notes/mxvatv.md`) | surveyed: a repack |
| ATV Offroad Fury 1–4 | PS2 | open free-roam areas | planned |
| MX vs ATV Untamed | PS2 | 49 maps, same terrain format; scenes in binary BPB (`notes/mxvatv-later.md`) | surveyed: BPB to decode |
| MX vs ATV Reflex / Alive | PS3 / 360 | freeride maps | planned, no disc |
| Motocross Madness 2 | PC | open Baja maps, heightmap terrain | planned |
| MX Bikes | PC | community freeride and compound maps (meshes) | planned |
| MX Simulator | PC | community freeride tracks (heightmap PNGs) | parked: the track site is down |

**No game data is in this repo.** You supply your own discs and downloads. Everything taken from them goes in
`iso/`, `extracted/` and `out/`, which git ignores.

## Building and installing

`./build.sh --check` builds every mod your discs give into `out/mods/` (`mods` is a link to it), checks
them, and copies them into the game's own `mods/` folder: openmx reads `mods/tracks/<id>/` beside its disc
and `settings.txt`. By default that is `openmx/`; `--install DIR` names another copy of the game (another
machine: copy `out/mods/` into the folder holding its disc), `--no-install` skips it. Ride one with
`openmx --track <id>`, or pick it in the menu (Settings: Mods on).

## Layout

```
plan/       the task list (agent-runner format): TODO.md, DONE.md
journal/    one folder per finished task
standard/   the freeride track mod standard (to write: T1)
notes/      format notes for each source game: where the terrain lives, file layouts, known gaps
tools/      importers, one per source, all ending in the shared heightfield-to-.trn writer (T2)
```

## Requirements

- A checkout of [OpenMX](https://github.com/FIZL0/openmx) next to this one, linked in as `openmx`
  (`ln -s ../openmx openmx`, git-ignored): its `game-sim` crate (`archive.rs`, `track.rs`) reads every format
  here, and is how output is checked.
- Rust (cargo). The tools read the disc images directly; nothing needs `7z`.

## Working the plan

The tasks in `plan/TODO.md` are worked by [agent-runner](https://github.com/FIZL0/agent-runner) (`AGENTS.md`
has the rules, `runner.toml` the settings, `./check.sh` is the gate):

```sh
agent-runner plan                # what is open
agent-runner run --dry-run 3     # what three slots would start
tmux new -s mods agent-runner run 3
```

## The target format in short

A freeride track is one `.res` member holding a `.scn` (CRLF INI: spawn, terrain parameters, placements), a
`.trn` (a quantised `u16` heightfield in a 16×16-way tree of 17×17-sample leaf blocks, 2129 bytes each, with
normals and per-sample surface), and one `.col` per placed shape. On `free06` the ground *is* the heightfield;
there is no separate terrain mesh. So every source here reduces to a height grid, a ground texture, and
optionally props. Detail: `../openmx/crates/game-sim/src/track.rs` and `../openmx/crates/game-render/src/terrain.rs`.
