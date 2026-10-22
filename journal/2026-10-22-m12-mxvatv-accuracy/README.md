# M12: MX vs. ATV Unleashed accuracy pass (every map through T7)

All 56 MX vs. ATV Unleashed maps ridden in PCSX2 (kit copy 9, US disc SLUS_211.04) and held against
their mods: the ground point by point, the scene's numbers as the game holds them, every file the game
opened for the map, and the original's half of the screenshot pairs from 11 poses a map. The per-map
table is `notes/mxvatv.md`, *Against the game (M12)*; how it was run is `notes/accuracy.md`, *Every
map*; what only openmx can fix is at the end of `notes/openmx-asks.md`.

## What was done

- **Into every map.** A fresh profile has six maps open; the game's own cheat `PITPASS`
  (`UI\CheatCodes.txt`) opened the rest, and state 5 of copy 9 is the Free Ride list after it. Presses
  drop while a track preview loads (T7 said so), so counting them failed (18 Downs landed on the 10th
  track). The driver reads the list's place from RAM instead (the window's top at `0x124cee4`, the
  cursor's row at `0x124cea4`, found by diffing three captures one press apart) and presses until it is
  the map's. Then Start Event, load, `accuracy capture` until the ride camera is found.
- **Per map**: `capture`, `heights --points 2000` (8,000 points in the four modes), `poses` with three
  landmarks, `shoot --live live` (the game's own camera) and `shoot` (11 poses: start, centre ×4,
  overview, ridge, 3 landmarks, live), `pair` (index only), then `scene` and `files` on the image. Three
  passes: 55 maps; 24 again (FS04, whose arena camera the viewport pick refused; FS01–08 and Lago, shot
  before the poses' box was fixed; 14 maps with a cut-short PNG); then FS03, MC06, SX03 and SX09, whose
  PNGs were still cut short. 611 shots, 9–11 a map, every one a whole PNG.
- **tools/accuracy**, added: `scene` (`sceneram.rs`), `files` (`loaded.rs`), landmark poses
  (`landmarks.rs`), the poses' box from the file's leaves (`look::leaf_extent`), PNG check in `shoot`
  (a cut-short shot is taken whole from the copy's `snaps` folder), the viewport pick strictly inside the box (the arenas' cube-map cameras sit at x = z = 0, on
  its edge), `tests/every_map.rs`.
- **tools/import-mxvatv**, fixed from what the game opened (`pull_loose`, three new requests):
  - the toys' physics collections: `<map>\<Name>.pco` for the first `NumberOfPhysicsCollections` of the
    `.toy` (`0x002171b0`), and each part's `Col\<Name>m.col`/`f.col` (`0x004922a0`). Lago's saloon doors
    were the first seen; Nat01, Nat02, Nat05, Nat06 and Nat07 have them.
  - the shadow materials, `Tex\shadowCaster<n>`, `terrainShadowReceiver<n>`, `shadowReceiver<n>` for
    six sets (`0x0031bc90`), on every map.
  - the machines' particle sheets in the map's folder (`0x004a6810`: `mxparticlezones.dat`,
    `atvparticles.dat`, … for each machine), which the game takes over `Part\`'s: MC10, Nat02 and
    SM01–03 opened their own `mxparticlezones.dat`; 15 maps have some sheet.
  - the spawn todo: "where the game starts that Free Ride was not looked up" is gone on MX vs. ATV
    Unleashed (the start is the scene's, seen in the game); a todo stays only where the start is under
    the ground (FS02, FS06, Nat10, Nat14, SM01–03). Untamed's keeps the old text.
  - `tests/disc.rs`: a test for the three pulls; the no-start test now checks the under-the-ground todo.
- **The mods were re-imported** (`out/mods/tracks/`, 56) and every check run on them.
- **The gate**: openmx's live checkout changed `Grove::load` under `tools/renderprobe`; main's fix
  (2d92cf6, T9) was applied hunk for hunk, not merged. C4 on main edited the same spawn `if` in
  `import_into`, so M12's rule sits in its own block after the base text, and the asks went in above
  T7's MD10 entry instead of at the file's end: `git merge-tree` with main is clean, and the merged tree
  builds and passes `import-mxvatv` and `accuracy`'s tests (checked in a throwaway worktree).

## How it was proved

- `heights`: on all 56, every height, surface and corner weight of both of the original's queries equal
  to openmx's at 8,000 points; normals 1–3 ulp off at 61–91 % of the points, every field equal with the
  step. The packed normal table is off on every map.
- The game's terrain box (`+0x30/+0x40`) is the file's leaves on every map (`tests/every_map.rs`), which
  is how the arenas' and Nat04's box differ from openmx's `Ground` (an ask).
- `scene`: all 28–31 fields a map equal, bit for bit, once the text is read as openmx reads it; the sun's
  direction is computed here and comes within 1–2 ulp.
- `files`: every name the game opened for the map is in the re-imported mod, on all 56 (on 7 the pool's
  head was overwritten; see Not verified). Before the fixes Lago lacked 24 (its doors and the shadow
  materials) and MC10 one (its particle zones).
- `cargo test -p accuracy --test every_map` (with `$DISCX_ISO`): 56 maps, ok. `./check.sh` at the end.

## What was tried and did not work

- **Counting presses** in the track list (above).
- **The first `files` pool finder** took the text around the first `env\<map>\<map>.scn` in RAM. On the
  MC maps that was a lone path, and "the longest run of names" picked the PSR's directory (every name on
  the disc) and then the machines' animation list. The pool is names two NULs apart; it is now the run
  holding the scene's name, else the one with the most map-kind names, with its first (joined) name
  dropped.
- **The first fog object** with the fog's vtable was a zone's copy on MC03 and MC09 (distances unset); the
  scene's is the first with a sane distance pair.
- **Rust's float parsing** made Lago's start and haziness one ulp above the game's; the game (and
  openmx's scene reader) truncate. Read through `game_sim::phy::number`, they are equal.
- **Poses from `Ground::extent`** put the arenas' "centre" 26 ft from the corner and the start pose under
  the ground; that is what found the box difference.
- **`./kit shot`** returned a cut-short PNG 27 times in the first 386: it copies PCSX2's screenshot once
  its size holds for 0.3 s, which can be mid-write. Retaking up to three times still left 8 short; the
  files in the copy's `snaps` folder were all whole (1,060 of 1,060), so `shoot` now waits for the newest
  one there to end in `IEND` and copies it.

## Not verified

- **openmx's half of the pairs.** openmx has neither MD2 (a track from a mod folder) nor MD10
  (`--shot-pose`), so `pair` indexes the original's shots alone (T9 takes them). No difference of look
  (props, textures, fog, sky, lights, ecosystem density as drawn) was found by comparing pictures: the
  look was checked through the data the game holds (scene, files), not the image.
- **Ecosystem density** was not compared: no plant count was read from RAM (the ecosystem object's
  layout was not looked for). The `.esb` and its species' models are in each mod (`files`).
- **Lights beyond colour, position and direction** (`ShadowRGB`, `ShadowIntensity`, `CastsShadows`) and
  the `[DistanceFade]` defaults (start 500, end 250, near 1, far 1000, `0x31f200`) were not compared.
- **The pool's head on MC01–05, MC07, MC08**: overwritten by a later file before the capture, so the
  names from the scene's to the first left (some props and shapes) were not seen; the names after it all
  are in the mods. A capture taken earlier in the load would see them.
- **`atvparticles.dat` and the other machines' sheets** are pulled because `0x004a6810` names them for
  their machine; only a bike was ridden, so only the MX names were seen opened.
- **The six shadow sets**: `shadowCaster1`–`6` are in the pool on all 56 maps, and no 7th; the caller
  that passes 6 to `0x0031bc90` was not found (it is called through a vtable).
- **Which activity props a Free Ride shows** (`Activity`, `Hide`, M2's todo) was not settled: the pool
  says what was loaded, not what is drawn.
- **The start under the ground**: seen from the camera's height on FS02, FS06, Nat10 and SM01–03, about
  a minute in (Nat14's rider had rolled away); whether the game drops the rider or places it was not
  looked at.
- **The landmarks** are a heuristic (biggest model placed once); on the arenas they are often the
  stadium shell. The brief's "the map's landmarks" was not picked by hand.
- **Untamed** (M13) was not touched beyond keeping its spawn todo as it was.
