//! MX vs. ATV Unleashed's and Untamed's songs as music packs (`standard/OPENMX-TRACK-STANDARD.md` §4).
//!
//! ```text
//! MXVSATV.PSR ── Aud\Music\music.pb        the jukebox's playlist: [Artists], then [Song N]
//!            ├── Aud\Music\<song>.vig      one stereo stream per song
//!            └── Aud\streams.juk           every stream's size and duration (the streamer's table)
//! MX08AUD.PSR ── the same, as Aud\Music\music.pb.bpb and Aud\streams.juk.bpb (BPB, tools/bpb)
//! ```
//!
//! A pack is the playlist member and every song's `.vig`, copied byte for byte to their own paths, and
//! a `mod.json` whose `songs` are the playlist in its own order. The playlist is read as openmx's
//! `DiscMusic::list` reads MX Unleashed's: `Filename` under `Aud\` with `.vig` added, `Title`, and
//! `Artist` as a number into `[Artists]`. Every stream is read back with openmx's own `.vig` reader
//! ([`audio::Clip::vig`] and [`audio::Adpcm::vig`]), and its header is held against `streams.juk`.
//! Anything the reader would play differently from the disc goes in `todo`; the file is still copied.

/// openmx's `crates/openmx/src/audio.rs`, compiled in from its own file: the `.vig` reader
/// (`Clip::vig`, `Adpcm::vig`) is a module of the shell's binary, which no other crate can depend on
/// (notes/openmx-asks.md, MD7). Only the readers are used; the mixer and the player come along unused.
#[path = "../../../openmx/crates/openmx/src/audio.rs"]
#[allow(dead_code, unused_imports, unused_mut, unused_variables, clippy::all)]
pub mod audio;

/// openmx's `crates/openmx/src/thread.rs`, which audio.rs's player spawns through (`crate::thread`).
#[path = "../../../openmx/crates/openmx/src/thread.rs"]
#[allow(dead_code, unused_imports)]
mod thread;

use audio::{Adpcm, Clip, Stream};
use game_render::ini;
use std::path::{Path, PathBuf};

/// The tool, as `mod.json`'s `author` names it.
pub const AUTHOR: &str = "openmx-mods tools/import-music 0.1";

/// One game's soundtrack and where its disc keeps it.
#[derive(Clone, Copy, Debug)]
pub struct Pack {
    /// discx's key for the game (`mxvatv`, `untamed`).
    pub game: &'static str,
    /// The pack's id and folder under `mods/music/`.
    pub id: &'static str,
    pub name: &'static str,
    /// The PSR the songs are in.
    pub psr: &'static str,
    /// The jukebox's playlist.
    pub playlist: &'static str,
    /// The streamer's table of every stream's size and duration.
    pub streams: &'static str,
}

pub const PACKS: &[Pack] = &[
    Pack {
        game: "mxvatv",
        id: "mxvatv_music",
        name: "MX vs. ATV Unleashed soundtrack",
        psr: "MXVSATV.PSR",
        playlist: "Aud\\Music\\music.pb",
        streams: "Aud\\streams.juk",
    },
    Pack {
        game: "untamed",
        id: "untamed_music",
        name: "MX vs. ATV Untamed soundtrack",
        psr: "MX08AUD.PSR",
        playlist: "Aud\\Music\\music.pb.bpb",
        streams: "Aud\\streams.juk.bpb",
    },
];

impl Pack {
    pub fn by_game(name: &str) -> Option<&'static Pack> {
        PACKS.iter().find(|p| p.game.eq_ignore_ascii_case(name) || p.id.eq_ignore_ascii_case(name))
    }
}

/// An INI member as text: BPB (Untamed) decoded by tools/bpb, anything else as the disc writes text.
pub fn ini_text(name: &str, bytes: &[u8]) -> Result<String, String> {
    if name.to_ascii_lowercase().ends_with(".bpb") {
        let b = bpb::parse(bytes).map_err(|e| format!("{name}: {e}"))?;
        Ok(ini::text(&b.to_ini()))
    } else {
        Ok(ini::text(bytes))
    }
}

/// One song of the playlist.
#[derive(Clone, Debug, PartialEq)]
pub struct Song {
    /// The PSR member, as the PSR spells it (`Aud\Music\RISE_AGAINST_GIVE.vig`).
    pub member: String,
    pub title: String,
    pub artist: String,
    /// The playlist member and section it was read from: `Aud\Music\music.pb [Song 15]`.
    pub from: String,
}

impl Song {
    /// Its path in the pack: the member's, `\` as `/`.
    pub fn file(&self) -> String {
        self.member.replace('\\', "/")
    }
}

fn unquote(s: &str) -> String {
    s.trim().trim_matches('"').to_string()
}

/// The playlist, in its own order. `members` are the PSR's member names; a song's file is matched to
/// one without regard to case or slash, as the game opens it. A song with no file, no title or no
/// artist, or a count that disagrees with `NumberOfSongs` / `NumberOfArtists`, is an error: the
/// jukebox's list is not guessed at.
pub fn playlist(text: &str, playlist_member: &str, members: &[String]) -> Result<Vec<Song>, String> {
    let sections = ini::parse(text);
    let find = |name: &str| sections.iter().find(|s| s.name.eq_ignore_ascii_case(name));
    let artists = find("Artists").ok_or_else(|| format!("{playlist_member}: no [Artists]"))?;
    let fold = |s: &str| s.replace('/', "\\").to_ascii_lowercase();
    let mut out = Vec::new();
    let mut why = Vec::new();
    for s in sections.iter().filter(|s| s.name.to_ascii_lowercase().starts_with("song ")) {
        let from = format!("{playlist_member} [{}]", s.name);
        let Some(file) = s.get("Filename") else {
            why.push(format!("{from}: no Filename"));
            continue;
        };
        let want = fold(&format!("Aud\\{}.vig", file.trim()));
        let Some(member) = members.iter().find(|m| fold(m) == want) else {
            why.push(format!("{from}: {want} is not on the disc"));
            continue;
        };
        let title = s.get("Title").map(unquote).unwrap_or_default();
        let artist = s
            .get("Artist")
            .and_then(|n| artists.get(&format!("Artist{}", n.trim())))
            .map(unquote)
            .unwrap_or_default();
        if title.is_empty() || artist.is_empty() {
            why.push(format!("{from}: title {title:?}, artist {artist:?}"));
            continue;
        }
        out.push(Song { member: member.clone(), title, artist, from });
    }
    let count = |sec: &str, key: &str| find(sec).and_then(|s| s.number(key)).map(|n| n as usize);
    match count("Songs", "NumberOfSongs") {
        Some(n) if n == out.len() + why.len() => {}
        n => why.push(format!("{playlist_member}: NumberOfSongs is {n:?}, the list has {}", out.len() + why.len())),
    }
    let listed = artists.keys.iter().filter(|(k, _)| k.to_ascii_lowercase().starts_with("artist")).count();
    if count("Artists", "NumberOfArtists") != Some(listed) {
        why.push(format!("{playlist_member}: NumberOfArtists is {:?}, [Artists] has {listed}", count("Artists", "NumberOfArtists")));
    }
    if out.is_empty() && why.is_empty() {
        why.push(format!("{playlist_member}: no [Song N]"));
    }
    if why.is_empty() { Ok(out) } else { Err(why.join("; ")) }
}

/// One stream's entry in `streams.juk`.
#[derive(Clone, Debug, PartialEq)]
pub struct JukEntry {
    pub file: String,
    pub rate: u32,
    pub size: u64,
    pub seconds: f64,
    pub offset: u64,
}

/// `streams.juk`: `[Song N]` sections of `Filename`, `SampleRate`, `Size`, `Duration`, `Pitch` and
/// `DataOffset`, one per stream the game has (songs, start-line ditties, menu loops).
pub fn juk(text: &str) -> Vec<JukEntry> {
    ini::parse(text)
        .iter()
        .filter_map(|s| {
            Some(JukEntry {
                file: s.get("Filename")?.trim().to_string(),
                rate: s.get("SampleRate")?.trim().parse().ok()?,
                size: s.get("Size")?.trim().parse().ok()?,
                seconds: s.get("Duration")?.trim().parse().ok()?,
                offset: s.get("DataOffset").and_then(|v| v.trim().parse().ok()).unwrap_or(0),
            })
        })
        .collect()
}

fn be32(b: &[u8], at: usize) -> u32 {
    u32::from_be_bytes([b[at], b[at + 1], b[at + 2], b[at + 3]])
}

/// What openmx's reader made of one `.vig`, and what its header says.
#[derive(Clone, Debug, PartialEq)]
pub struct ReadBack {
    /// The header's per-channel data size (`0x0c`, big-endian) and rate (`0x10`).
    pub size: u32,
    pub header_rate: u32,
    /// Untamed's headers start `VAGp`; MX vs. ATV Unleashed's are zero there.
    pub vagp: bool,
    /// `Clip::vig`'s rate, channels and frames.
    pub rate: u32,
    pub channels: u8,
    pub frames: usize,
    /// The frames `Adpcm::vig`, the player's stream, gives before it runs out.
    pub stream_frames: usize,
    /// Bytes of the file past `0x800 + 2 × size`, which the reader leaves unread (never more than 0
    /// on these discs: the size runs one block pair, 32 bytes, past the file).
    pub unread: i64,
    /// The left and right channels' correlation over the whole song: a stereo song read with the
    /// wrong interleave decodes to two unrelated noises, near 0.
    pub correlation: f64,
    /// Block pairs after the first one flagged as the end (flags bit 0, in either channel), which
    /// `decode_stereo` and `Adpcm` play though `decode_mono` would stop there; `None` if no block is.
    /// Untamed's songs end on the flag (0); MX vs. ATV Unleashed's have one more pair of `0x07`
    /// filler blocks (1), 28 frames of a constant 224.
    pub after_end: Option<usize>,
}

impl ReadBack {
    pub fn seconds(&self) -> f64 {
        self.frames as f64 / self.rate.max(1) as f64
    }

    /// The header's length in seconds, as the game's `streams.juk` counts it.
    pub fn header_seconds(&self) -> f64 {
        self.size as f64 / 16.0 * 28.0 / self.header_rate.max(1) as f64
    }

    /// Why openmx would not play this as the disc has it, if it would not.
    pub fn problems(&self) -> Vec<String> {
        let mut p = Vec::new();
        if self.channels != 2 {
            p.push(format!("read as {} channel(s), not stereo", self.channels));
        }
        if self.rate != self.header_rate {
            p.push(format!("header rate {} read as {}", self.header_rate, self.rate));
        }
        if self.stream_frames != self.frames {
            p.push(format!("Adpcm::vig gives {} frames, Clip::vig {}", self.stream_frames, self.frames));
        }
        if self.unread > 0 {
            p.push(format!("{} bytes past the header's size are not played", self.unread));
        }
        match self.after_end {
            None => p.push("no block is flagged as the end".into()),
            Some(0) => {}
            Some(n) => p.push(format!(
                "Clip::vig and Adpcm::vig play {n} block pair(s) ({} frames) past the end flag, which the game's SPU presumably stops at",
                n * 28
            )),
        }
        p
    }
}

/// One `.vig` read back with openmx's reader.
pub fn read_back(bytes: &[u8]) -> Result<ReadBack, String> {
    if bytes.len() < 0x800 + 32 {
        return Err(format!("{} bytes: shorter than a header and one block pair", bytes.len()));
    }
    let clip = Clip::vig(bytes)?;
    let size = be32(bytes, 0x0c);
    let mut stream = Adpcm::vig(bytes.to_vec());
    let mut buf = vec![0i16; 2 * 4096];
    let mut stream_frames = 0;
    loop {
        let n = stream.read(&mut buf);
        stream_frames += n;
        if n < 4096 {
            break;
        }
    }
    let ch = clip.channels.max(1) as usize;
    let (mut sl, mut sr, mut sll, mut srr, mut slr) = (0f64, 0f64, 0f64, 0f64, 0f64);
    for f in clip.pcm.chunks_exact(ch) {
        let (l, r) = (f[0] as f64, f[ch - 1] as f64);
        sl += l;
        sr += r;
        sll += l * l;
        srr += r * r;
        slr += l * r;
    }
    let n = clip.frames().max(1) as f64;
    let var = ((sll - sl * sl / n) * (srr - sr * sr / n)).sqrt();
    let correlation = if var > 0.0 { (slr - sl * sr / n) / var } else { 0.0 };
    let pairs = (bytes.len().min(0x800 + 2 * size as usize) - 0x800) / 32;
    let after_end = (0..pairs)
        .position(|i| (bytes[0x800 + 32 * i + 1] | bytes[0x800 + 32 * i + 17]) & 1 != 0)
        .map(|i| pairs - 1 - i);
    Ok(ReadBack {
        size,
        header_rate: be32(bytes, 0x10),
        vagp: bytes.starts_with(b"VAGp"),
        rate: clip.rate,
        channels: clip.channels,
        frames: clip.frames(),
        stream_frames,
        unread: bytes.len() as i64 - (0x800 + 2 * size as i64),
        correlation,
        after_end,
    })
}

/// A song packaged: where it came from and what the reader made of it.
#[derive(Clone, Debug)]
pub struct Packed {
    pub song: Song,
    pub read: ReadBack,
    pub bytes: u64,
}

/// A pack written out.
#[derive(Clone, Debug)]
pub struct Built {
    pub dir: PathBuf,
    pub songs: Vec<Packed>,
    pub todo: Vec<String>,
}

/// What the disc is, for `mod.json`'s `source`.
#[derive(Clone, Debug)]
pub struct Origin {
    pub game: String,
    pub boot: String,
}

/// Build a pack in `root/<id>/` from the disc's members, which `read` gives by PSR name. `members`
/// is the PSR's directory. A folder already there is replaced, but only if it is this pack's.
pub fn build(
    pack: &Pack,
    origin: &Origin,
    members: &[String],
    read: &mut dyn FnMut(&str) -> Result<Vec<u8>, String>,
    root: &Path,
) -> Result<Built, String> {
    let name_of = |want: &str| {
        members
            .iter()
            .find(|m| m.eq_ignore_ascii_case(want))
            .cloned()
            .ok_or_else(|| format!("{want} is not in {}", pack.psr))
    };
    let pb_member = name_of(pack.playlist)?;
    let pb = read(&pb_member)?;
    let songs = playlist(&ini_text(&pb_member, &pb)?, &pb_member, members)?;
    let streams_member = name_of(pack.streams)?;
    let streams = juk(&ini_text(&streams_member, &read(&streams_member)?)?);

    let dir = root.join(pack.id);
    clear(&dir, pack.id)?;
    let put = |member: &str, bytes: &[u8]| -> Result<(), String> {
        let path = dir.join(member.replace('\\', "/"));
        std::fs::create_dir_all(path.parent().unwrap()).map_err(|e| format!("{}: {e}", path.display()))?;
        std::fs::write(&path, bytes).map_err(|e| format!("{}: {e}", path.display()))
    };
    put(&pb_member, &pb)?;

    let mut todo = vec![format!(
        "{}: titles and artists are music.pb's; not yet compared with the jukebox's screen in PCSX2 (T7)",
        pb_member
    )];
    let mut packed = Vec::new();
    let mut problems: std::collections::BTreeMap<String, Vec<String>> = Default::default();
    let total = songs.len();
    for song in songs {
        let bytes = read(&song.member)?;
        put(&song.member, &bytes)?;
        let file = song.file();
        let rb = match read_back(&bytes) {
            Ok(rb) => rb,
            Err(e) => {
                todo.push(format!("{file}: openmx's Clip::vig refuses it: {e}"));
                continue;
            }
        };
        for p in rb.problems() {
            problems.entry(p).or_default().push(file.clone());
        }
        match streams.iter().find(|j| j.file.eq_ignore_ascii_case(&song.member)) {
            None => todo.push(format!("{file}: not in {streams_member}")),
            Some(j) => {
                if j.size != bytes.len() as u64 || j.rate != rb.header_rate || j.offset != 0 {
                    todo.push(format!(
                        "{file}: {streams_member} has {} bytes at {} Hz from {}, the file {} at {} Hz",
                        j.size,
                        j.rate,
                        j.offset,
                        bytes.len(),
                        rb.header_rate
                    ));
                }
                if (j.seconds - rb.header_seconds()).abs() > 1e-4 {
                    todo.push(format!("{file}: {streams_member} lasts {} s, the header {} s", j.seconds, rb.header_seconds()));
                }
            }
        }
        packed.push(Packed { song, read: rb, bytes: bytes.len() as u64 });
    }
    // A problem every song has is one line.
    for (p, files) in problems {
        if files.len() == total && total > 1 {
            todo.push(format!("Aud/Music/*.vig (all {total}): {p}"));
        } else {
            todo.extend(files.iter().map(|f| format!("{f}: {p}")));
        }
    }
    let mut source_members = vec![pb_member.clone()];
    source_members.extend(packed.iter().map(|p| p.song.member.clone()));
    let json = mod_json(pack, origin, &source_members, &packed, &todo);
    std::fs::write(dir.join("mod.json"), json).map_err(|e| format!("{}: {e}", dir.display()))?;
    Ok(Built { dir, songs: packed, todo })
}

/// Make way for a pack: a folder that is not there, or one whose `mod.json` is this pack's, removed.
fn clear(dir: &Path, id: &str) -> Result<(), String> {
    if !dir.exists() {
        return Ok(());
    }
    let ours = std::fs::read_to_string(dir.join("mod.json"))
        .is_ok_and(|t| t.contains(&format!("\"id\": {}", json_str(id))) && t.contains("\"kind\": \"music\""));
    if !ours {
        return Err(format!("{}: there already, and not this tool's {id} pack; not touched", dir.display()));
    }
    std::fs::remove_dir_all(dir).map_err(|e| format!("{}: {e}", dir.display()))
}

/// A JSON string.
pub fn json_str(s: &str) -> String {
    let mut o = String::from("\"");
    for c in s.chars() {
        match c {
            '"' => o.push_str("\\\""),
            '\\' => o.push_str("\\\\"),
            '\n' => o.push_str("\\n"),
            c if (c as u32) < 0x20 => o.push_str(&format!("\\u{:04x}", c as u32)),
            c => o.push(c),
        }
    }
    o.push('"');
    o
}

/// `mod.json` (§1, §4). Beside v1's fields each song has `seconds`, its length as openmx reads it.
pub fn mod_json(pack: &Pack, origin: &Origin, members: &[String], songs: &[Packed], todo: &[String]) -> String {
    let list = |items: Vec<String>, indent: &str| {
        if items.is_empty() {
            "[]".to_string()
        } else {
            format!("[\n{indent}  {}\n{indent}]", items.join(&format!(",\n{indent}  ")))
        }
    };
    let songs: Vec<String> = songs
        .iter()
        .map(|p| {
            format!(
                "{{\"file\": {}, \"title\": {}, \"artist\": {}, \"from\": {}, \"seconds\": {:.3}}}",
                json_str(&p.song.file()),
                json_str(&p.song.title),
                json_str(&p.song.artist),
                json_str(&p.song.from),
                p.read.seconds()
            )
        })
        .collect();
    format!(
        "{{\n  \"standard\": 1,\n  \"kind\": \"music\",\n  \"id\": {},\n  \"name\": {},\n  \"source\": {{\n    \"game\": {},\n    \"boot\": {},\n    \"psr\": {},\n    \"members\": {}\n  }},\n  \"version\": \"1.0.0\",\n  \"author\": {},\n  \"permission\": \"own-disc\",\n  \"songs\": {},\n  \"todo\": {}\n}}\n",
        json_str(pack.id),
        json_str(pack.name),
        json_str(&origin.game),
        json_str(&origin.boot),
        json_str(pack.psr),
        list(members.iter().map(|m| json_str(m)).collect(), "    "),
        json_str(AUTHOR),
        list(songs, "  "),
        list(todo.iter().map(|t| json_str(t)).collect(), "  "),
    )
}

/// Build a pack off a disc image.
pub fn import(image: &Path, pack: &Pack, root: &Path) -> Result<Built, String> {
    let mut disc = discx::Disc::open_psr(image, Some(pack.psr))?;
    let game = disc.game.ok_or_else(|| format!("{}: not a disc this repo knows", image.display()))?;
    if game.key != pack.game {
        return Err(format!("{}: is {} ({}), not {}", image.display(), game.title, game.boot, pack.name));
    }
    let origin = Origin { game: game.title.to_string(), boot: game.boot.to_string() };
    let members: Vec<String> = disc.entries().iter().map(|e| e.name.clone()).collect();
    let mut read = |name: &str| {
        let e = disc.find(name).cloned().ok_or_else(|| format!("{name}: not in {}", pack.psr))?;
        disc.read(&e)
    };
    build(pack, &origin, &members, &mut read, root)
}

/// Every song of a built pack read back again from its folder with openmx's reader: the file each
/// `songs` entry names, and its frames against the `seconds` the pack says.
pub fn check(dir: &Path) -> Result<Vec<(String, ReadBack)>, String> {
    use modcheck::json::Json;
    let text = std::fs::read_to_string(dir.join("mod.json")).map_err(|e| format!("{}: {e}", dir.display()))?;
    let j = Json::parse(&text).map_err(|e| format!("{}: mod.json: {e}", dir.display()))?;
    let mut out = Vec::new();
    for s in j.get("songs").and_then(Json::arr).unwrap_or_default() {
        let file = s.get("file").and_then(Json::str).ok_or("a song with no file")?.to_string();
        let bytes = std::fs::read(dir.join(&file)).map_err(|e| format!("{file}: {e}"))?;
        let rb = read_back(&bytes).map_err(|e| format!("{file}: {e}"))?;
        if let Some(said) = s.get("seconds").and_then(Json::num) {
            if (said - rb.seconds()).abs() > 0.001 {
                return Err(format!("{file}: mod.json says {said} s, the reader {} s", rb.seconds()));
            }
        }
        out.push((file, rb));
    }
    if out.is_empty() {
        return Err(format!("{}: no songs", dir.display()));
    }
    Ok(out)
}
