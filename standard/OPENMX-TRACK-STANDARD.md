# OpenMX track mod standard (v1, draft)

What a mod for the MX Unleashed port (`openmx`) holds: a **track** (one map from another game or from
elevation data, §2), a **world** (several tracks joined into one ground, §3), a **music** pack (§4) or a
**trick** pack (§6). The port's loader is openmx Phase 14: MD2 reads tracks, MD7 music, MD8 and MD9
worlds; nothing reads trick packs yet (`notes/openmx-asks.md`, M19).

The rule all three keep: **a mod is its source as the source's disc has it.** The files are the source's
own bytes, copied and never resampled, rescaled, re-encoded or "fixed". Anything the port has to read
differently goes in `mod.json`, and the files stay as they are. What the port cannot show yet goes in
`todo` (§1) and in this repo's `notes/openmx-asks.md`. A mod can only add a track, songs or tricks. With no
`mods/` folder, or with Settings → Mods: Off, the port reads nothing from it, and every pick, ride and
replay hash is the same as without mods.

Mods are game data. They are built into `out/mods/` (T4) and are never committed.

Status: the fields below are fixed for v1. The loader comes in steps: MD2 loads a track (its terrain from
MX vs. ATV needs MD1, a big one MD3); MD4 draws the ground from the terrain's own texture pages; MD5 reads
class-1 collision; MD6 handles online; MD7 plays music packs; MD8 loads worlds; MD9 adds blending and the
world sky; MD11 adds the game filter. Until a step lands, the port ignores the files and fields it covers.
It does not reject them.

## 1. Every mod

```
mods/
  tracks/<id>/    a track (§2) or a world (§3)
  music/<id>/     a music pack (§4)
  tricks/<id>/    a trick pack (§6)
```

The `mods/` folder sits beside `settings.txt`. On wasm the browser's storage holds it under the same paths.
Each mod's folder has a `mod.json` (UTF-8 JSON):

```json
{
  "standard": 1,
  "kind": "track",
  "id": "mxvatv_mc01",
  "name": "Machine Challenge 1",
  "source": {
    "game": "MX vs. ATV Unleashed",
    "boot": "SLUS_211.04",
    "members": ["Env\\MC01\\mc01.res", "Env\\MC01\\mc01_mat.res"]
  },
  "version": "1.0.0",
  "author": "openmx-mods tools/import-mxvatv 0.1",
  "permission": "own-disc",
  "todo": ["Col\\MC01_WM_Blade.col: collision class 1, refused (openmx MD5)"]
}
```

| field | | what |
|---|---|---|
| `standard` | required | `1`. If the number is higher than the loader knows, the mod is skipped with one log line. |
| `kind` | required | `track`, `world`, `music` or `tricks` (§6). Any other value is skipped quietly. It is not an error. |
| `id` | required | Equal to the folder name. `[a-z0-9_]`, 1–40 characters, `<prefix>_<map>` (table below), never changed once published. The prefix keeps a mod from ever taking a disc track's name (`free06`, `nat01`, `sx01`). |
| `name` | required | What the track pick or the song list shows. Use the source game's own name for the map where it has one (its UI strings, M8). |
| `source` | required | Where it came from. `game`: exactly one of the names below, which the track pick filters by (MD11). From a disc: `boot` (the boot ELF in SYSTEM.CNF, AGENTS.md's table) and `members` (the PSR members the files were copied from). From a data set: `data` (product and tile, e.g. `"USGS 3DEP 1 m, n33w115"`) and `date` (when it was fetched). |
| `version` | required | The mod's own version, `major.minor.patch`. Bump it whenever any file changes. Online, a peer needs the same id and hash (MD6), so a stale version is refused there anyway. |
| `author` | required | Who or what built it: the tool and its version, or a person. |
| `permission` | required | `own-disc`: copied from the user's own disc, never shared. `public-domain`: may ship whole (USGS, SRTM). Otherwise a sentence naming who gave permission and where. |
| `todo` | required | Every known gap between the mod and its source, one string each, naming the file and the reason. Use `[]` only when T7 has found none. |
| `checked` | optional | The T7 accuracy result, e.g. `"T7 2026-11-02: heights exact at 1000/1000, 5 poses"`. Leave it out until the check has run. |

| `source.game` | `id` prefix |
|---|---|
| `MX Unleashed` | `mxu_` (only as a world's tile, §3: the disc already has these tracks) |
| `MX vs. ATV Unleashed` | `mxvatv_` |
| `MX vs. ATV Untamed` | `untamed_` |
| `ATV Offroad Fury 2` (1, 3, 4 alike) | `atvof2_` … |
| `MX vs. ATV Reflex`, `MX vs. ATV Alive` | `reflex_`, `alive_` |
| `Real elevation` | `elev_` |
| `Combined` (a world, §3) | `world_` |

A later source adds its row here in the same commit as its importer.

The loader ignores fields it does not know. An importer may add its own fields (`map`, `notes`, …). It may
not give a v1 field a different meaning.

A **broken** mod is one whose `mod.json` does not parse, whose required field is missing or wrong, or
whose required file (§2) is missing or refused by openmx's reader. A broken mod is one log line naming the
mod and the reason, and the pick leaves it out. The port never crashes on one (MD2).

The **identity** used online (MD6) is the `id` plus a hash, taken over every file in the folder (paths
lowercased and sorted, with `mod.json` included and `previews/` left out). MD6 chooses the hash function.

## 2. Track mod

```
mods/tracks/<id>/
  mod.json
  <the source .res's members, loose, at their own paths>
  previews/track.png    optional: the track pick's picture (MD11/P70 decide its size)
```

For a map taken off a disc, the folder is the map's track archive (and its material archive, if it has
one) unpacked. Each `.res` member becomes the file at its member path, with `\` turned into `/` and the
spelling kept. From MX vs. ATV Unleashed's `Env\MC01\mc01.res` and `mc01_mat.res` (the kinds of file
`notes/mxvatv.md` lists; the exact member spelling is M2's to find):

```
mods/tracks/mxvatv_mc01/
  mod.json
  Env/MC01/MC01.scn           the scene (§2a)
  Env/MC01/MC01.ps2.trn       the terrain the scene's TerrainFile names
  Env/MC01/MC01.toy  .esb …   toys, ecosystem, the map's own files
  Col/*.col                   the collision shapes the scene places
  Geo/*.psg                   models (SLT=)
  *.pst  *.psm  *.atm         textures and materials, wherever the archive keeps them
  *.ps2.cub                   the sky the scene's CubeFile names
```

The rules:

- **Everything the source's archives hold is copied, including what the port does not read yet.** A file
  the port ignores costs nothing, and a later loader step will find it there. The copy has the member's
  bytes exactly (the import's check, §5).
- When two archives hold the same path: identical bytes make one file. Different bytes break the import.
  It stops and names the path. It never chooses one.
- **Loose members the game opens for the map are copied too.** When the source game, finding a name in
  none of its loaded archives, opens the container's loose member at that path (MX vs. ATV Unleashed
  and Untamed do, `notes/mxvatv.md` *Loose PSR members*, `notes/mxvatv-later.md` *The importer (M4)*), that member is copied at its own path. It is listed in
  `source.members`, and the importer may give its reason in a field of its own. A path an archive holds
  is always the archive's.
- **Lookups ignore case and slash style.** The port finds a file the way game-render's `disc::Res` does,
  by the end of its path compared in lowercase (`read_suffix`), so the scene's `SLT=bale_single_02` finds
  `Geo/bale_single_02.psg` wherever it sits. Two files whose paths differ only in case break the mod.
- **The scene is the folder's only `.scn`.** Its file stem is the track's **stem** (`MC01`, `free06`). A mod
  with no `.scn`, or more than one, is broken. The port builds several names from the stem as it does on the
  disc: `<stem>.toy` (toys), `<stem>particles.prt`/`.pst` (particles), `<stem>_shadow_01..16.pst` (the
  ground's light map), `<stem>.aud` (track sounds; today the port reads only the exact disc path
  `tracks\<stem>\<stem>.aud`, so a mod needs MD2 to find it by suffix, `notes/openmx-asks.md`). The stem is the source's own map name, so these
  files keep their source names. A generated map (`elev_*`) uses its `id` as the stem: `<id>.scn`,
  `<id>.ps2.trn`.
- **The terrain** is the file the scene's `TerrainFile` names. It is looked up as the shell looks up a disc
  track's: first the name as written, then with the platform tag inserted (`MC01.trn` → `MC01.ps2.trn`).
  It is a PLN v7 `.trn` that openmx's `track::Terrain` reads. MX vs. ATV's terrains have texture pages
  between the blocks and the tree, and MXU's nationals and supercross tracks do too
  (`notes/mxu-terrain-limits.md`). `Terrain::parse` needs MD1 to read those. A generated terrain (T2)
  follows `notes/mxu-terrain-limits.md` § *What a writer has to respect*.
- **Collision**: one `.col` per distinct `COL=` in the scene's `[Object<n>]` sections. A shape that is
  missing, or that `track::Collision::parse` refuses, does not break the mod. The port skips it as it
  skips a disc shape (one log line), and it is listed in `todo`.
- **Not in a mod**: the bike, the rider, their physics and the follow camera. MXU keeps these in a
  freeride track's second archive (`free06_2.res`). A mod has no second archive. The port rides a mod
  with the disc's own freeride bike and files (MD2 decides which disc archive supplies them).

### 2a. The scene

The scene is the source's `.scn`, byte for byte: CRLF INI text, read by two readers in openmx. game-sim's
`track::Scene` reads the spawn, the terrain layout and the placements in the game's own floats.
game-render's `scene::Scene` reads the props, lights, fog and ecosystem for drawing. **A track mod's
scene is never edited.** If the port has to read something differently, `mod.json` says so (the spawn,
§2b). A generated map's scene is written by T2's writer and has every key the table marks *must*.

The key and section names below are the ones `free06`, `nat01` and `sx01` on MXU's disc use (all three use
the same set). Section names ending in digits are numbered: `[Model1]` … `[Model577]`.

| section | key | who reads it | in a mod |
|---|---|---|---|
| `SceneInfo` | `DefaultStartPosition`, `DefaultStartDirection` | both readers: the spawn (§2b) | **must** |
| | `SceneName` | both readers: the name in the log | **must** |
| `Environment` | `TerrainFile` | both readers: the terrain (§2). `track::Scene` refuses a scene without it | **must** |
| | `TerrainWidth`, `TerrainBreadth` | game-sim: 256-sample units a side. The map edge is width × 256 × scale (`0x001a6e50`). This is the tree's root cells only on a two-level tree: a one-level tree (MX vs. ATV's supercross and freestyle maps) has 16 root cells of 16 samples and `TerrainWidth=1` | **must**, ≤ 16, equal to the terrain's samples a side / 256 (below, for a disc scene that disagrees) |
| | `TerrainWidthScale`, `TerrainBreadthScale` | game-sim: feet a sample, for the edge and the radar (`radar_init`) | **must**, equal to the `.trn`'s own scale (below, for a disc scene that disagrees) |
| | `TerrainBorderWidth`, `MinDivisionSize` | game-sim `TerrainLayout` | **must** (free06, nat01 and sx01 all have both) |
| | `CubeFile` | game-render `sky`: `<name>.ps2.cub`, then the name as written | as the source has it |
| | `EcosystemFile`, `EcoDistance2D`, `EcoDistance3D`, `ShrubberyDistLimit` | game-render `eco`: the ecosystem and its distances | as the source has it |
| | `DetailMapTilingMultiplier` | the game's `.trn` loader (T3). The port does not read it yet: game-render `terrain` assumes 8, MXU's value; MD4 | as the source has it |
| | `Wind` | openmx `particles` | as the source has it |
| | `AmountOfTransmittedLight`, `ParticleTextureMapName` | not read | kept, ignored |
| | `Xmin`, `Xmax`, `Zmin`, `Zmax` | the game's level load (T3), not the port | kept, ignored |
| `Fog` | `ColorRGB`, `MinimumFogDistance`, `MaximumFogDistance`, `Visibility`, `Haziness` | game-render: fog. A missing key takes the defaults in `scene.rs` | as the source has it |
| `Lights` | `NumberOfLights` | not read: the port counts the sections | kept, ignored |
| `Light<n>` | `Type`, `ColorRGB`, `Position`, `TargetPosition` | game-render: the lights. A section without `Type` is not a light | as the source has it |
| | `CastsShadows`, `EmitsLight`, `ShadowIntensity`, `ShadowRGB` | not read | kept, ignored |
| `StaticModels` | `NumberOfStaticModels` | not read | kept, ignored |
| `Model<n>` | `SLT`, `Position`, `LookVector`, `UpVector` | both readers: a drawn prop (`<SLT>.psg`). No collision | as the source has it |
| | `Type` | game-sim: `Placement::kind` | as the source has it |
| | `AllowShadowReceiving`, `UseDynamicLighting` | not read | kept, ignored |
| `CollisionObjects` | `NumberOfCollisionObjects` | not read | kept, ignored |
| `Object<n>` | `COL` (or `SLT`), `Position`, `LookVector`, `UpVector`, `Type` | game-sim: a collision shape placed | as the source has it |
| anything else | | ignored by both readers, as the game's INI reader ignores keys no class claims | kept, ignored |

A disc map whose own scene disagrees with its terrain is copied as it is, since the scene is never
edited. On MX vs. ATV Unleashed four do: Nat04 has `TerrainWidth=5` over 4 root cells (a two-level
tree, so 1024 samples, not 1280), SX07 has `TerrainBreadth=2` over a 256-sample terrain, and MC08 and
SX18 have `TerrainWidthScale=3` and `TerrainBreadthScale=3` over a 3.5-ft terrain. Such a mod names each
disagreement in `todo`, one entry per key that names both the scene file and the key, for example
`"Nat04.scn: TerrainWidth=5 on the disc, the terrain is 1024 samples (4 root cells) wide"`. The check
(§5 step 2) fails the mod without that entry, and with it reports the disagreement as a note. What the
game does with such a scene (its edge and radar from the scene's numbers, the ground from the tree) is
for T7 to see. A generated map (T2) has no such excuse: its scene must agree with its terrain.

A scene that a source game writes differently (MX vs. ATV Untamed's binary `.scn.bpb`) is decoded to this
INI text by the importer (M10's `tools/bpb`), not by the port. The decoded `.scn` sits beside the `.bpb`,
which is still copied. Its `todo` says so, and the decode is proved by `Scene::parse` and the hash check
in M10. The other BPB kinds the port reads as text are decoded beside themselves the same way (T11): the
materials (`.psm.bpb`), the toys (`.toy.bpb`) and the particle sheets (`.prt.bpb`). One `todo` entry a
kind names every decoded file.

The scene keeps its own lengths and units: feet, positions in the terrain's world space with the origin
at the tree's corner. A mod that is not in feet, or that is offset, is a resampled mod, and this
standard does not allow it. `track::Scene::parse` wants the text to be UTF-8. A source scene that is
not is a port ask, not a reason to edit the scene.

### 2b. The spawn

A track mod's spawn is its scene's `[SceneInfo]` `DefaultStartPosition` (x, y, z in feet) and
`DefaultStartDirection` (a direction). free06, nat01 and sx01 have both, and the port places a freeride
rider there.

The source game may start a freeride somewhere else, for example from an activity file (MX vs. ATV's
`.atyList`, M8). Then `mod.json` names that start, and the scene is not edited:

```json
"spawn": {"position": [2884.0, 338.0, 2750.0], "direction": [0.0, 0.0, 1.0],
          "from": "Env\\MC01\\MC01.atyList [Freeride] StartPosition"}
```

`position` and `direction` are read as single floats (the scene's own conversion, `track::Scene`'s
`single`). `from` is required: it names the file and key the start was read from, so T7 can check it
against the running game. A `spawn` with no source is a guess, and a guess is not allowed. v1 has only
one spawn. A list of named starts would come in a later version.

## 3. World mod (the combined maps, C1)

A world is several tracks laid side by side as one ground. Each track is a **tile**, kept whole as its
own track mod. **Join** ground fills the band between neighbours, where each map's edge berm was cut.
The port loads it as one ride (MD8). Under each tile the port queries that tile's own terrain with the
original's query, so the ground there is bit-exact to the tile ridden alone.

```
mods/tracks/world_mxvatv/
  mod.json                 kind "world"
  tiles/<tile>/            one complete track mod folder (§2) per tile, with its own mod.json
  joins/<join>/            one generated terrain per join piece: <join>.ps2.trn (T2's writer)
                           and <join>.blend (below)
```

```json
{
  "standard": 1, "kind": "world", "id": "world_mxvatv", "name": "MX vs. ATV Unleashed: everything",
  "source": {"game": "Combined", "games": ["MX vs. ATV Unleashed"]},
  "version": "1.0.0", "author": "openmx-mods tools/stitch 0.1", "permission": "own-disc",
  "tiles": [
    {"tile": "mc01", "at": [0.0, 0.0], "keep": [64, 64, 1216, 1216]},
    {"tile": "nat03", "at": [5760.0, 0.0], "keep": [64, 48, 1216, 1232]}
  ],
  "joins": [
    {"join": "mc01_nat03", "at": [5472.0, 0.0], "between": ["mc01", "nat03"]}
  ],
  "drop": [{"tile": "mc01", "section": "Model214", "why": "in the cut berm band"}],
  "sky": "mc01",
  "spawn": {"tile": "mc01"},
  "todo": []
}
```

- `tiles[].tile` is the folder under `tiles/`, and its `mod.json` is the track mod it was copied from,
  unchanged. `at` is where the tile's origin (sample 0, 0) sits in the world, `[x, z]` in feet. There is
  no vertical offset: heights stay the tile's own, and the joins take up the difference. Choose the
  layout so neighbours' base heights are close. `at` must be a whole multiple of the tile's leaf size
  (16 × its spacing), so the tile starts on a whole leaf and world − `at` loses nothing to the offset
  itself. MD8 checks what single floats do far from the origin.
- `keep` is `[x0, z0, x1, z1]` in the tile's own samples, half-open: the interior left after the edge berm
  is cut (C1 finds the band from the terrain and the scene's border and shootoff values). Inside `keep`
  the tile owns the ground. Outside it, the tile is not used.
- `joins[].join` is the folder under `joins/`. Its `<join>.ps2.trn` is a PLN v7 terrain (T2's writer) at
  `at`, and its scale is one of the world's tile spacings. C1 puts every join on one lattice for the whole
  world, at the commonest tile spacing (the finer on a tie), so joins agree where they meet and each
  interior's edges lie on lattice lines; `joins[].samples` (C1's own field) gives a join's samples a side.
  Round a tile whose spacing is not the lattice's, the joins next to it are at that tile's spacing
  (C6), listed first, and meet the lattice's joins along lines where both have samples.
  The ground is C1's solved blend, with no
  height or slope step at any seam and nothing steeper than the bike climbs. Each sample's surface byte is
  the nearer tile's. The join has no `.col` and no scene. The heightfield is its collision, as on the disc.
- `<join>.blend` is the ground's look across the join (MD9): one byte per terrain sample of the join for
  each tile in `between`, in that order, rows along x and then z (`samples_x × samples_z × between.len()`
  bytes). The bytes of one sample add up to 255. Each is that tile's share of the drawn ground texture and
  detail at that sample. It fades from 255 on one side to 0 on the other, with no hard edge. The port draws
  each tile's own ground (MD4) under the weights. A corner where more than two tiles meet just lists more
  `between`.
- **Precedence**: a world point inside a tile's `keep` is that tile's. Otherwise it belongs to the first
  join in `joins` whose terrain covers it. Otherwise it is off the world, and the shootoff is only there,
  at the world's outer edge.
- `drop`: placements left out because they stand in a cut band, one `{tile, section, why}` each, where
  `section` is the scene section (`Model214`, `Object88`). The tile's scene is not edited. The port skips
  these sections. Every dropped prop is listed, and nothing else is dropped.
- `sky`: the tile whose `CubeFile`, `[Fog]` and `[Light<n>]` light the whole world when Settings →
  World sky is Global (MD9's default). It defaults to the first tile. Per map uses each tile's own.
- `spawn`: `{"tile": "<tile>"}` uses that tile's spawn (§2b) moved by its `at`, or
  `{"position": [x, y, z], "direction": [x, y, z], "from": "…"}` in world feet. It defaults to the first
  tile's.
- A world may hold another world's tiles (the ultra map, C5). It copies the tiles and joins it keeps. It
  does not point at another mod.
- `source.games` lists every game a tile came from. The pick shows worlds under their own heading (MD11).

## 4. Music pack (A1)

```
mods/music/mxvatv_music/
  mod.json         kind "music": the pack and its playlist
  <the streams, as the disc's members, at their own paths>   e.g. Aud/Music/<song>.vig
```

```json
{
  "standard": 1, "kind": "music", "id": "mxvatv_music", "name": "MX vs. ATV Unleashed soundtrack",
  "source": {"game": "MX vs. ATV Unleashed", "boot": "SLUS_211.04", "members": ["Aud\\Music\\*.vig", "Aud\\Music\\music.pb"]},
  "version": "1.0.0", "author": "openmx-mods tools/import-music 0.1", "permission": "own-disc",
  "songs": [
    {"file": "Aud/Music/song01.vig", "title": "…", "artist": "…", "from": "Aud\\Music\\music.pb [Song 1]"}
  ],
  "todo": []
}
```

- The streams are the disc's `.vig` members, byte for byte, with no re-encode. openmx's `Clip::vig` /
  `Adpcm::vig` reads them (`crates/openmx/src/audio.rs`: the 0x800-byte header, stereo ADPCM, its rate).
  A stream the reader refuses goes in `todo` and `notes/openmx-asks.md`. It is still copied.
- `songs` is the playlist, in the order the source game's own list has it. Each song has `file` (a path
  in the pack), `title` and `artist` as the source game's jukebox shows them (A1 checks this), and `from`,
  the playlist member and entry they were read from. A song may have its own `game` if a pack mixes
  games. Otherwise it takes the pack's `source.game`, which the port shows beside the title and which its
  "which games' music plays" setting filters by (MD7).
- The playlist file the disc has (`music.pb`, Untamed's `.bpb`/`.txt`/`.ini`) is copied too, for proof.
  The port reads only `songs`.
- The port gives each song the id `mod:<pack id>/<file stem>`, next to the disc's `disc:<stem>`.

## 5. How a mod is proved

An importer or T4's `build.sh --check` proves each mod it writes, with openmx's readers and nothing
else:

1. Every copied file equals its source member, byte for byte (read with `game_sim::archive::{Psr, Res}`).
2. `track::Scene::parse` reads the scene. `track::Terrain::parse` reads the terrain it names (with MD1's
   relaxation until that lands, saying so). `track::Collision::parse` reads every `.col` the scene places,
   or the refusal is in `todo`. The scene's `TerrainWidth × 256` equals the terrain's samples a side and its
   `TerrainWidthScale` the terrain's scale (and the same for breadth), or a `todo` entry names the scene
   and the key (§2a).
3. A world (§3): every tile's folder passes 1–2. Every seam point is continuous in height and slope. Each
   `.blend` sums to 255 per sample. Every `drop` names a section that exists. Under `keep` the tile's
   ground is identical to the tile alone (C1's `--check`).
4. A music pack: every `songs[].file` exists and `Adpcm::vig` reads it.
5. T7 then compares the mod with its source game running in PCSX2. Until that has passed, the map is not
   done, and `checked` is absent.

## 6. Trick pack (M19)

A later game's tricks as its disc has them: the stunt table, the stunt animations, and the bikes'
`.phy` whose `[Backflip]` and `[Three Sixty]` tune the rotations. What each trick is and how the game
drives it is `notes/tricks.md`; openmx's side is in `notes/openmx-asks.md` (M19). Until openmx reads a
pack, it ignores it.

```
mods/tricks/mxvatv_tricks/
  mod.json                          kind "tricks"
  Rider/Anim/MX_Rider_Stunts.ini    the tables, as the PSR has them (MXF_, ATV_ beside)
  Rider/Anim/MX_RiderStunts.mcf     each rider set's stunt animation set, and its .MLB
  Rider/Anim/MX_ThreeSixtyL.mot     every clip the .mcf names
  Veh/Anim/Bike_ThreeSixtyL.mot     the bike's clip of each stunt, found by name
  Veh/mx_Pro250A.phy                the rotations' tuning, every MX and ATV .phy
```

```json
{
  "standard": 1, "kind": "tricks", "id": "mxvatv_tricks", "name": "MX vs. ATV Unleashed tricks",
  "source": {"game": "MX vs. ATV Unleashed", "boot": "SLUS_211.04", "psr": "MXVSATV.PSR", "members": ["Rider\\Anim\\MX_Rider_Stunts.ini", "…"]},
  "version": "1.0.0", "author": "openmx-mods tools/import-tricks 0.1", "permission": "own-disc",
  "tables": [{"file": "Rider/Anim/MX_Rider_Stunts.ini", "from": "StuntInfo.res", "stunts": 46, "key": "DefaultKeyMapping"}],
  "riders": [{"rider": "MX", "table": "Rider/Anim/MX_Rider_Stunts.ini", "mcf": "Rider/Anim/MX_RiderStunts.mcf", "mlb": "Rider/Anim/MX_RiderStunts.MLB", "bike_clips": "Veh/Anim/bike_<name>.mot"}],
  "new": [{"name": "threesixtyL", "display_name": "Three Sixty Left", "table": "…", "section": "Stunt45", "hold_time": 0.72, "safe_time": 1.26, "duration": 1.5, "point_value": 2000, "hold_point_value": 15000, "gesture": null, "moves": "rotation: …"}],
  "rotations": [{"name": "threesixty", "stunts": ["threesixtyL", "threesixtyR"], "phy": "[Three Sixty]", "buttons": "R1", "stick": "…"}],
  "phy": ["Veh/mx_Pro250A.phy", "…"],
  "todo": ["Rider/Anim/MX_Rider_Stunts.ini: openmx's trick::Stunts::parse reads MX Unleashed's 57 sections and refuses this table of 46 …"]
}
```

- Every file is the disc member's bytes. The tables are the members the game's loader opens out of its
  `StuntInfo.res` (Untamed: `res\StuntInfo.res`, BPB), which are also loose in the PSR byte for byte;
  the pack copies the loose ones, at their own paths. Untamed's pack has one table, `.ini.bpb`, which
  every rider set reads.
- `tables`: each table, the archive the loader reads it from, its stunt count (the loader's) and the key
  its default gestures are in. `riders`: each rider set's table, `.mcf` and `.MLB` (`null` when the ELF
  names one the disc lacks; then `todo` says so) and the bike clips' name pattern.
- `new`: the stunts MX Unleashed's 57 have no row of, by name, each with its row, clocks (as the loader
  reads them: truncated to a float), points, default gesture (`null`: none) and how it moves. `rotations`:
  the rotating moves, the `.phy` section that tunes each and how they are triggered.
- `todo` names every table openmx's `trick::Stunts::parse` refuses, every clip openmx's
  `skin::Clip::parse` refuses (MX vs. ATV's `.mot` are versions 6 and 7), every new stunt openmx has no
  move for, and the PCSX2 check (T7).
- Proof: `build.sh --check` (modcheck) compares every file with its member and finds every file
  `mod.json` names; `import-tricks check DIR` reads every table back with the loader's rules and with
  openmx's `Stunts::parse`, and every `.mot` with `skin::Clip::parse`.
