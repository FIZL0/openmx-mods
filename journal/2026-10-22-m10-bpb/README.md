# M10: tools/bpb, Untamed's binary INI

`tools/bpb` decodes BPB to INI text: `bpb FILE.bpb [OUT]`, or `bpb -r DIR OUTDIR` for a tree. The library
has `parse`, `Bpb::to_ini`, `hash`, `Bpb::bad_hashes` and an encoder, `write`, that the tests use. The
format write-up is in `notes/mxvatv-later.md`, under *Scenes: binary (BPB)*.

## How it went

1. **Discs.** Extracted all 7,697 loose `.bpb` with `discx x untamed '**.bpb'`, and `SLUS_217.01` with
   `bsdtar`, into this worktree's `extracted/untamed/` (git-ignored). The images were read from the
   main checkout's `iso/`, read-only, through `DISCX_ISO`.
2. **Layout from the bytes first** (`Nat_AZ.scn.bpb`). The header offsets close exactly:
   0x20 + 16 × sections = key offset, and so on. That made the records 16 bytes, not the 12 the brief
   guessed. A Python survey in the scratchpad (not committed) checked the layout on all 7,697 files and
   found that the "type" u32 is really a u16 type plus inline bytes (type 2, short strings).
3. **Hash.** I tried common hashes on four names. djb2 matched, but I first wrote it down as
   case-sensitive. My test loop printed only the first variant that passed, and that was the lowercased
   one. Checking every name on the disc caught it: 129k of 130k section hashes failed case-sensitive,
   and none fail lowercased.
4. **Ghidra.** I set up the project for SLUS_217.01 with ../ps2-decomp-kit, without touching the kit's
   checkout. The kit's `config.py` takes `KIT_CONFIG`, so `extracted/untamed/ghidra/kit.toml` holds
   absolute `[ghidra] project` and `[paths] context` paths under the same folder. I copied the ELF to
   `context/elf/boot.elf`, then ran `KIT_CONFIG=… ./kit ghidra analyse` (1m29s) and `./kit ghidra dump`.
   To use it: `cd ../ps2-decomp-kit && KIT_CONFIG=<worktree>/extracted/untamed/ghidra/kit.toml ./kit fn 0x1425c0`.
   The project lives only in this worktree's `extracted/`. A later task needs to redo these steps (about
   2 minutes) or copy it into the main checkout's `extracted/untamed/ghidra`. I did not copy it, because
   this session must not touch the main checkout.
   - I found the reader through `%s.bpb` (0x3b62ef) and the 0x1505 seed. Loader 0x1425c0, header 0x142a50,
     section title `%s%d` 0x142c30, section lookup 0x142ca0 (the trailing digits split off by 0x142f90),
     key lookup 0x140df0 and 0x141060, getters 0x141310 (text: `%.06f` and `%d`), 0x141420 (int) and
     0x141690 (float), hash 0x138510 with tolower 0x347778. The text-INI fallback builds the same table
     (0x143680, 0x143160), with every value as type 1.
   - The ELF confirmed what the bytes showed and added three things: the `ZLIBME` zlib wrapper (none on
     the disc, so it is refused with a message), the section-title format, and how each getter converts
     between types.
5. **The INI's floats.** These were the one real choice. The game reads a type-8 value as its exact
   bits. openmx's `Scene::parse` reads text floats like MXU does (`strtod`, then truncate to single), so
   `%.06f` would lose bits for anything like 0.4 or 0.1, whose single is just above the decimal.
   Decision: write `%.06f` (what the game's get-as-text prints, and the old text files' style) when it
   reads back to the same bits under both the truncating and the nearest parse. Otherwise write the
   double's shortest decimal, which is exact under either. That is 10,392 of 47,530 floats.
6. **Values ending in a CR.** The first disc run failed on three UI files with a line break in a value:
   `Twin Air\r`, `201,49\r`, `4,-4\r`. These are stray CRs from the source text files and they are
   authentic data. They are written as they are held, and the test counts them; it fails only on a LF
   or on a CR anywhere else. My first archive glob (`Env\*\*.res`) matched nothing, because the map
   archives are `res\env\…`. Fixed.

## How it was proved

- `tools/bpb/tests/built.rs` (in the gate): a scene BPB built byte by byte in the test, with all four
  value types, an indexed and an unindexed section, and a key with no `COL`. It decodes to exactly the
  INI expected, `write` gives the same bytes back, and openmx's `Scene::parse` reads the result (name,
  terrain, width, one collision placement). Also tested: hash values off the disc; a flipped hash is
  reported; truncated, ZLIBME, wrong-magic, bad-type and keys-past-the-end files are refused; floats
  read back to their bits under both parses; an empty indexed section (`[Empty0]`), a 300-byte string,
  and the encoder refusing a 6-byte short string.
- `tools/bpb/tests/disc.rs` (`DISCX_ISO=… cargo test -p bpb --release -- --ignored`): the disc read
  through discx. Result:
  `7697 .bpb, 50 scenes (77542 placements), values by type {0: 3, 1: 331280, 2: 19583, 4: 131035, 8: 47530}`
  and `48 scenes in the map archives, 48 the same bytes as the loose one`. Every file parses, every
  section and key hash recomputes from its name, and every file re-encodes **byte for byte**, which
  proves the table order, the dedupe and the padding too. Every scene is read by `Scene::parse`, with
  its terrain file, `TerrainWidth`, `TerrainWidthScale` and `MinDivisionSize` (bit for bit), and its
  model and collision counts equal to the BPB's. The value counts match the Python survey's.
- Mutation checks: hashing without lowering fails `the_hash_is_djb2_of_the_lowered_name`, and dropping
  the truncation half of the float check fails `floats_read_back_to_their_bits`. Both were restored.
- `bpb -r extracted/untamed/psr <scratch>` wrote 7,697 INIs in 0.16 s with exit status 0 (no hash
  mismatch).

## Decisions

- **The encoder is in the library**, not only in the tests. It is the strongest proof (byte-exact on the
  whole disc), and M4 or a later mod may want to write a BPB. It writes the hashes as they are held, so
  a test can build a wrong one.
- **Values are bytes, not `String`.** Nothing on the disc is non-UTF-8 as far as `Scene::parse` saw (it
  needs UTF-8 and read all 50), but the other 7,647 files were not checked for it, so bytes are kept.
- The section index is printed as a signed 16-bit number, as the game's `%d` of a short does. The
  disc's largest index is 5713, so this never shows.
- Follow-up filed: **M14**, the survey decodes Untamed's scenes with `tools/bpb` and fills its scene
  columns. I did not do it here (M10 names only `tools/bpb` and the note), and survey's table text
  still says "not read (M10)". The note says why.

## Not verified

- **The game was not run.** All of this comes from the ELF (static reading in Ghidra) and the disc. No
  PCSX2 check that the game reads a key as the INI says, such as a float through its getter. That is
  T7's harness, which does not exist yet.
- **Getter conversions** (int from a float truncates; `T`/`ON`/`Y`; `strtol` with base 0; `atof`) come
  from the decompiled C. 0x347498 and 0x33e5c0 were matched to `strtol` and `atof` by shape (newlib
  wrappers), not stepped through. The decoder does not depend on them: the INI holds the stored value.
- **ZLIBME** is described from the loader only: a 6-byte tag, a length at 12, then 0x1a9e30 on the
  bytes from 16. I took 0x1a9e30 to be zlib from the tag's name and did not read it. No ZLIBME file is
  on the disc, so `tools/bpb` refuses them.
- **Untamed's other INI kinds** (`.psm`, `.lyt`, `.phy`, …) decode, hash and re-encode, but nothing in
  openmx reads their text, so their INI is checked only by format, not by a consumer.
- **`.toy.bpb` and `.rpy.bpb` have no openmx reader** and were checked like the other non-scene kinds.
- The **Ghidra project is in this worktree's `extracted/`**, not the main checkout's, so it is lost if
  the worktree is cleaned. Step 4 has the two commands that rebuild it.
- `MX08AUD.PSR` holds 16 more `.bpb` (`Aud\carengine.aud.bpb`, `Aud\mix.pb.bpb`, …). Run by hand
  (`discx x … --psr MX08AUD.PSR`, then `bpb -r`), all 16 decode with every hash matching. They are not
  in the disc test, so they were not re-encoded byte for byte.
