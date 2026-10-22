# M4: tools/import-mxvatv on MX vs. ATV Untamed

`import-mxvatv --disc <Untamed image> --out STAGE` now writes `STAGE/untamed_<map>/` for the 40 maps
Untamed's activity list offers as a Free Ride or a national, each read back by modcheck. It is M2's
importer with M10's decoder in front. The MX vs. ATV Unleashed path is unchanged except for one line
of its binary's output wording. Findings are in `notes/mxvatv-later.md`, *The importer (M4)*.

## What I did

1. **Which maps.** There is no per-map `.atyList`. `Env\Master.atyList.bpb` (BPB, decoded by
   tools/bpb) has a `[<MAP> General]` per map, with `DisplayName` and `Activity` lines naming sections
   that each have a `Type`. "Freeride and national maps" became: every map archive
   (`res\env\<MAP>\<MAP>.res`, 48) whose activities include `free ride` or `national`. That gives 40:
   12 nationals, 6 supermoto, 12 supercross, 4 freestyle, AM_CS/AM_GFUC/AM_JY, MT_A, BT_A, BT_B. Left
   out and printed as `skip`: BT_C to BT_F (opencross only), HC_A (hill climb), SJ_A, SJ_B (ski jump),
   TR_A (a2b race, a 1.6 KB stub scene). SM_IP, TestAG, MT_B and AM_AF have a Free Ride in the list
   but no map archive (SM_IP's terrain is loose, with a stub scene), so `map_folders` never sees them.
   `--map` takes any archive's folder, chosen or not.
2. **Archives.** `Res::parse` refuses every Untamed map archive for the zero bytes after its
   directory (T6). `discx::Disc::res` now falls back to `ResFile::unpadded`, the same cut as
   tools/survey's `res_view`, and records `padding`. The importer writes one todo entry per padded
   archive. modcheck's byte check reads the archives the same way and adds a note.
3. **Scene.** `decode_scenes` (lib.rs) turns each `.scn.bpb` member into `<MAP>.scn` beside it with
   `bpb::parse(..).to_ini()`. It refuses a file whose hashes do not recompute, and refuses a decode that
   would land on an existing member. A todo entry names both files, which is what §2a asks for. After
   that the scene goes through M2's checks unchanged: `Scene::parse`, the terrain lookup
   (`Nat_AZ.trn` → `Nat_AZ.ps2.trn`), layout, collision, the key diff against MXU, game-render.
4. **Other BPB.** Materials (`.psm.bpb`) and toys (`.toy.bpb`) are decoded in memory only, so the
   render check can follow a model's materials to their textures and run `toy::cars`. Every BPB kind
   gets one todo entry saying no openmx reader takes it, and it is a port ask. Each file there is
   decoded and its hashes checked.
5. **Free Ride.** `free_ride_of` is M2's `free_ride` with the `[General]` section name as a
   parameter. Only Nat_MA's Free Ride has a `StartPosition`. FS_A's has `DefaultStartPosition`, which
   the ELF's activity reader (`0x002cb110`) does not read (it is the scene's key, `0x0030e490`). So
   FS_A spawns at its scene's start, and the todo says why.
6. **Loose members (M15's rule on Untamed).** I built a Ghidra project for SLUS_217.01 under this
   worktree's `extracted/untamed/ghidra/` (M10's steps, with s2 paths in `kit.toml`; analyse plus dump
   took about 2 minutes). The resource manager has the same shape as SLUS_211.04's. `0x0011e950` and
   `0x0011eff0` look a name up among the loaded archives, and on a miss `0x0011c7f0` opens the PSR
   member (`0x001bc760`). The off-switch `0x1000` is set only through `0x0011f250` ← `0x00222050`
   (the `.res` loader) when its fifth argument (`$t0`) is non-zero. I disassembled all 43 call sites,
   and each passes `move $8, $zero`. `untamed::pull_loose` follows what the code builds:
   - placed models, `Geo\<slt>.psg` (`0x00186bd0`; the folder is set to `Geo\` at `0x001c27d0`);
   - collision, `Col\<name>.col` (`0x00114f20`);
   - the sky, `Skies\<CubeFile less ext>.ps2.cub` (`0x0030c4d0` `%s.ps2.cub`, `0x0018f230` `Skies\`);
   - plants, `Eco\<model>.psg` (taken from M15, see *Not verified*);
   - the material table `Env\<MAP>\<MAP>.mst` (`0x001c2d20`, `%s%s\%s.mst` with `Env\`), each line
     read as `<line>.psm.bpb` and then `<line>.psm` (`0x00168ff0` → `0x001425c0` opens `%s.bpb`, then
     the plain name, both through `0x0011e950`);
   - their textures: folder + `TextureMap_<n>` (`0x00163930`, `%s%s`, skipping RenderTarget…), tried
     as `.pst`, then `.atm` (`0x0016cba0`). Untamed has no `.pss`/`.tga` step, unlike Unleashed.

## Results on the disc

- 40 maps import and read back: 11,394 files, 356 MB in all, the biggest 12 MB. The 120 archives
  are all padded, by 56 to 2,003 bytes.
- **2,481 loose members pulled (36 MB)**: 1,230 `.psm.bpb`, 1,202 `.pst`, 27 `.col`, 18 `.psg`,
  2 `.cub`, 2 `.atm`. That is 1 (BT_A) to 151 (SX_G) a map. Among them: every map's
  `Env\<MAP>\TerrainLOD.psm.bpb`; Nat_OR's 22 shared `Col\` shapes and 13 `Geo\` props; AM_JY's three
  sign shapes; BT_B's bulldozer and bobcat (model and shape); `Geo\nat04_lightflare.psg` for AM_GFUC,
  SM_AS and SM_HS; and SM_CF's and SX_B's skies (`Skies\sky04.ps2.cub`, `Skies\sky25.ps2.cub`), which
  their archives lack. No plant is pulled, since every species is in its map's archive.
- What is left in todo, beyond the kinds every map has (padding, decoded scene, MD1/MD3, BPB kinds,
  `.mst`, the MXU key diff, Activity/Hide keys on 12 maps):
  - seven placed `.col` that are nowhere on the disc (as T6 found), and AM_CS's two
    `AM_CS_Concretein*.psg`, also nowhere;
  - `screen.psm` (jumbotrons, `TextureMap = RenderTarget`) on the 12 SX maps, the 4 FS maps and MT_A,
    plus SX_D's `skyline01` and SX_G's 14 `alt-02_*` materials, which have no `.psm` anywhere;
  - `Blimp.mot` refused by `skin::Clip::parse` (Nat_CO, SM_S1);
  - `DetailMapTilingMultiplier` 6 (BT_B, SM_MB, SM_OF, SM_S1) and 16 (FS_C);
  - Nat_OR's scales (4.5 in the scene, 4.499999 in the file) and SM_HS's `TerrainWidth=5` over 4 root
    cells, both kept as the disc has them (§2a);
  - FS_A's Free Ride `DefaultStartPosition` and `AllowRadar=FALSE`.

## How it was proved

- `tools/import-mxvatv/tests/untamed.rs` (in the gate, 4 tests). It builds an Untamed-layout disc:
  `MX08PS2.PSR` and `MX08AUD.PSR`, three map archives padded by 300, 57 and 2,003 bytes, a BPB scene,
  BPB materials and toys, a `.mst`, a master list with a second map offering only supercross, five
  loose members the game would open, and six decoys it would not (one is a loose `Tex\Rock.psm.bpb`
  at an archive member's path). The tests check: the choice and the skip reason; the padding counts;
  every member byte for byte; the decoded scene equal to tools/bpb's and read by `Scene::parse`; the
  exact pulled list and order; no decoy copied; the archive winning over the loose copy; mod.json's
  source, members, `loose` reasons, spawn `from`; the todo entries; modcheck passing with the padding
  note; FS_A's `DefaultStartPosition` named and not used; and the binary skipping the second map.
- Ignored, on the user's disc (`DISCX_ISO=… cargo test --release -p import-mxvatv -- --ignored`, 85 s
  for the Untamed pair):
  - `every_chosen_untamed_map_imports_and_reads_back`: 40 chosen, 40 import, `modcheck::check` passes
    each, 2,481 loose members pulled.
  - `every_untamed_col_goes_through_openmx_first`: the 1,604 loose `.col` give 1,602 read and the two
    known refusals (`Col\FS_Panel_Blue01.col` 0 bytes, `Env\Nat_DVCA\Anim\Nat05_Gallows_Door.col`
    class 1); the 1,472 in the 48 map archives are all read.
  - M2's three ignored tests (56 MX vs. ATV Unleashed maps, the M15 pulls, the MXU key table) still pass.
- Mutations, each restored: trying `.pst .pss .tga .atm` (Unleashed's order) pulled the decoy
  `Tex\Gone.tga`, and the test failed. Making `Disc::res` skip the padding fallback failed two tests.
- `./build.sh --check --iso <main checkout's iso> --out <scratchpad>`: 56 MX vs. ATV Unleashed and 40
  Untamed track mods, two music packs, `modcheck: 98 ok, 0 failed`. I deleted the scratch output
  afterwards.
- `./check.sh` passes.

## What failed on the way

- The first standalone `modcheck` run on an imported map failed on the padded archives. The binary
  was stale: I had rebuilt only `discx` and `import-mxvatv`. The import's own read-back, which links
  the new discx, had passed.
- The first pull missed the scene's own models and skies. The first full run showed 13 Nat_OR props,
  BT_B's bulldozer and bobcat, `Nat04_LightFlare` on three maps, and SM_CF's and SX_B's skies, all
  "loose on the PSR". I traced the model and sky loaders (above) and added them.
- My first assertion that "no todo mentions sky1" was too broad: the test's fake sky does not decode,
  and that entry is correct.
- M2's binary test expected an Untamed disc to be skipped. It now uses an unknown boot ELF.

## Decisions

- **One crate, one binary.** The brief names tools/import-mxvatv, and build.sh already runs this
  crate on SLUS_217.01. The Untamed part is `src/untamed.rs`. lib.rs got an `Origin` (game, boot, id
  prefix, archive folder), `load` shared by both games, `Map.ride`, and the BPB hooks. Each hook is a
  no-op on Unleashed's disc, which has no `.bpb`.
- **Only the scene is decoded into the folder**, as §2a says. Decoding materials and toys into the
  folder would mean files that are not the disc's, which the standard does not allow for them. They
  are a port ask instead.
- **The padding fallback is in discx**, not copied a third time into import and modcheck. `ResFile`
  got a `padding` field so every caller can say so. `from_bytes` is unchanged.
- **The standard** (§2, loose members) now says Untamed does this too and points at the notes. No
  field changed.
- **The spawn todo wording** (".atyList's [Free Ride]") is M2's and I kept it for Untamed. The `from`
  names the master list.

## Port asks

`notes/openmx-asks.md` (Open): a BPB reader for everything past the scene (materials, toys,
particles, sounds), and more evidence on the `Res::parse` padding.

## Not verified

- **Nothing was run in PCSX2 (T7).** That covers the spawns (39 maps use the scene's
  `DefaultStartPosition`, and where the game starts their Free Ride was not looked up), and which
  pulled members a Free Ride actually opens. It also covers what the game does with the seven placed
  `.col` that are nowhere on the disc (AM_CS ×2 files, MT_A, Nat_MD, Nat_SDCA, SX_A, SX_G).
- **No mod was loaded by openmx** (no MD2). "Read back" means modcheck and the readers accept the
  files.
- **Plants as `Eco\<model>.psg`** come from M15's reading of MX vs. ATV Unleashed's GAME_REL.BIN.
  Untamed's GAME_REL.BIN overlay was not loaded into Ghidra. Its strings have `EcoSystem.cpp` and
  `.esb`, and the boot ELF keeps an `Eco\` folder string beside `Geo\` and `Tex`, as Unleashed's does.
  On the disc, no species was pulled: every map's plants are in its own archive.
- **The texture folder prefix** (`%s%s` at `0x00163930`): I took its first argument to be the
  `.psm`'s folder, as M15 found in Unleashed. I did not follow the caller `0x0019d1e0` to see what
  it passes.
- **The sky name**: I did not see where `0x0030c4d0` drops `CubeFile`'s `.cub` before `%s.ps2.cub`.
  The rule (stem + `.ps2.cub`) matches every archive's own sky member (`CubeFile=sky25.cub` with
  `skies\sky25.ps2.cub`).
- **`ResourceFileOverride` (flag `0x800`)** was not looked for in Untamed. M15 found its default 0
  in Unleashed.
- **The `.mst`'s `#include`s and the toys', particles' and animated objects' requests** were not
  followed, as in M15.
- **Material textures are checked through tools/bpb's decode**, not through a reader of openmx's on
  the BPB, because none exists.
- **The Ghidra project lives in this worktree's `extracted/`** and is lost with it. Step 6 says how
  to rebuild it (about 2 minutes).
