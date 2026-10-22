# M15: does MX vs. ATV Unleashed load a map's files from loose PSR members?

**Yes.** When none of the loaded archives holds a name, SLUS_211.04's resource manager opens the PSR
member at that path. `tools/import-mxvatv` now follows the requests a map makes past its archives,
copies the loose members they reach, and lists them in `source.members`. The addresses and the rule
are in `notes/mxvatv.md`, *Loose PSR members (M15)*.

## What I did

1. **Ghidra for SLUS_211.04**, set up the way M10 set up Untamed's, under this worktree's
   `extracted/mxvatv/ghidra/` (git-ignored): a `kit.toml` with absolute project and context paths, the
   boot ELF copied out with `bsdtar` to `context/elf/boot.elf`, then `KIT_CONFIG=… ./kit ghidra analyse`
   (1m19s) and `dump`. The ecosystem code is not in the boot ELF but in **GAME_REL.BIN**, an `MWo3`
   overlay. Its header gives the load address `0x0045c580`. I first tried offset 0x80 (after the
   header) and found no references to its strings. Mapping the whole file, header included, there gave
   56 references to the overlay's own source-file name. So it is `[[overlay]] load = "0x45c580"` with
   no offset, `./kit elf --program game --root <dir with GAME_REL.BIN>`, then analyse and dump for
   `game`.
2. **The loader**, found from `MXVSATV.PSR` (`0x00155300`), and from `ResourceFileOverride`, which
   turned out to be a flag on the resource manager (`0x003ea990`). On a miss in its hash of loaded
   archive members, the manager opens the PSR member by its full path, ignoring case (`0x0011dcf0` →
   `0x001c6a70`). The one switch that turns this off (`0x1000`) is never set, because no caller of the
   `.res` loader passes the argument that sets it. The notes list every address.
3. **What a map asks for.** I traced each kind of request the brief names to the name the code builds:
   - plants as `Eco\<model>.psg`, while the eco loader swaps the model folder to `Eco\`;
   - materials from the map's own Material String Table, `MST\Env\<MAP>.mst`, each parsed from
     `<line>.psm`;
   - textures from each pass's `TextureMap_n`, prefixed by the `.psm`'s folder and tried as `.pst`,
     `.pss`, `.tga`, `.atm`;
   - collision as `Col\<name>.col`.
   Every one goes through the manager, so each gets the PSR fallback.
4. **`pull_loose`** in `tools/import-mxvatv/src/lib.rs` follows those chains on each map's own files
   and copies what only the PSR holds (loose). The pulled members go into the folder at their member
   path. They also go into `source.members` (modcheck's byte check already reads non-`.res` members)
   and into a new mod.json field, `loose`, that says why each was pulled. They are left out of the
   "N loose members not copied" todo count. The standard (§2) got one rule saying such members are
   copied.
5. Side findings, in the notes: the game swaps or adds `EcosystemFile`'s extension to `.esb`, so
   `EcosystemFile=mc02` works there. M2's todo said this was not looked up; it now gives the address.
   The texture loader cuts a name at its last dot, so `….dxm` becomes `….pst`.

## Results on the disc

- 56 maps import and read back through modcheck (`--ignored`, 102 s). **3,490 loose members (44 MB)
  are pulled**: 1,735 `.psm`, 1,679 `.pst`, 66 `.atm` and 10 `.psg`, from 5 to 159 a map.
- The members the brief names are among them, pinned by a new ignored test: Nat02's ten `Eco\` plants,
  and `<map>bb.psm` and `<map>eco.psm` for MC03/04/05/09/10, Nat13, SM01 and SM03, with their textures.
  Every map's `TerrainLOD.psm` and `<MAP>_d.pst` detail map come through the same rule.
- Compared with the import before the change, the todo lost exactly the nine entries that named these
  files as "loose on the PSR". The only other change is the recounted "loose members not copied" line.
  No pulled model or texture is refused by a reader.
- None of the pulled paths is in an archive outside `Env\`. 1,075 are in no `.res` at all, and 2,415
  only in other maps' archives. So no shared archive the game might also have loaded answers first.
- **The 16 missing `.col`** are on no part of the PSR, so there is nothing to copy. The collision
  loader returns 0 for them and builds no shape.

## How it was proved

- `loose_members_the_game_opens_are_copied_and_listed`, on a generated disc. It has an extension-less
  `EcosystemFile`; an `.esb` with one species only loose and one in both the archive and the PSR (with
  different bytes); and a `.mst` with an `#include`, a comment, an archive material, a loose material
  whose texture is loose only as `.pss` and `.tga`, a material with no `.psm` whose `.tga` is loose,
  and a name that is nowhere. Decoys sit loose too: an unnamed `.pst` and `MST\Global.mst`. The test
  checks the exact pulled list and order, the bytes, that the archive's `Eco\Shrub.psg` wins, that no
  decoy is copied, `source.members`, the `loose` reasons, that the plant's "missing" todo is gone, and
  that modcheck passes.
- Two mutations, each restored afterwards. Trying `.tga` before `.pss` failed the test with the wrong
  file pulled. Dropping the archive-first lookup failed it too.
- `the_loose_members_the_maps_lack_are_pulled` (ignored, on the disc) and
  `every_freeride_map_imports_and_reads_back` (ignored) both pass, as does `mxu_scene_keys_are_the_discs`.
- `./check.sh`: ok, 51 passed. Clippy shows nothing new in this crate's new code.

## What failed on the way

- I first mapped GAME_REL.BIN from offset 0x80; it is loaded whole (see 1).
- My first count of "pulled `.psm` some file names" came out 0 of 1,735. My Python script took
  `basename` of `Tex\X.psm`, which does not split on `\` on Linux. Fixed, the count is 1,676.
- I took the detail map `Nat01_d.PST` for code-named and missed by the rule. It is in fact pulled, named
  by the archive's `TerrainPlain.psm` (`TextureMap_2 = NAT01_d`).

## Decisions

- **The whole material table, not only what is drawn.** The game parses a listed material the first
  time something draws with it. Telling which ones a Free Ride draws needs every consumer (models,
  terrain, particles, toys, UI) or T7. I copy every material the map's own `.mst` lists. The table is
  the map's declaration of its materials, and an unused file costs the port nothing. 59 of the pulled
  `.psm` are named by no file in the mod, `TerrainLOD.psm` among them (named by code).
- **`#include`d tables are not followed.** They are `MST\Global.mst` and others, shared by every map
  and served by the global archives. The map's own lines are what reach for its files.
- **Member spelling is kept**, so a mod may hold both `Env/mc03/` (the archive's spelling) and
  `Env/MC03/TerrainLOD.psm` (the PSR's). The standard keeps spellings, and modcheck compares at exact
  paths. The files differ, so the suffix lookup is not ambiguous, and modcheck passes all 56.
- No port ask: the pulled files are ordinary members that the port's existing readers take.

## Not verified

- **Nothing was run in PCSX2 (T7).** Everything here comes from the ELF and the overlay read
  statically. That includes the claim that the fallback really fires during a map load, and which of
  the 3,490 pulled members a Free Ride actually opens. Some materials in a table may never be drawn.
- **The `ResourceFileOverride` INI**: I did not find which file it is read from. I took its default of
  0 from the getter's default argument. If a file on the disc set it to 1, loose members would win even
  over archive members. Of the loose members beside a map that share an archive member's path, all have
  the same bytes (M2), so for those the result would not change.
- **The scene placement's handling of a `.col` that fails to load** (16 on the disc) was not followed
  past the collision loader's `return 0`.
- **Other callers that might name loose files**: animated objects (`Anim\`), toys, particles,
  scene-placed models, and the `#include`d tables were not traced. M9 found no placed model needing a
  loose file, but the rest were not checked.
- **`.atm` animated textures** were not opened to see whether they name further frame files.
- **The INI getter's handling of spaces and tabs in values** (`TextureMap_2\t\t= NAT01_d`) was taken to
  match game-render's `ini::parse`. It was not compared with `0x00144b50`.
- **The Ghidra project lives only in this worktree's `extracted/`.** Rebuilding it takes about 3
  minutes, with the steps in 1.
