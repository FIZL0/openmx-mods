# MX vs. ATV Untamed (PS2, SLUS-21701) — and later: Reflex, Alive

## Untamed (PS2)

First survey 2026-10-21, USA disc. Same container family as MX Unleashed and MX vs. ATV Unleashed:

```
MX08PS2.PSR   1.44 GB   PSR version 2, 20,552 members; openmx's Psr::parse reads it unchanged
MX08AUD.PSR   223 MB    audio
BLANKS.PAD    2.7 GB    padding, ignore
SLUS_217.01, GAME_REL.BIN, UI_REL.BIN
```

### Terrain: the same PLN v7

49 loose `Env\<MAP>\<MAP>.ps2.trn`, all PLN v7 `ps2`, all readable by `Terrain::parse` with the same one
relaxation as MX vs. ATV Unleashed (16 embedded textures between the blocks and the tree; `notes/mxvatv.md`).

| maps | levels | leaves | spacing (ft) | blocks |
|---|---|---|---|---|
| Nat_AZ/CO/DVCA/IN/MA/MD/NC/NY/OR/PA/SDCA/UT, SM_*, SJ_A/B, TR_A, AM_*, BT_A–F, HC_A | 2 | 80×80 (BT_E, SM_HS: 4480) | 3.5 or 4.5 | 282–1050 |
| FS_A–D, MT_A, SX_A–L | 1 | 16×16 | 3.5 | 257 |

### Scenes: binary (BPB), the one real decomp here

There is no text `.scn`. Each map has `<MAP>.scn.bpb` (and `.toy.bpb`, `.rpy.bpb`); 7,697 `.bpb` members
on the disc. The file starts `BPB\0`, then a small header, then fixed 12-byte records and a string table
that holds exactly the INI's section and key names (`SceneInfo`, `DefaultStartPosition`, `Environment`,
`TerrainFile`, `TerrainWidth`, …). The records carry a 32-bit value that looks like a hash of the name,
a string-table offset, and two 16-bit fields (parent/index and count), so it is the same INI as a tree.

Decoding it (M10) means: the header, the record, the value encoding (strings, numbers, vectors), and the
hash, from the boot ELF's BPB reader. The proof is a round trip: decode → INI → `Scene::parse`, and
for the keys, every hash recomputed from its name.

### Collision

1,604 `.col` members. Not parsed yet.

## Reflex, Alive (PS3 / 360)

No discs yet. Reflex has deformable terrain, so likely a heightfield again.

## Not verified

- No `.bpb` decoded; the record layout above is read off one hexdump.
- No Untamed `.col`, `.psg` or texture parsed.
- Heights sampled only; the 3.5 vs 4.5 spacing comes from the terrain header, not from a scene.
