# OpenMX Mods

Tools that pull free-roam terrain out of other off-road games, and out of real-world elevation data, and turn
it into freeride track mods for the MX Unleashed (PS2) port, [OpenMX](https://github.com/FIZL0/openmx).

The port has no mod loader yet. `standard/` will define what one should load (see `PLAN.md`, T1).

| Source | Platform | What it gives | Status |
|---|---|---|---|
| Real elevation (USGS 3DEP, SRTM) | — | real OHV areas: Glamis, Ocotillo Wells, Little Sahara, Hatfield-McCoy | planned, first |
| MX vs ATV Unleashed | PS2 / Xbox | freeride areas, same studio (Rainbow) as MX Unleashed | planned |
| ATV Offroad Fury 1–4 | PS2 | open free-roam areas | planned |
| MX vs ATV Untamed / Reflex / Alive | PS2 / PS3 / 360 | freeride maps | planned |
| Motocross Madness 2 | PC | open Baja maps, heightmap terrain | planned |
| MX Bikes | PC | community freeride and compound maps (meshes) | planned |
| MX Simulator | PC | community freeride tracks (heightmap PNGs) | parked: the track site is down |

**No game data is in this repo.** You supply your own discs and downloads. Everything taken from them goes in
`iso/`, `extracted/` and `out/`, which git ignores.

## Layout

```
standard/   the freeride track mod standard (to write: PLAN.md T1)
notes/      format notes for each source game: where the terrain lives, file layouts, known gaps
tools/      importers, one per source, all ending in the shared heightfield-to-.trn writer (T2)
```

## Requirements

- A checkout of [OpenMX](https://github.com/FIZL0/openmx) next to this one as `../openmx`: its `game-sim`
  crate (`track.rs`) reads the `.trn`/`.scn` format the writer has to produce, and is how output is checked.
- Rust (cargo), and `7z` for unpacking discs.

## The target format in short

A freeride track is one `.res` member holding a `.scn` (CRLF INI: spawn, terrain parameters, placements), a
`.trn` (a quantised `u16` heightfield in a 16×16-way tree of 17×17-sample leaf blocks, 2129 bytes each, with
normals and per-sample surface), and one `.col` per placed shape. On `free06` the ground *is* the heightfield;
there is no separate terrain mesh. So every source here reduces to a height grid, a ground texture, and
optionally props. Detail: `../openmx/crates/game-sim/src/track.rs` and `../openmx/crates/game-render/src/terrain.rs`.
