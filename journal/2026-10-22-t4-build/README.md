# T4: build.sh, and tools/modcheck to prove what it builds

## What it does

```
./build.sh                 import and package into out/mods/
./build.sh --check         the same, then prove everything in out/mods/ (standard §5)
./build.sh --check-only    prove out/mods/ as it is
    --iso DIR (default iso/, or $DISCX_ISO)   --out DIR (default out/)
```

- **Discs** are found by boot ELF through `discx discs`, never by file name.
- **Steps** are one table in `build.sh` (`STEPS`): when (`disc`, `data`, `after`), the boot ELF a disc
  step needs, the crate, where its mods go (`tracks` or `music`), and the task that writes the crate.
  Today it lists `import-mxvatv` for SLUS_211.04 (M2) and SLUS_217.01 (M4), `import-music` for both (A1),
  `elevation` (M1) and `stitch` after the rest (C1). None of these crates exists yet, so every step says
  `skip …: tools/<crate> does not exist yet (<task>)`. MX Unleashed has no step: its tracks are the
  port's own, and a mod of one is only a world's tile (§1).
- **The importer contract** (in `build.sh`'s header, for M2, M1, A1 and C1 to follow): the binary is
  the crate's name, run as `<crate> [--disc IMAGE] --out STAGE [--mods OUT/mods]`. It writes
  `STAGE/<id>/` with a `mod.json` per mod. build.sh moves each folder to `out/mods/<dest>/<id>/`,
  replacing the old one whole so no stale file survives a rebuild. A step that fails is reported, and
  none of its output is packaged. The run goes on and exits 1. A folder without `mod.json` is not
  packaged.
- **Packaging** (unpacking the `.res`, merging the material archive, choosing maps, names, `todo`) is
  left to the importers, as M2's brief has it. build.sh only stages and places the folders.
- cargo's output is shown only if the build fails, so openmx's own warnings don't show on every run.

`tools/modcheck` (`modcheck [--iso DIR] [--quiet] PATH...`, a lib and a bin) is §5's check:

- **§1** `mod.json`. It must parse (a small JSON reader in `src/json.rs` refuses duplicate keys and
  trailing commas; the workspace still takes no crates.io dependency). Every required field must be
  there: `standard` 1; `id` equals the folder, is `[a-z0-9_]{1,40}` and has the game's prefix; an
  `mxu_` id only as a world's tile; `world_` if and only if `kind` is world; `source.game` is one of the
  table's names; `boot` + non-empty `members` (and the boot is that game's), or `data` + `date`;
  `version` is semver; `todo` is a list of strings, and `[]` only with `checked`; `spawn` has
  position, direction and `from`.
- **§2** One `.scn` (the stem is the id for `elev_`), read by `track::Scene::parse`. Its
  `TerrainFile` is looked up as the shell does: the name as written, then with `.ps2` inserted. The
  lookup is on the path's end, case and slash folded, on a `/` boundary, and a name that finds two
  files fails. The terrain is read by `track::Terrain::parse`, or through the MD1 view (below), which
  is said in a note. The scene's layout is checked against the terrain (below), and every leaf in the
  root cells' rectangle is walked with the game's descent (`Terrain::leaf`, `block`). Every distinct
  placed `COL=` must be read by `track::Collision::parse`, or else be named in `todo`. Paths that
  differ only in case fail.
- **§5.1** For a disc mod, the disc with `source.boot` is found in `--iso`, every PSR of that game is
  opened (Untamed has two), and each `source.members` pattern (discx's glob) is expanded. A `.res`
  is compared member by member at its path; any other member is compared as itself. Both reads go
  through `game_sim::archive::{Psr, Res}`. Every file in the folder that no member accounts for must
  be named in `todo`. If the disc is not present, the mod fails ("cannot prove the copy"); it is not
  skipped.
- **§3 worlds**: every tile is checked as a track (as a tile, so `mxu_` is allowed). Each join's
  terrain is read, each `.blend` must sum to 255 per sample, and each `drop` must name a section the
  tile's scene has; `sky` and `spawn.tile` must be tiles. Seams and `keep` are C1's `--check`, which a
  note says.
- **§4 music**: each song has file, title, artist and from, and its file must be there. `Adpcm::vig`
  is not run (below).

Every problem is a `fail`. Anything checked more weakly than the standard asks is a `note`, printed
under the mod, so nothing is skipped silently.

### The MD1 view

`Terrain::parse` refuses every MX vs. ATV terrain (MD1: texture pages between the blocks and the tree).
`modcheck::terrain::md1_view` walks the pages as the game's loader does: a format word, a mip count of
1 to 6, and per mip a 16-byte header whose third word is the pixel length. Each page must start at the
header's own offset word, there must be at most 128 pages, and the last must end exactly at the tree.
It then hands `Terrain::parse` a copy of the same length. In the copy the header's texture-word count
grows by gap/4, the blocks move up so they end at the tree, and the gap they leave becomes those
zero words. The tree and its absolute offsets are not moved, so every node and block the parser reads
is the file's own byte. T6's brief asks for the same until MD1 lands. This is a separate copy, local to
this crate.

## How it was proved

- `tools/modcheck/tests/disc.rs` builds a disc by hand: ISO 9660, SYSTEM.CNF `SLUS_211.04` and
  MXVSATV.PSR, with discx's own builders copied in. On it is a track `.res`: a scene; a 2-level PLN v7
  terrain with 2 texture pages in the gap; a mesh `.col`; a class-1 `.col`, which openmx refuses; a
  sky. There is also a `_mat.res` that shares the sky path with identical bytes. Ten tests:
  - the MD1 view gives `Terrain::cell` identical to the same terrain written without pages, at five
    samples across two root cells, and leaves the tree byte-identical. A plain terrain gets no view,
    and a wrong page-offset word is refused.
  - the packaged map passes, with the expected notes.
  - one flipped byte fails, and so does a deleted file.
  - an extra file fails until `todo` names it.
  - a case-twin file fails.
  - the class-1 `.col` fails unless `todo` names it, and so does a missing `.col`.
  - `TerrainWidth=2` over a 1-cell terrain fails unless `todo` names the scene and the key.
  - each `mod.json` violation fails: a wrong id, `todo` [] without `checked`, empty members, a member
    not on the disc, a spawn without `from`, a wrong prefix, bad JSON.
  - with no disc present, the mod fails.
  - the binary on a tree of two mods prints one ok and one failed, and exits 1.
  - plus two JSON unit tests.
- Mutation checks, both restored afterwards. Moving the blocks 4 bytes short in the view fails the
  view test. Comparing copies by length only fails the differs test.
- **Real disc, end to end**, read-only from the main checkout's `iso/` (this worktree has no `iso`
  link). I wrote a scratch `tools/import-mxvatv` crate that unpacks every `Env\*\*.res` (skipping
  `_mat`/`_mst`) with its `_mat.res` into one folder per map, the way §2 describes, and deleted it
  afterwards; it was never committed. `./build.sh --check --iso …` then built and checked all
  **56 MX vs. ATV Unleashed maps**:
  - all 56 terrains read through the MD1 view. Nat01, for example, has 48 pages, 3,146,880 bytes.
  - every file in all 56 folders is byte-equal to its member.
  - **41 pass outright.** The other 15 are the source as it is, which M2 has to list in `todo`:
    - 17 distinct `COL=` names that the scenes place but that neither archive holds. These are
      `NatStartGate.col` and `NatDropGate.col` (3 maps each), `Anaheim_pole.col` (2),
      `trash_can.col`, `recycling_can.col`, `SingleBale01.col`, `Ring_Trigger.col`,
      `Nat_tray_runnoff.col`, `Nat12_Sphinx.col`, `Nat10_tray_water.col`, `nat10_MailboxOpen.col`,
      `nat10_Mailbox_Closed.col`, `Nat07_churchsnow.col` and `Dino_Egg_A/B/C.col`.
      `Env\Nat01\trash_can.pco` and `recycling_can.pco` sit loose in the PSR beside Nat01's archives.
      There is also `Col\trash_canf.col`/`trash_canm.col` loose in the PSR, which may be what the game
      uses. This is for M2/T6 to find out.
    - scenes that disagree with their own terrain: Nat04 has `TerrainWidth=5` over 4 root cells, with
      640 leaves of its 4×5 rectangle holding no block. SX07 has `TerrainBreadth=2` over a 256-sample
      terrain. MC08 and SX18 have `TerrainWidthScale=3` and `TerrainBreadthScale=3` over 3.5-ft
      terrains.
  - mutations on Nat01's folder each failed with the expected message: a byte flipped in
    `Col/RockA.col` (first at byte 40), a case twin `Col/rocka.col`, a removed `Col/boulderB.col`
    (the scene says `BoulderB.col`, which the case-folding lookup found), `TerrainWidth=5→4` in the
    scene, and an `--iso` without the disc.
- `./build.sh --check --iso <main iso>` with no importers lists the four images (Hot Shots Tennis as
  SCUS_976.10, not ours), skips every step with its task, and exits 0. `./build.sh --check` with the
  default `iso/`, absent here, says so and exits 0. This is the task's test as it can run today.
- `./check.sh` passes (20 tests).

## What failed on the way

- **Root cells ≠ TerrainWidth.** First I checked `TerrainWidth` against the root cells the tree uses,
  as the standard's table says. 29 maps failed (18 SX, 8 FS, 3 MC). They have a one-level
  tree: 16 root cells of 16 samples, and `TerrainWidth=1`. `TerrainWidth` is 256-sample units (the
  edge is width × 256 × scale), and it is the root-cell count only when the tree has two levels. The
  check now compares samples. The standard's wording is filed as **T8**.
- **Source disagreements.** A disc scene that disagrees with its terrain cannot be fixed, since the
  scene is never edited (§2a). So such a fail becomes a note only when a `todo` entry names both the
  scene file and the key. T8 also asks the standard to say this.
- Holes in a terrain's leaf grid (Nat04) were a fail at first. They are the source's own tree, and
  the game's query finds no ground there, so now they are a note.
- `set -e` and a `[ … ] && …` as the last command of a `while` body would have ended the script on
  the first crate that doesn't exist. Rewritten as `if`.

## Port asks

Added to `notes/openmx-asks.md` (Open):

- MD2: the loader's suffix lookup should match on a segment boundary.
- MD7: the `.vig` reader is in the shell crate (wgpu, ggrs, iroh), so §5 step 4 cannot be run from here.

## Not verified

- **No real importer exists** (M2, M4, A1, M1, C1 are all open), so `build.sh` has packaged a real map
  only through the scratch importer described above. The importer contract (the arguments and the
  staging) is a proposal that those tasks have not yet used. The `after` step (`--mods`) and the
  `data` step have never run.
- **Untamed and MXU through modcheck**: no Untamed map was packaged. Its scenes are `.scn.bpb` (M10), so
  `Scene::parse` has nothing to read yet. MXU's terrains, nationals and supercross with pages, did not
  go through the MD1 view. Only MX vs. ATV Unleashed's 56 and the synthetic terrains did.
- **World and music checks** ran on nothing real, and the tests do not cover them either. They are
  written to §3 and §4 but unexercised. Seams, slopes and `keep` are C1's, `Adpcm::vig` is not run
  (port ask above), and the `.blend` length is not checked against the join terrain's sample count,
  because §3 does not say whether a side has N×16 or N×16+1 samples.
- **`at` as a whole leaf multiple** (§3) is not checked.
- **The identity hash** (§1, MD6) is not computed: MD6 chooses the function.
- The extent check compares sample counts, and the scale with `f32` equality. It does not check that
  the root cells form a full rectangle beyond counting leaves without a block.
- `build.sh` with a symlinked `out/` (the runner's `links`): this worktree has none, so out/ was a
  plain folder, created and then deleted for the runs above.
- Disc images over 4 GB and Untamed's second PSR (`MX08AUD.PSR`) as a `members` source: the second is
  opened, but no mod drew on it.
