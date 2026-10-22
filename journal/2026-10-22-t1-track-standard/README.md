# T1: the track mod standard

Result: `standard/OPENMX-TRACK-STANDARD.md` (v1, draft), with §2 track mods (for MD2), §3 worlds (for
C1/MD8/MD9), §4 music packs (for A1/MD7) and §5 how a mod is proved. Port asks: `notes/openmx-asks.md`
(*Open*, "MD2, what the standard asks of the loader").

## What was done

1. **The model.** I read opentennis-mods' `standard/HST-COURT-UMPIRE-STANDARD.md` with `gh api` and took
   its shape: one rule up front, a status line naming the loader steps (there M8f*, here MD1–MD11), a
   `kind` table, then one section per kind with a folder tree, a `mod.json` example and a bullet per
   field. Unknown kinds are skipped quietly, and so are files a step has not landed for.
2. **What the port reads.** From openmx, read only: `game-sim/src/track.rs` (`Scene::parse`: which
   keys, and that `TerrainFile` is the one it refuses without), `game-render/src/scene.rs` (the render
   keys and their defaults), `game-render/src/disc.rs` (`read_suffix`: case and slash-insensitive suffix
   lookup), `openmx/src/main.rs` (the ride's loader: the one `.scn`, `terrain_bytes`' `.ps2.` tag rule,
   the `_2.res` second archive), and by grep the names built from the track stem (`toy.rs`, `splat.rs`,
   `particles.rs`, `audio.rs`, `sky.rs`). Music: `audio.rs`' `vig` reader and `music.pb` playlist.
3. **What a track archive holds.** A throwaway crate in the session scratchpad (not committed),
   depending on `openmx/crates/game-render` by path, opened `openmx/MX Unleashed (USA).iso` read-only. It
   listed `Tracks\free06.res`, `nat01.res` and `sx01.res` by directory and extension, and printed each
   scene's section and key *names* (no values). The scene key table in §2a comes from that run:
   all three scenes use the same key set.

## Decisions (made unattended, with why)

- **Files keep their source names. The brief's `<id>.scn` / `<id>.ps2.trn` hold only for generated
  maps.** T4 names folders `<source>_<map>` (`mxvatv_mc01`), but the scene's `TerrainFile`, `CubeFile`,
  `SLT=` and the stem-built names (`<stem>.toy`, `_shadow_NN.pst`, `particles.prt`) all point at the
  source's own names. Renaming would mean editing the scene, and the accuracy rule says no. So the
  folder's one `.scn` gives the stem, and the id is only the folder.
- **The scene is never edited. Overrides go in `mod.json`.** M2's brief expects the start position,
  terrain name and sky to be rewritten in the scene. The port already resolves `MC01.trn` →
  `MC01.ps2.trn` and `sky.cub` → `sky.ps2.cub` by itself, so only the spawn could need an override, and
  that is `mod.json`'s `spawn` with a required `from`. This keeps every file byte-comparable with the
  disc (§5.1), and that comparison is the proof. M2 should follow the standard and not its brief here.
- **Member paths are kept.** The `.res` is unpacked as it is, `\` → `/`. The port finds files by suffix
  anyway, and keeping paths means unpacking never has to choose a location.
- **Worlds live in `mods/tracks/` with `kind: "world"`**, with whole track folders as `tiles/` and
  T2-written terrains as `joins/`. Tiles are copies, so a world never depends on another mod being
  present. There is no vertical offset (heights stay the tile's own, per the C1 brief). `at` is a
  multiple of the tile's leaf size, so world − `at` is exact. `keep` is in samples, so it is exact too.
  `drop` lists cut props by scene section, so tile scenes stay verbatim. `.blend` is N bytes a sample
  summing to 255 (MD9's fade). `sky` is a tile name (C1 brief / MD9: global, default the first map's).
- **Music `songs` live in `mod.json`**, each with a `from`, and the disc's own playlist file is copied
  beside them for proof.
- **The hash function is left to MD6.** The standard fixes only which files enter the hash.

## What did not work, or was corrected

- My first draft said game-render reads `DetailMapTilingMultiplier`. It does not: `terrain.rs`
  hard-codes 8 ("8 on every track on this disc"). The table now says the game reads it and the port does
  not yet.
- My first draft had `<stem>.aud` as a suffix lookup. `audio.rs` reads the exact path
  `tracks\{track}\{track}.aud`, which an MX vs. ATV mod (`Env\…`) would miss. That is now a port ask.
- I first wrote "every disc scene has" for `TerrainBorderWidth`/`MinDivisionSize` and both start keys.
  I checked only three scenes, so the text now says those three.

## Not verified

- **No mod was built or loaded.** MD2 does not exist yet (its test is to build a mod by this standard),
  and T4/M2 are later tasks. Every rule about what "the port does" with a mod is a requirement on MD2,
  not observed behaviour.
- The scene key set was read from three MXU scenes (free06, nat01, sx01). The other MXU scenes and
  every MX vs. ATV scene were not listed. MX vs. ATV may use keys that are not in the table. They fall
  under "anything else: kept, ignored", but one the port should read would be missed.
- The MX vs. ATV folder example (`Env/MC01/…`, `Geo/`, `Col/`, `mc01_mat.res`) is from
  `notes/mxvatv.md`, not from a listing. The MX vs. ATV disc is in the main checkout's `iso/`, which this
  worktree must not touch.
- Whether two archives of one map (`<map>.res`, `<map>_mat.res`) ever hold the same path with different
  bytes was not checked. The standard says the import then stops.
- The `.atyList` spawn key in the §2b example (`[Freeride] StartPosition`) is illustrative. M8 has not
  read an `.atyList`.
- The music example's file names and the `music.pb` entry format for MX vs. ATV are placeholders. A1
  reads them. Untamed's playlist format is unknown until M10.
- That world − `at` is exact in singles when `at` is a multiple of the leaf size was reasoned, not
  tested. Far from the origin a single has fewer bits below the point, so positions there are coarser
  whatever `at` is. MD8 is asked to check that.
- `previews/track.png`'s size is left to P70/MD11. The P70 code was not read.
