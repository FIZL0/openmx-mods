# M2: tools/import-mxvatv, MX vs. ATV Unleashed's maps as track mods

## What it does

```
import-mxvatv --disc IMAGE --out STAGE [--map NAME,...] [--no-check]
```

This is build.sh's importer contract (T4). For every `Env\<MAP>\<map>.res` on the disc (56 maps) it
writes `STAGE/mxvatv_<map>/`:

- **Files**: every member of `<map>.res`, `<map>_mat.res` and `<map>_mst.res`, at its own member path
  (`\` becomes `/`, spelling kept), byte for byte (§2). If a path is in two archives with the same bytes,
  it is written once. Different bytes stop the import and name both members, and so does one path spelt
  two ways. Neither happens on the disc. A failed import leaves no folder behind. I took `_mst.res` too,
  following "everything the source's archives hold is copied". It is 12 KB of `MST\*.mst`.
- **Not copied**: the loose PSR members under `Env\<MAP>\` (race `.tdf`, AI, `.cs`/`.rpy` cameras,
  the activities' `.aty`, `Anim\`). One todo entry counts them by kind. Loose files at an archive
  member's path are compared with it, and all are the same bytes.
- **mod.json**: the fields §1 requires; `name` from the `.atyList`'s `DisplayName` (M8); and two
  importer fields, which §1 allows: `map` (the PSR folder) and `lookups`, the files that the scene's
  `TerrainFile`, `CubeFile`, `EcosystemFile` and `<stem>.toy` resolve to by the standard's lookup.
  The brief says "rewritten (start position, terrain file name, sky)". The standard (T1, written after
  the brief) says the scene is never edited. So nothing is rewritten. The start goes in `spawn` (§2b).
  The terrain and sky names need nothing, because §2's lookup already tries `<name>.ps2.<ext>`, and
  `lookups` records what that lookup finds.
- **spawn**: the `.atyList`'s `[Free Ride]` (the activity named in `[General]` with `Type = "free
  ride"`) `StartPosition`/`StartDirection`, written as the file has them, with `from`. Only MC01 and
  FS05 have one. The other 54 get a todo entry: the scene's `DefaultStartPosition`, and it is not known
  where the game starts that Free Ride.
- **todo**, one entry per gap, each naming the file and why:
  - game-sim: the MD1 view (every terrain), scene/terrain disagreements (§2a, worded so that
    modcheck's scene-and-key match finds them), each `COL=` that is missing (and where on the PSR it is
    loose, if anywhere) or refused, MD3 above free06's 48×48 leaves, and `DetailMapTilingMultiplier` ≠ 8
    (MD4).
  - **the scene's keys against every MX Unleashed scene's**: `src/mxu_keys.rs` holds the 53
    (section kind, key) pairs that MXU's 57 scenes use. Each key outside that list gets one entry,
    with its section count and commonest values. For `[Model]`/`[Object]` the entry says that the
    port draws or places them all.
  - game-render: the sky (`Sky::load`'s lookup, then `Sky::parse`), the ecosystem
    (`Ecosystem::parse` with `eco::extent`), every placed prop, plant species and toy car through
    `psg::Model::parse`, every material those name through `Material::parse` and
    `world::Assets::resolve` (as the port finds a texture), the billboards as `Grove::new` names
    them, toy cars through `toy::cars` and `skin::Clip::parse`, and every `.psg`/`.pst`/`.atm`/
    `.cub`/`.mot` in the folder through its reader. A missing file says whether it sits loose on the PSR.
    A missing texture whose name, read without its spaces or its `.dxm` tail, is in the archive says so.
  - one entry per kind that no game-sim or game-render reader takes (`.mst`, `.pco`, `.prt`, `.mcf`,
    `.vut`, `.emt`), `<stem>.aud` (MD2), and the Free Ride's `ATYFile` and `AllowRadar=FALSE`.
- **Read-back**: after each map, `modcheck::check` runs against the same disc (§5 steps 1–2). A map that
  fails is removed and named, and the exit is 1. `--no-check` skips this.
- A disc that is not SLUS_211.04 gets one line and exit 0, because build.sh also runs this crate on
  Untamed (M4's). The boot ELF is read before any PSR is opened.

What it found on the disc is in `notes/mxvatv.md`, *The importer (M2)*. Port asks are in
`notes/openmx-asks.md` (Open): the Activity/Hide keys, `EcosystemFile` with no extension, texture names
with a space or `.dxm`, Nat12's and Nat16's `.psg`, and the animated objects' `.mot`.

## How it was proved

- `tools/import-mxvatv/tests/disc.rs` builds a disc (ISO 9660, SYSTEM.CNF, MXVSATV.PSR, using
  modcheck's test builders copied in). On it is a track `.res`: a scene with MX vs. ATV-only keys,
  a paged terrain, a mesh `.col`, a class-1 `.col`, an `.aud`, a `.prt`, and a sky that does not
  decode. There is also a `_mat.res` that shares the sky, a loose scene copy, a loose `.tdf`, a loose
  `Col\Loose.col` that the scene places, and an `.atyList`. Nine tests:
  - the import copies every member exactly and the shared sky once, and copies nothing loose. It checks
    mod.json's fields, the spawn and `from`, the lookups, and each expected todo entry. Things that agree
    must not appear in todo. Then modcheck passes it.
  - a scene that disagrees with its terrain is kept byte for byte, named in todo, and passes modcheck.
  - differing bytes at one path, and one path spelt two ways, stop the import, and no folder is left.
  - no `.scn` and two `.scn` are refused.
  - a commented-out start gives no `spawn`, plus a todo entry. With no `.atyList` the name is the folder's.
  - `free_ride` reads numbers as written (an `f` suffix dropped) and refuses `.5`, `01`, `1e3`, two
    numbers and words. It ignores a `[Free Ride]` that `[General]` does not list.
  - the scene key set is taken by section kind.
  - the binary imports and reads back, rejects an unknown `--map`, and skips an Untamed disc.
- **Ignored, against the user's discs** (`DISCX_ISO=… cargo test --release -p import-mxvatv --
  --ignored`, both pass, 33 s):
  - `every_freeride_map_imports_and_reads_back`: all 56 maps (all offer Free Ride, M8) import, and
    `modcheck::check` passes each against the disc.
  - `mxu_scene_keys_are_the_discs`: the 57 scenes in MX Unleashed's `Tracks\*.res` give exactly
    `src/mxu_keys.rs`. I first derived the table with a throwaway Python script in the scratchpad.
- `./build.sh --check --iso <main checkout's iso> --out <scratchpad>`: 56 mods into `out/mods/tracks/`,
  `modcheck: 56 ok, 0 failed`. The Untamed step gives 0 mods and passes. `--check-only` exits 0.
  The scratch output was deleted afterwards.
- Mutations, each restored. Dropping the layout todo made Nat04 and SX07 fail the read-back, and the
  disagreement test fail. Skipping the byte compare in `unpack` made the differing-archives test fail.
- `./check.sh` passes.

## What failed on the way

- The first run flagged `Nat03BB.tga` as a material with no `.psm`. `Grove::new` names a billboard's
  material without its `.tga`, so the importer now does the same.
- `EcosystemFile=mc02` found nothing, but `MC02.esb` is in the archive. The importer now also tries
  `<name>.esb` and says that the port does not (port ask).
- Nat12's and Nat05's "missing" textures were spelling: `Nat12 _genericsandstone` (a space) and
  `Nat05_liverystable_detail.dxm`. The todo entry now names the file the archive does have.
- The binary opened the PSR before checking the boot ELF, and an Untamed image has no `MXVSATV.PSR`, so
  it failed. It now reads the boot through `discx::images` first.
- `discx ls mxu 'Tracks\*\*.res:…'` found nothing. MXU keeps its track archives at `Tracks\<t>.res`,
  one level up.

## Decisions

- **Loose PSR members the archives lack are not copied**, even where a scene or ecosystem names them
  (Nat02's ten `Eco\*.psg`, `Tex\nat11eco.psm`, …). The standard defines the folder as the archives
  unpacked. Copying them would guess that the game loads them, and nobody has checked. The todo says
  where each one is. Filed as **M15**.
- **Activity-specific placements are kept and only named.** §2a forbids editing the scene, and v1 has
  no way to drop a track's sections (only a world's `drop`). Which ones a Free Ride hides needs the
  ELF. Filed as **M14**.
- `EcoAmbientScale`/`EcoDiffuseScale` get a todo entry even at 1,1,1, so the list stays the plain
  diff and does not depend on my guess at what a scale of 1 means.
- MD3's todo uses "more than free06's 48 leaves a side", the largest freeride terrain the port draws today.

## Port asks

Added to `notes/openmx-asks.md` (Open): MX vs. ATV's placement keys; `eco` and an extension-less
`EcosystemFile`; `Material::parse` texture names (spaces, `.dxm`); `psg::Model::parse` on Nat12's plants
and Nat16's `northernlights.psg`; `skin::Clip::parse` on the animated objects' `.mot`.

## Not verified

- **Nothing was compared with the game running** (T7, M12). That covers whether the 54 scene spawns
  are where the game starts a Free Ride, whether the game hides the Activity/Hide placements, whether
  it reads loose PSR members, whether it adds `.esb` to `EcosystemFile`, and whether it strips spaces
  and `.dxm` from texture names. All of these come from the files alone, and no ELF was decompiled
  (SLUS_211.04 has no Ghidra project).
- **No mod was loaded by openmx.** The port has no mod loader yet (MD2). "Read back" means that
  modcheck and the game-sim/game-render readers accept the files. It does not mean that anything was
  drawn or ridden.
- **game-render's terrain reader** (`terrain::Terrain::parse`) was not run. The terrain went through
  game-sim's reader via modcheck's MD1 view, as the standard asks. Particles (`.prt`, in the shell
  crate) and `.mst` were not read by anything.
- **The material path is exercised only on the disc.** The generated test has no valid `.psg`, so
  `Assets::resolve` and the spaces/`.dxm` hint run only in the ignored test.
- The scene-key diff compares key names only. It does not compare values against MXU's ranges, and a
  key that MXU has but a scene lacks (e.g. no `EcosystemFile`) is not listed, because the port's default
  for it is MXU's.
- That `EcoAmbientScale=1,1,1` changes nothing is assumed, not checked (it is listed anyway).
- M8's note says "the 33 big ones" (Nat01–17, SM01–03, MC01–05, MC09–10), but that list is 27 maps,
  and the importer's own count is 27. I did not change M8's text.
