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
- **New: `Res::parse` and Untamed's map archives** (T6, `notes/mxvatv-later.md`). All 48
  `res\env\<MAP>\<MAP>.res` on the Untamed disc are refused: `res\env\FS_A\FS_A.res`: `.res: the header
  claims a 1730-byte directory, 1470 bytes were read`. The directory's 45 entries are followed by 260
  zero bytes, which the header's directory length counts. On the other maps the padding is 56 to 2,003
  bytes. The blocks start after the padding. Fix: after the last entry, accept zero bytes up to the
  claimed length. `tools/survey` reads them now through a view without the padding.
- **Untamed's binary INI (BPB) past the scene** (M4, `notes/mxvatv-later.md`, *The importer (M4)*).
  An Untamed mod holds its scene twice: the disc's `<MAP>.scn.bpb` and the `<MAP>.scn` the importer
  decodes beside it (§2a). Every other INI of the map is BPB only, and nothing in openmx reads BPB:
  its materials (`res\env\Nat_AZ\Nat_AZ_mat.res:tex\nat_az_bowsaw.psm.bpb`, 24 to 130 a mod, counting the loose
  ones pulled), its toys (`env\nat_az\Nat_AZ.toy.bpb`; `toy::Traffic::load` looks for `Nat_AZ.toy`),
  its particles (`part\SX11particles.prt.bpb`) and its sounds (`Nat_AZ.aud.bpb`). So a mod draws every
  Untamed model untextured and runs no traffic. The textures are there: read through tools/bpb's decode,
  `world::Assets::resolve` finds a texture for every material but the `screen.psm` jumbotrons
  (`RenderTarget`). Fix, either: game-render reads a `<name>.bpb` where it would read `<name>` as text,
  as the game's loader does (SLUS_217.01 `0x001425c0`: `%s.bpb` first, then the text; format and hash
  in `notes/mxvatv-later.md`, *Scenes: binary (BPB)*); or the standard lets an importer decode every
  BPB beside itself, as it does the scene.
- **`Res::parse` and Untamed's padding, more evidence** (M4): the 120 archives of the 40 imported maps
  are all padded (56 to 2,003 bytes); `tools/import-mxvatv` and `modcheck` open them through
  `discx::ResFile::unpadded`, the same cut as `tools/survey`'s, and each mod's `todo` names the padding.
- **MD5, more evidence** (T6): on MX vs. ATV Unleashed, `collision class 1 is not understood` is every
  animated object, and only those: 7 `.col` in the map archives and 15 loose, all under `Env\<MAP>\Anim\`
  (`Nat04_FerrisWheel`, `Nat05_Gallows_Door`, `Nat10_TireSwing`, `PackCamel`, `bigfoot`, …; the full list
  is in `notes/mxvatv.md`, *Survey tables*). Untamed has one, `Env\Nat_DVCA\Anim\Nat05_Gallows_Door.col`.
  MX Unleashed's own disc has more than the one file that `game-sim/src/track.rs`'s comment mentions:
  `survey mxu` finds 11 in the track archives, 8 distinct. Three are under `anim\` (`free10\anim\shovel`,
  `nat07\anim\mill`, `nat11\anim\gondola`) and five under `col\`: `watertower`, `mainhouse` (nat01),
  `cemeterygate` (nat02), `utilityshack1` (free09, nat06), `nat08smallshanty`. Whether nat01's scene
  places those was not checked.
- **MD2, lookups on a segment boundary** (T4, `tools/modcheck`): `game-render::disc::Res::find_suffix`
  matches a plain string suffix, so `RockA.col` would also find `Col/xRockA.col` if that came first. On
  the disc's own `.res` the order hides it; a mod folder has no member order. `modcheck` matches on a
  `/` boundary and fails a name that finds two files. Fix: the same in the mod loader's lookup.
- **MX vs. ATV scene keys the port does not read** (M2, `notes/mxvatv.md`, *The importer*). On the 27
  big maps, `[Model<n>]` and `[Object<n>]` carry `Activity`, `Hide`, `HasCol`, `Invisible`, `IsTrigger` and
  `Waypoint`. `Env\Nat03\nat03.res:Env\nat03\Nat03.scn` `[Model224]` is `SLT=Type_National`, `Type=StartZone`,
  `Activity="Free Ride"`, `Hide=True`. These are start zones, waypoint gates, hill-climb markers and
  air-race rings. game-render's and game-sim's scene readers take every `[Model]` and `[Object]`, so a mod
  draws and collides all of them in a Free Ride. What the game does with these keys (from SLUS_211.04's
  scene loader) is still to be found. A mod cannot fix it, since the scene is never edited (§2a).
- **game-render `eco`: an `EcosystemFile` with no extension** (M2). MC02, SM01, SM02 and SM03 say
  `EcosystemFile=mc02` (…), and the archive holds `Env\mc02\MC02.esb`. `Grove::load` reads the name as
  written, so it finds nothing and those maps get no plants. Fix: if the name finds nothing and has no
  extension, try `<name>.esb` (that the game does this is inferred from the files, not looked up).
- **game-render `scene::Material::parse`: texture names** (M2). (1) `Env\Nat12\nat12_mat.res`:
  `Tex\Nat12_genericsandstone.psm` and `Nat12_pillarA.psm` say `TextureMap_1 = Nat12 _genericsandstone`
  with a space, and the archive has `Tex\Nat12_genericsandstone.pst`. The original's text reader drops
  spaces (as `scene::slt_name` already does for `SLT=`). (2) `Tex\Nat05_liverystable_detail.psm` (Nat05,
  MC05) says `Nat05_liverystable_detail.dxm`, and the archive has `Nat05_liverystable_detail.pst`. Fix:
  squeeze spaces out of the name, and drop a `.dxm` tail (what else the original strips is unknown).
- **game-render `psg::Model::parse`** (M2). `Env\Nat12\nat12.res:Eco\nat12_busha.psg` (and `bushb`,
  `bushc`, `waitaminute`, `weedsb`): `unexpected UNPACK of 12 x 2 x 8 bits at 489`.
  `Env\Nat16\nat16.res:Geo\northernlights.psg`: `the file ends at 23513 but 3933048 bytes were wanted at
  373`. The other 2,766 `.psg` in the 56 map archives read.
- **game-render `skin::Clip::parse`: animated objects' clips** (M2). 24 of the 44 `.mot` under
  `Env\<map>\anim\` in the map archives are refused, for example `Env\MC01\mc01.res:Env\mc01\anim\Nat07_WM_Blade1.mot`:
  `not a .mot: a name of 1308622848 bytes at 21`. Also refused: Nat04's Ferris wheel, Nat05's signs,
  noose, gallows door and bell, Nat10's mill wheel, tyre swing and windmill, and the score-tower people
  of MC07 and MC08. These are the animated objects whose `.col` is class 1 (MD5).
- **MD7, the `.vig` reader is in the shell crate** (T4): `Clip::vig` / `Adpcm::vig` live in
  `crates/openmx/src/audio.rs`, which brings wgpu, ggrs and iroh with it, so this repo cannot call
  them to prove a music pack (§5 step 4). `modcheck` checks only that each song's file is there.
  Fix: the `.vig` header and ADPCM decode in a crate with no shell dependencies (game-render or its own).
  Until then, A1's `tools/import-music` compiles `audio.rs` into itself by `#[path]` and runs
  `Clip::vig` / `Adpcm::vig` on all 42 MX vs. ATV songs. That build breaks if `audio.rs` ever uses
  `crate::`.
- **MD7, stereo streams play past the end flag** (A1, `notes/music.md`). `decode_stereo` and
  `Adpcm::read` decode every block pair up to the header's size, but `decode_mono` stops at the
  first block flagged as the end (flags bit 0). All 20 of MX vs. ATV Unleashed's `Aud\Music\*.vig` have
  one pair of `0x07` filler blocks after the end-flagged pair, so openmx plays 28 frames of a constant
  224 at the end of each song (`import-music` puts this in the pack's `todo`). Untamed's songs end on the
  flag. Fix: stop a stereo stream at the pair whose blocks carry the end flag, as `decode_mono` does.
  Not checked: whether MX Unleashed's own `.vig` songs have the same filler.
- **New: game-render opens only `MX.PSR`** (M9, `tools/renderprobe`). `game-render/src/disc.rs`
  `Disc::read_from` looks up `MX.PSR` by name, and reads 64 KiB of directory. MX vs. ATV's
  `MXVSATV.PSR` directory is longer than that (20,391 members). `Res` has private fields and no
  constructor except `Disc::res`. So `Assets::load`, `Sky::load`, `Grove::load` and `Splat::load` take
  `&[Res]` that can only come out of an MXU disc. A mod's loader needs the same thing for a `.res` in a
  mods folder (MD2). Fix: `Res::from_bytes(Vec<u8>)` (and `Res::parse` of the file it holds), and
  `Disc` reading the directory length out of the PSR header. `renderprobe` gets round it for now with a
  small in-memory ISO holding the `.res` unchanged.
- **New: material texture names** (M9). `scene::Material::parse` keeps `TextureMap_1` as written, and
  `world::texture_files` appends `.pst`. Three cases:
  - (1) An extension: every MX vs. ATV map's `Tex\SunFlare<N>.psm` says `sunflare<N>.pst`, so the
    lookup is `sunflare33.pst.pst` (Nat01). MXU's `free06` `sunflare03.psm` is the same. Nat05's
    `Nat05_liverystable_detail.psm` says `….dxm`, and the `.pst` is beside it.
  - (2) A space: Nat12's `Nat12_genericsandstone.psm` and `Nat12_pillarA.psm` say
    `Nat12 _genericsandstone`. The file is `Nat12_genericsandstone.pst`, and 10 placed meshes go
    untextured. `scene::slt_name` already drops spaces the way the original's text reader does.
  - (3) Only the first texture map is read. The ground's `TerrainPlain.psm` has its detail map in
    `TextureMap_2` (`NAT01_d`, see MD4).

  Fix: drop whitespace from the value as `slt_name` does, and strip a trailing extension before
  appending `.pst` / `.atm`. Whether the original strips any extension or only `.pst` was not looked up.
- **New: `EcosystemFile` without its extension** (M9). SM01, SM02, SM03 and MC02 give `EcosystemFile=sm01`
  (`mc02`); the archive holds `Env\sm01\sm01.esb`. `eco::Grove::load` does `read_suffix(file)` and
  finds nothing, so these maps have no trees. Fix: try `<file>.esb` when the name has no extension.
  What the original does with it was not looked up.
- **New: `.psg` streams game-render does not take** (M9). `psg.rs` refuses:
  - **V2-8 texture coordinates.** Nat12 `Eco\nat12_busha.psg` (and `bushb`, `bushc`, `weedsb`,
    `waitaminute`: 6 of Nat12's 10 eco species) fail with `unexpected UNPACK of 12 x 2 x 8 bits at 489`;
    so does loose `Geo\nat12_jumpgatea.psg`. Fix: accept `(2, 8)` UVs, scaled as the 16-bit ones are
    once the scale is known.
  - **Stream mask `0x1f`.** Nat16 `Geo\northernlights.psg` (placed) fails with `the file ends at 23513
    but 3933048 bytes were wanted at 373`. Its one mesh has stream mask `0x1f`, and the reader knows only
    bits 1, 2, 4 and 8.
  - **V3-16 normals.** Loose `Tex\dallas_stadium.psg`: `unexpected UNPACK of 16 x 3 x 16 bits`. It is on
    no map's archive, so this one is low priority.
- **MD4, more evidence** (M9, `notes/mxvatv.md` *Render-side formats*). Each terrain page is a `.pst`
  less its palette. Put the `.trn` header's palette (GS order, as `pst::unswizzle`) in front, and
  `pst::Texture::parse` decodes all 1,812 pages of the 56 MX vs. ATV maps, and MXU `nat01`'s 32. The
  pages are an aerial painting of the map in 256×256 tiles. The ground's material is `TerrainPlain.psm`
  (`Vu1TerrainRenderer.vuo`): map 1 the pages, map 2 `<MAP>_d`, a 512×512 detail `.pst` loose beside the
  map (`Env\Nat01\Nat01_d.PST`), alpha-blended. MX vs. ATV has no `terrainscreenspacemesh.psm` and no
  `_shadow_NN.pst`, so `splat::Splat::load` gives nothing on all 56. game-render's
  `terrain::Terrain::parse` takes page 0 from its header on as "the ground texture". Still open: which
  tile a block draws, and how.
- **New: the `.prt` reader is in the shell crate** (M9): `crates/openmx/src/particles.rs` reads the particle sheet,
  so this repo cannot prove a map's `Part\<map>particles.prt`. Fix: as for `.vig`, a crate with no shell
  dependencies.
- **New: MX vs. ATV's terrain query is not quite MX Unleashed's** (T7, `notes/accuracy.md`). Proved on
  Lago (`mxvatv_nat05`) with `tools/accuracy heights`. MX vs. ATV Unleashed's own query is in
  `GAME_REL.BIN`: `0x4838f0`, and `0x4833c0` for the wheels. It ran on a RAM image of the game in a Free
  Ride, against `contact_search::ground` and `nearest_surface` on the mod's `.trn`, at 8000 points in the
  four modes. Every height, surface and corner weight is equal. The normals are not: 7114 of 8000 points
  differ by 1–3 ulp a lane. MX vs. ATV's cell reader (`0x47b300`) hands normals back over 127: `k =
  1.0/127.0`, x and z times `vertical_scale × k`, y times `k`. openmx's `Terrain::cell` keeps the raw
  bytes. With that one step (`tools/accuracy/src/heights.rs`, `Over127`), all 8000 points are equal in
  every field. Two more differences: (1) the query clamps the point into `[min + border, max − border]`
  (object `+0x50`, 0.01 on Lago) before scaling, so off the map the original answers with the edge cell
  where openmx's misses; (2) a global (`0x541518`) switches the reader to a packed normal table (`0x39e320`).
  It is off on Lago. Fix: a per-source variant of the query for MX vs. ATV mod tracks (the normal step
  and the clamp), chosen from mod.json's `source.boot`, if wheels on those tracks are to be bit-exact.
- **MD10, what the harness passes** (T7). `tools/accuracy pair` runs `openmx --mod {mod} --shot {shot}
  --shot-pose x,y,z,yaw,pitch,fov`. The template is `--openmx-args`, and `--mod` is a guess at MD2's flag.
  Yaw is `atan2(forward.x, forward.z)` in degrees (0 = +z, 90 = +x), pitch `asin(forward.y)` (positive
  up), no roll, and fov is the original's viewport number (85 on MX vs. ATV's ride camera). Whether 85
  spans the width or the height is not established (`notes/accuracy.md`). MD10 should say which it takes,
  so the pairs frame alike. The original's shots carry its HUD; `--overlay none` or `original` on openmx's
  side decides whether the pairs compare HUDs too.
- **New: `world::Ground` sizes a terrain from the scene, the game from the file** (M12, `notes/mxvatv.md`
  *Against the game*). `Ground::new` takes `TerrainWidth × 16^levels` samples a side. MX vs. ATV
  Unleashed's terrain object (its box at `+0x30/+0x40`, read in a ride on every map) reaches as far as
  the file's leaves do, 16 samples a leaf. On the 28 one-level terrains (FS01–08, MC06–08, SX01–18) the
  port's box is 56 × 56 ft (SX01–02: 80) where the game's is 896 × 896 (1,280); on SX07 (`TerrainBreadth=2`)
  56 × 112; on Nat04 (56 × 80 leaves, the scene says 5 × 5) 5,760 × 5,760 where the game's is
  4,032 × 5,760. `contact_search::ground` does not clamp to the box, so every height on those maps is still
  the game's (`tools/accuracy heights`, 8,000 points a map). What reads `Ground::extent` (the edge, the
  radar, the spawn checks) gets the wrong size. Fix: the field's size from the file's leaves (as
  `tools/accuracy/src/look.rs` `leaf_extent` walks them), part of MD3.
- **MX vs. ATV's normal step, on every map** (M12). The T7 entry above (`0x47b300` hands the normals back
  over 127) holds on all 56 MX vs. ATV Unleashed maps: heights, surfaces and weights equal at 8,000 points
  a map in the four modes, normals 1–3 ulp off at 61–91 % of them, every field equal with the step. No
  map turns on the packed normal table (`0x541518`).
- **New: camera-following props** (M12). MC01, Nat04 and Nat16 place a `[Model<n>]` with
  `Type=CameraFollowing`, `FollowCamera=TRUE` (`SLT=CameraRain` on MC01, at the map's centre): the rain
  (snow) sheet the game draws round the camera (MC01's shots in `out/accuracy/mc01/`). `track::Scene`
  reads no `Type`, so the port draws it once, fixed where the scene puts it. Fix: a model whose `Type` is
  `CameraFollowing` follows the camera (its `MaxTiltAngle`, `MaxVelocity` and `RenderLoop<n>Camera` keys
  say how), and is not drawn in place.
- **New: physics collections** (M12). The game opens `<map folder>\<Name>.pco` for the first
  `NumberOfPhysicsCollections` `[PhysicsCollection<n>]` of the map's `.toy` (SLUS_211.04 `0x002171b0`), and
  each part's `Col\<Name>m.col` and `f.col` (GAME_REL.BIN `0x004922a0`): Lago's saloon doors, swinging on
  pins (`[Constraint<n>] Type = Pin`, ±70°). Nat01, Nat02, Nat05, Nat06 (ten) and Nat07 have them. The mods
  carry them now; no openmx reader takes a `.pco`, so the doors stand as fixed props, if at all. Fix: a
  `.pco` reader and the rigid bodies (a later MD task; the class-1 animated `.col` of MD5 are the same
  family).
- **New: a Free Ride start under the ground** (M12). Every MX vs. ATV Unleashed Free Ride starts at the
  scene's `DefaultStartPosition` (or the `.atyList`'s `StartPosition` on MC01 and FS05), seen in PCSX2 on
  all 56 maps: about a minute in, the ride camera is 14 ft behind it on 50 (and 5 ft up), and the other
  six had rolled downhill from it (FS08, MC05, Nat05, Nat14, Nat17, SX06: 39–263 ft). On FS02, FS06,
  Nat10, Nat14 and SM01–03 that position is under the ground (6–312 ft; `y` 90 or 135 on all but Nat14),
  and the game stands the rider on the ground over it (the camera 5 ft over the ground, 20–310 ft over
  the start; Nat14 had rolled away). Fix (MD2): a mod track's spawn takes the ground's height where the
  scene's is below it. Each such mod's `todo` says so.
