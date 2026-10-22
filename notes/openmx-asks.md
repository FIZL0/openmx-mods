# What the port needs from openmx

Changes openmx has to make before a mod works there. Agents here write them; a human files them in openmx's
`plan/TODO.md` (Phase 14). Each entry: the file that fails, the reader and its error, and what would fix it.

## Filed

- **MD1** `Terrain::parse` rejects every MX vs. ATV terrain: `257 blocks of 2129 from 2189 end at 549342,
  but the tree starts at 1598302` (Untamed FS_A). The texture table is non-empty and the pages sit
  between the blocks and the tree.
- **MD2** a mods folder the shell loads tracks from (the standard, T1).
- **MD3** 80×80-leaf terrains at 4.5 ft (TerrainWidth 5): edge, radar, draw distances, height band.
- **MD4** the ground drawn from the terrain's own texture pages.
- **MD5** `Collision::parse`: `collision class 1 is not understood` (MX vs. ATV Unleashed
  `Col\Nat07_WM_Blade.col`).
- **MD6** online rooms on a mod track: same mod id and hash, or not seated.
- **MD7** music packs from mods/music/, with a setting for which games' music plays.
- **MD8** worlds of several terrains (the combined and ultra maps), each tile bit-exact to its source.
- **MD9** the join ground blending smoothly in look (textures, detail) as well as shape; World sky: Global (default, one skybox and lighting) or Per map.
- **MD10** `--shot-pose`, for the accuracy harness (T7).
- **MD11** the track pick filtered by game: All games, MX Unleashed, MX vs. ATV Unleashed, Untamed, …

## Open

- **MD1, more evidence** (T3, `notes/mxu-terrain-limits.md`): `Terrain::parse` refuses MX Unleashed's
  own terrains too, not only the other games'. `tracks\nat01\nat01.ps2.trn`: `538 blocks of 2129 from
  3377 end at 1148779, but the tree starts at 3246699`. `tracks\sx01\sx01.ps2.trn`: `257 blocks of 2129
  from 2189 end at 549342, but the tree starts at 1598302`. That is all 22 nationals and all 24
  supercross tracks on MXU's disc. The gap is 16/32/48 texture pages of 65,560 bytes. The game
  (`0x001d9930`) reads them in sequence: a format word, a mip count, then per mip a 16-byte header and
  the pixels. The root node follows the last page, and the header's tree offset is never read. Fix:
  accept `blocks_end <= tree_at`, or better, walk the page records and require them to end at
  `tree_at`.
- **MD3, the radar's span** (`crates/openmx/src/main.rs:2768`): `overlay::radar_span(ground.spacing)`
  uses the `.trn`'s scale. The game uses max(192 × scene TerrainWidthScale, 192 × TerrainBreadthScale)
  (`radar_init`, `0x0035d000`..`0x0035d03c`). It is the same number on every shipped track. It differs
  only for a mod whose scene and file disagree. Fix: take it from `TerrainLayout`'s two scales.
- **MD3, the ground's draw radius** (`crates/openmx/src/main.rs:250`, `crates/openmx/src/web.rs:71`):
  `GROUND_RADIUS = 44` samples is "about a hundred and twenty yards" at 9 ft. At 4.5 ft (MXU's own
  nationals, and every big MX vs. ATV map) it is half that, 198 ft. Fix: a radius in feet, divided by
  `ground.spacing`.
- **MD3, the height band** (`crates/game-render/src/terrain.rs`): the band it fits from props is in the
  file. It is the leaf's `height_scale` (node +0x10) × the file's scale (+0x80), which is what
  `terrain_cell_read` `0x001ddaf0` and `terrain_point_query` `0x001dce80` apply, and what game-sim's
  `TerrainNode::height_scale` already reads. The module's header map also names the levels/tree offset
  "1, size" and the two block counts "columns, rows". On a national those counts are 538, not samples.
  Only `first-draw` uses this reader. Fix: build its field from game-sim's `Terrain`, or retire it.
- **MD3, nothing to change**: the edge (`0x001a6e50`, game-sim `world::Ground::area`) and the query
  have no grid-size limit below 16 root cells a side. The original's one cap is a 4000-leaf draw list
  (`0x001d5274`), which the port does not copy and needs no stand-in for.
- **MD2, what the standard asks of the loader** (T1, `standard/OPENMX-TRACK-STANDARD.md`): (1) track
  sounds: `crates/openmx/src/audio.rs` reads `tracks\{track}\{track}.aud` by exact name from the track
  `.res`. MX vs. ATV keeps its maps under `Env\<MAP>\`, so a mod's `.aud` needs a suffix lookup
  (`<stem>.aud`), as `toy.rs`, `splat.rs` and `particles.rs` already do for their files. (2) The track
  stem: `<stem>.toy`, `<stem>particles.prt`, `<stem>_shadow_NN.pst` are built from `args.track`. For a mod
  the stem is its one `.scn`'s file stem, not its id. (3) `<track>_2.res`: a mod has no second archive
  (bike, physics, `biplanecpresets.ini`), so the ride takes them from a disc track's.
- **MD2, lookups on a segment boundary** (T4, `tools/modcheck`): `game-render::disc::Res::find_suffix`
  matches a plain string suffix, so `RockA.col` would also find `Col/xRockA.col` if that came first. On
  the disc's own `.res` the order hides it; a mod folder has no member order. `modcheck` matches on a
  `/` boundary and fails a name that finds two files. Fix: the same in the mod loader's lookup.
- **MD7, the `.vig` reader is in the shell crate** (T4): `Clip::vig` / `Adpcm::vig` live in
  `crates/openmx/src/audio.rs`, which brings wgpu, ggrs and iroh with it, so this repo cannot call
  them to prove a music pack (§5 step 4). `modcheck` checks only that each song's file is there.
  Fix: the `.vig` header and ADPCM decode in a crate with no shell dependencies (game-render or its own).
