# Music: MX vs. ATV Unleashed and Untamed (A1)

The jukebox songs of both MX vs. ATV discs, and how `tools/import-music` packages them as music packs
(standard §4). MX Unleashed's own songs are in `Aud\audio.res` and the port already plays them
(openmx `crates/openmx/src/audio.rs`, `DiscMusic`). This file covers only the two later discs.

## Where the songs are

| | MX vs. ATV Unleashed (SLUS_211.04) | MX vs. ATV Untamed (SLUS_217.01) |
|---|---|---|
| container | `MXVSATV.PSR` | `MX08AUD.PSR`, the audio PSR (554 members) |
| songs | 20 `Aud\Music\*.vig`, 6.9–12.1 MB each, 188 MB in all | 22 `Aud\Music\*.vig`, 4.8–12.1 MB each, 191 MB in all |
| playlist | `Aud\Music\music.pb` (text, 2,644 bytes) | `Aud\Music\music.pb.bpb` (BPB, 4,064 bytes, tools/bpb) |
| stream table | `Aud\streams.juk` (text, 193 sections) | `Aud\streams.juk.bpb` (BPB); `Aud\streams.txt` lists the same paths |
| also in `Aud\Music` | menu loops as mono pairs: `PJ_UIL/UIR`, `MXvsATV_UIL/UIR`, `KOTTONMOUTH_UIL/UIR.VAG` | `PJ_UIL/UIR.VAG`, `MXvsATV_UIL.VAG` (no right half) |

The songs are loose PSR members (not in a sound bank as on MX Unleashed), so they are copied as members.

## The playlist

Both games use the same layout. Unleashed's is text and Untamed's is BPB, and they decode to the same keys:

```
[Songs]    NumberOfSongs, Ordered=1
[Artists]  NumberOfArtists, Artist1..ArtistN    (quoted in Unleashed, bare in Untamed)
[Genres]   NumberOfGenres, Genre1=Rock
[Song N]   Filename=Music\<stem>, Title, Artist=<number>, Genre, Locked=0
           (Unleashed's Song 1 also has Year, Album, Label)
```

`Filename` is relative to `Aud\` and has no extension. The stream is `Aud\<Filename>.vig`, matched without
regard to case: Unleashed's `Music\RISE_AGAINST_Give` is the member `RISE_AGAINST_GIVE.vig`. This is
the rule openmx's `DiscMusic::list` uses for MX Unleashed's `music.pb`. Both ELFs hold the strings this
reader needs (`Music\music.pb`, `Song %d`, `Title`, `Artist%d`, `NumberOfSongs`) and the jukebox's
`STR_DYN_JUKEBOX_SONG_TITLE` / `STR_DYN_JUKEBOX_BAND_NAME`. That is the static evidence that the
jukebox shows `music.pb`'s strings. The disc also has the credits' mixed-case names ("Heroes and
Villains" where `music.pb` has "HEROES & VILLAINS"). The pack uses `music.pb`'s strings. What the
jukebox actually puts on screen has not been compared in PCSX2 (T7).

Untamed's list uses artist 1 (AUTHORITY ZERO) for two songs (Song 1 and Song 16). Its 21 artists are
not in song order: Song 22 uses Artist16. Every `[Song N]` names a member that exists, and the counts
agree on both discs.

## The `.vig` streams

openmx's reader (`Clip::vig`, `Adpcm::vig`) takes all 42 songs: stereo, 44,100 Hz, 16-byte interleave
(one ADPCM block of left, then one of right).

- **Header, 0x800 bytes.** Big-endian per-channel size at `0x0c`, rate at `0x10` (`0x0000ac44`, 44100).
  Unleashed's header is otherwise zero. Untamed's starts `VAGp`, then version `0x20` at 4, and holds
  one stray 16-byte ADPCM block at `0x30` (different in each song, never the stream's first block). The
  reader ignores it. Whether the game ever plays it was not checked. The streamer's table says
  `DataOffset=0`, and its durations count only the header's size.
- **Size.** On every song of both discs, the header's size runs exactly one block pair (32 bytes) past
  the file. `streams.juk`'s `Size` is the file's length and its `Duration` is the header's size
  (`size / 16 × 28 / 44100`) to 1e-4 s on all 42 songs. openmx's reader plays to the end of the file,
  which is 28 frames short of `Duration`.
- **End.** Untamed's last block pair carries the end flag (flags bit 0) in both channels. Unleashed's
  end-flagged pair is followed by one more pair of filler blocks (`07 00 77 …`, filter 0, shift 7), which
  decodes to 28 frames of a constant 224. `decode_mono` stops at the end flag, but `decode_stereo` and
  `Adpcm` do not, so openmx plays the filler. It is 0.6 ms at about −43 dBFS, but it is not what the
  SPU is presumably told to play. It is in the pack's `todo` and in `notes/openmx-asks.md`.
- **Interleave.** No end flag appears before the last pairs. Decoded at 16 bytes, left and right
  correlate 0.49 to 0.91 over each whole song. With the wrong interleave the two channels would be
  unrelated noise.
- `streams.juk`'s `Pitch=3763` is 44100 / 48000 × 4096: the SPU plays the streams at their own rate.

## The packs

`out/mods/music/mxvatv_music/` and `out/mods/music/untamed_music/` (id = folder, §1 prefixes) hold
`mod.json`, the playlist member and every song's `.vig`, all at their member paths (`Aud/Music/…`) and
byte for byte as on the disc. `source.members` lists exactly those files. `source.psr` names the
container. Each song has the §4 fields plus `seconds` (its length as openmx reads it). `todo` lists the
jukebox check still owed (T7) and, on Unleashed, the filler pair. Build them with
`import-music mxvatv|untamed|all [OUT]` or `./build.sh` (its `--disc IMAGE --out STAGE` form). Read them
back with `import-music check DIR`. `modcheck` proves them byte for byte against the discs.

openmx's reader is a module of the shell's binary (`crates/openmx/src/main.rs` `mod audio`). That brings
wgpu and iroh with the package, so `tools/import-music` compiles `audio.rs` itself
(`#[path = "../../../openmx/crates/openmx/src/audio.rs"]`). It needs only `game-render` (without its GPU
half) and `ps2-float`. If openmx's `audio.rs` ever uses `crate::…`, that build breaks. The MD7 ask (a
crate of its own) would end that.
