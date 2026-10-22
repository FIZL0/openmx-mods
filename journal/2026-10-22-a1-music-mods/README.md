# A1: music packs from MX vs. ATV Unleashed and Untamed

`tools/import-music` builds `out/mods/music/mxvatv_music/` (20 songs) and `out/mods/music/untamed_music/`
(22 songs). Each holds the disc's playlist member, every song's `.vig` byte for byte at its member path,
and a `mod.json` (§4) with `songs` in the playlist's order: `file`, `title`, `artist` and `from`, plus
`seconds`. Usage: `import-music mxvatv|untamed|all [OUT]`, `import-music --disc IMAGE --out STAGE` (the
form `build.sh`'s A1 steps already call), and `import-music check DIR`. The findings are in
`notes/music.md`.

## How it went

1. **Members.** `discx ls` found 20 `.vig` plus `music.pb` in `MXVSATV.PSR`, and 22 `.vig` plus
   `music.pb.bpb` in `MX08AUD.PSR`. Both playlists have the same layout (`[Artists]`, `[Song N]` with
   `Filename`, `Title`, `Artist=<n>`). Untamed's decodes with tools/bpb. One case mismatch:
   Unleashed's `Music\RISE_AGAINST_Give` is the member `RISE_AGAINST_GIVE.vig`. The match ignores case
   as the game does, and the pack keeps the member's own spelling.
2. **openmx's reader.** `Clip::vig` / `Adpcm::vig` are in `crates/openmx/src/audio.rs`, a module of the
   shell's binary, not its library (`openmx_web` does not include it). Depending on the crate would also
   pull in wgpu and iroh (the existing MD7 ask). `include!` of the file failed (its `//!` inner docs
   are not allowed there). A `#[path]` module works: the file needs only `game-render` (without GPU)
   and `ps2-float`, and the tool runs openmx's real code unchanged.
3. **Headers.** I first thought Untamed's header size was 1 KiB short of the file. That was my hex
   arithmetic. On all 42 songs the size runs exactly one block pair past the file, as openmx's comment
   says for MX Unleashed.
4. **A cross-check from the game.** Both discs have `Aud\streams.juk` (Untamed: `.bpb`), the streamer's
   table, with each stream's `Size` and `Duration`. The importer holds every song against it. The
   sizes match the files, and the durations match the header (`size/16×28/44100`) within 1e-4 s.
5. **End flags.** I first checked that the last pair carries the end flag in both channels, and it
   flagged all 20 Unleashed songs. Their end-flagged pair is followed by one pair of `0x07` filler
   blocks. openmx's stereo decode does not stop at the flag (its mono decode does), so it plays 28
   frames of DC 224. The check became "pairs after the first end flag", which is 1 on Unleashed and 0
   on Untamed and nowhere earlier. The finding is one grouped `todo` line in the Unleashed pack, and an
   ask.
6. **Interleave.** The L/R correlation of each whole decoded song is 0.49 to 0.91. A wrong interleave
   would give unrelated noise. Taken with the end flags falling in both channels of the same final
   pair, this confirms 16-byte interleave.
7. **Jukebox strings.** I could not run the game. Static evidence: both ELFs contain `Music\music.pb`,
   `Song %d`, `Title`, `Artist%d` and `STR_DYN_JUKEBOX_SONG_TITLE/BAND_NAME`. The other copies of the
   titles on the discs are the credits, in mixed case and sometimes worded differently ("Heroes and
   Villains"). The pack uses `music.pb`'s strings, and a `todo` line in each pack says the on-screen
   comparison (T7) is owed.
8. **Packaging.** The id is the folder name (§1: `mxvatv_music`, `untamed_music`). The brief's
   `out/mods/music/<game>/` is read as that. The playlist member is copied too, so `source.members`
   (every file, listed exactly, not as a glob) accounts for everything in the folder and modcheck can
   prove it. The importer replaces an existing folder only if its `mod.json` is this pack's.

## How it was proved

- `cargo test -p import-music` (in the gate, 8 tests): a generated `.vig` (header, interleave,
  correlation, mirrored channel, filler pair, bad rate, short size, too short). Generated playlists,
  text and BPB (through `bpb::write`), with order, case, quotes and counts, and the refusals.
  `streams.juk` in both spellings. A whole generated pack written, files byte-equal, read back by
  `check`, and passed by `modcheck::check` (all but the disc copy). Rebuilding over itself works, and
  over a foreign folder it is refused. A `streams.juk` mismatch becomes a `todo`.
- `DISCX_ISO=…/iso cargo test -p import-music --release -- --ignored` (11 s): every song on both discs
  is packaged and read back with openmx's reader (stereo, 44,100 Hz, size one pair past the file,
  L/R > 0.3, over 90 s). The only `todo` lines are the expected ones, and `modcheck` passes both packs
  with 21 and 23 files compared byte for byte against the discs. It passed.
- `DISCX_ISO=… ./build.sh --check`: both packs built into this worktree's `out/mods/music/` (ignored),
  `modcheck: 2 ok, 0 failed`.
- `./check.sh` (run by `agent-runner done`).

## Not verified

- **Titles and artists against the jukebox's screen.** PCSX2 was not run (T7 is not done). The
  evidence is static (the ELF strings above). Each pack's `todo` says so.
- **Whether the game plays the filler pair** after Unleashed's end flag, or stops at the flag as
  assumed. The game's streamer was not decompiled.
- **Untamed's stray ADPCM block at header `0x30`.** Not explained. openmx ignores it. Whether the game
  plays it was not checked.
- **Whether MX Unleashed's own songs have the same filler pair.** Out of scope, and noted in the ask.
- **The packs in the main checkout's `out/`.** They were built only in this worktree's `out/`, because
  the session must not touch the main checkout. `./build.sh` there builds them.
- **The port playing a pack.** MD7 (the loader) is not in openmx yet. Only the reader was run, not the
  mixer or the audio output, and no one listened to the result.
