//! A generated `.vig` and playlist through the reader checks, the playlist reader and a whole pack,
//! which modcheck then proves (all but the byte check against a disc, which needs one).

use bpb::{Bpb, Key, Section, Value, hash};
use import_music::{Origin, PACKS, ReadBack, build, check, ini_text, juk, playlist, read_back};
use std::collections::HashMap;
use std::path::PathBuf;

/// A stereo `.vig` of `pairs` block pairs: the header's size one pair past the file, as on both discs,
/// the last pair flagged as the end, then `filler` pairs of `0x07` blocks (MX vs. ATV Unleashed's).
/// Each sample is a nibble at shift 0 (`n × 4096`), the left and right the same unless `mirror`.
fn vig(pairs: usize, rate: u32, vagp: bool, filler: usize, mirror: bool) -> Vec<u8> {
    let mut b = vec![0u8; 0x800];
    if vagp {
        b[..4].copy_from_slice(b"VAGp");
    }
    b[0x0c..0x10].copy_from_slice(&(((pairs + filler + 1) * 16) as u32).to_be_bytes());
    b[0x10..0x14].copy_from_slice(&rate.to_be_bytes());
    for i in 0..pairs {
        for ch in 0..2 {
            let mut blk = [0u8; 16];
            blk[1] = if i + 1 == pairs { 1 } else { 0 };
            for j in 0..14 {
                let n = ((i + j) % 8) as u8;
                let n = if ch == 1 && mirror { (16 - n) & 15 } else { n };
                blk[2 + j] = n | n << 4;
            }
            b.extend_from_slice(&blk);
        }
    }
    for _ in 0..2 * filler {
        let mut blk = [0x77u8; 16];
        blk[0] = 0x07;
        blk[1] = 0;
        b.extend_from_slice(&blk);
    }
    b
}

#[test]
fn a_generated_vig_reads_back_as_the_header_says() {
    let r = read_back(&vig(100, 32000, false, 0, false)).unwrap();
    assert_eq!((r.channels, r.rate, r.header_rate, r.frames, r.stream_frames), (2, 32000, 32000, 2800, 2800));
    assert_eq!((r.size, r.unread, r.after_end, r.vagp), (101 * 16, -32, Some(0), false));
    assert!(r.correlation > 0.999, "{}", r.correlation);
    assert!(r.problems().is_empty(), "{:?}", r.problems());
    assert!((r.header_seconds() - 101.0 * 28.0 / 32000.0).abs() < 1e-9);
    let r = read_back(&vig(100, 44100, true, 0, true)).unwrap();
    assert!(r.vagp);
    assert!(r.correlation < -0.9, "a mirrored right channel: {}", r.correlation);
}

#[test]
fn what_openmx_would_play_differently_is_a_problem() {
    let r = read_back(&vig(50, 44100, false, 1, false)).unwrap();
    assert_eq!(r.after_end, Some(1));
    assert_eq!(r.frames, 51 * 28);
    assert!(r.problems().iter().any(|p| p.contains("1 block pair(s) (28 frames) past the end flag")), "{:?}", r.problems());

    // A rate the reader does not believe is played at 44100.
    let r = read_back(&vig(50, 0, false, 0, false)).unwrap();
    assert!(r.problems().iter().any(|p| p == "header rate 0 read as 44100"), "{:?}", r.problems());

    // A header that says less than the file holds leaves the rest unplayed.
    let mut b = vig(50, 44100, false, 0, false);
    b[0x0c..0x10].copy_from_slice(&(40u32 * 16).to_be_bytes());
    let r: ReadBack = read_back(&b).unwrap();
    assert_eq!(r.unread, 10 * 32);
    assert_eq!(r.after_end, None, "the end flag is past the size");
    assert_eq!(r.problems().len(), 2, "{:?}", r.problems());

    assert!(read_back(&[0u8; 0x810]).is_err());
}

const PB: &str = "[Songs]\nNumberOfSongs=3\nOrdered=1\n\n[Artists]\nNumberOfArtists=2\nArtist1=\"BAND ONE\"\nArtist2=\"BAND TWO\"\n\n\
[Song 1]\nFilename=Music\\Zed_Song\nTitle=\"ZED; NOT A COMMENT?\"\nArtist=2\n\n\
[Song 2]\nFilename=Music\\alpha\nTitle=\"ALPHA\"\nArtist=1\nYear=2000\n\n\
[Song 3]\nFilename=Music\\BETA\nTitle=BETA\nArtist=1\n";

fn members() -> Vec<String> {
    ["Aud\\Music\\music.pb", "Aud\\Music\\ALPHA.vig", "Aud\\Music\\beta.vig", "Aud\\Music\\Zed_Song.vig", "Aud\\Music\\x.VAG", "Aud\\streams.juk"]
        .map(String::from)
        .to_vec()
}

#[test]
fn the_playlist_keeps_its_order_and_the_discs_spelling() {
    // `;` starts a comment in the disc's INI reader (game-render's, as openmx reads music.pb).
    let text = PB.replace("ZED; NOT A COMMENT?", "ZED?");
    let songs = playlist(&text, "Aud\\Music\\music.pb", &members()).unwrap();
    let got: Vec<_> = songs.iter().map(|s| (s.member.as_str(), s.title.as_str(), s.artist.as_str(), s.from.as_str())).collect();
    assert_eq!(
        got,
        [
            ("Aud\\Music\\Zed_Song.vig", "ZED?", "BAND TWO", "Aud\\Music\\music.pb [Song 1]"),
            ("Aud\\Music\\ALPHA.vig", "ALPHA", "BAND ONE", "Aud\\Music\\music.pb [Song 2]"),
            ("Aud\\Music\\beta.vig", "BETA", "BAND ONE", "Aud\\Music\\music.pb [Song 3]"),
        ]
    );
    assert_eq!(songs[1].file(), "Aud/Music/ALPHA.vig");
}

#[test]
fn a_playlist_that_does_not_add_up_is_refused() {
    let m = members();
    let e = playlist(&PB.replace("NumberOfSongs=3", "NumberOfSongs=4"), "pb", &m).unwrap_err();
    assert!(e.contains("NumberOfSongs is Some(4)"), "{e}");
    let e = playlist(&PB.replace("Music\\BETA", "Music\\GAMMA"), "pb", &m).unwrap_err();
    assert!(e.contains("[Song 3]: aud\\music\\gamma.vig is not on the disc"), "{e}");
    let e = playlist(&PB.replace("Artist=2", "Artist=3"), "pb", &m).unwrap_err();
    assert!(e.contains("[Song 1]: title \"ZED\", artist \"\""), "{e}");
    let e = playlist(&PB.replace("NumberOfArtists=2", "NumberOfArtists=1"), "pb", &m).unwrap_err();
    assert!(e.contains("NumberOfArtists"), "{e}");
}

/// The same playlist as Untamed keeps it: BPB, sections `Song ` with an index, no quotes.
fn pb_bpb() -> Vec<u8> {
    let sec = |name: &str, index: Option<u16>, keys: &[(&str, Value)]| Section {
        hash: hash(name.as_bytes()),
        name: name.as_bytes().to_vec(),
        index,
        keys: keys.iter().map(|(k, v)| Key { hash: hash(k.as_bytes()), name: k.as_bytes().to_vec(), value: v.clone() }).collect(),
    };
    let t = |s: &str| Value::Text(s.as_bytes().to_vec());
    let bpb = Bpb {
        sections: vec![
            sec("Songs", None, &[("NumberOfSongs", Value::Int(2)), ("Ordered", Value::Int(1))]),
            sec("Artists", None, &[("NumberOfArtists", Value::Int(1)), ("Artist1", t("THE BAND"))]),
            sec("Song ", Some(1), &[("Filename", t("Music\\beta")), ("Title", t("12:34")), ("Artist", Value::Int(1))]),
            sec("Song ", Some(2), &[("Filename", t("Music\\alpha")), ("Title", t("HAPPY?")), ("Artist", Value::Int(1))]),
        ],
    };
    bpb::write(&bpb).unwrap()
}

#[test]
fn a_bpb_playlist_reads_as_its_text_would() {
    let text = ini_text("Aud\\Music\\music.pb.bpb", &pb_bpb()).unwrap();
    let songs = playlist(&text, "Aud\\Music\\music.pb.bpb", &members()).unwrap();
    let got: Vec<_> = songs.iter().map(|s| (s.member.as_str(), s.title.as_str(), s.artist.as_str(), s.from.as_str())).collect();
    assert_eq!(
        got,
        [
            ("Aud\\Music\\beta.vig", "12:34", "THE BAND", "Aud\\Music\\music.pb.bpb [Song 1]"),
            ("Aud\\Music\\ALPHA.vig", "HAPPY?", "THE BAND", "Aud\\Music\\music.pb.bpb [Song 2]"),
        ]
    );
}

#[test]
fn streams_juk_reads_both_spellings() {
    let j = juk("[Song 1]\nFilename = Aud\\Music\\A.vig\nSampleRate = 44100\nSize = 9060096\nDuration = 179.723801\nPitch = 3763\nDataOffset = 0\n\n[Song 2]\nFilename=Aud\\UI\\b.vig\nSampleRate=22050\nSize=10\nDuration=184.0266571044922\nDataOffset=0\n");
    assert_eq!(j.len(), 2);
    assert_eq!((j[0].file.as_str(), j[0].rate, j[0].size, j[0].offset), ("Aud\\Music\\A.vig", 44100, 9060096, 0));
    assert!((j[1].seconds - 184.0266571044922).abs() < 1e-12);
}

fn scratch(name: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("import-music-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    d
}

#[test]
fn a_generated_pack_is_written_whole_and_proves() {
    let pack = &PACKS[0];
    let mut files: HashMap<String, Vec<u8>> = HashMap::new();
    files.insert("Aud\\Music\\music.pb".into(), PB.as_bytes().to_vec());
    files.insert("Aud\\Music\\Zed_Song.vig".into(), vig(30, 44100, false, 1, false));
    files.insert("Aud\\Music\\ALPHA.vig".into(), vig(20, 44100, false, 1, false));
    files.insert("Aud\\Music\\beta.vig".into(), vig(10, 44100, false, 1, false));
    // streams.juk as the disc has it: the size of the file, the duration of the header.
    let juk_text: String = ["Zed_Song", "ALPHA", "beta"]
        .iter()
        .map(|n| {
            let b = &files[&format!("Aud\\Music\\{n}.vig")];
            let size = u32::from_be_bytes(b[0x0c..0x10].try_into().unwrap());
            format!("[Song]\nFilename = Aud\\Music\\{n}.vig\nSampleRate = 44100\nSize = {}\nDuration = {:.6}\nDataOffset = 0\n\n", b.len(), size as f64 / 16.0 * 28.0 / 44100.0)
        })
        .collect();
    files.insert("Aud\\streams.juk".into(), juk_text.into_bytes());
    let mut read = |name: &str| files.get(name).cloned().ok_or_else(|| format!("{name}: no such member"));
    let origin = Origin { game: "MX vs. ATV Unleashed".into(), boot: "SLUS_211.04".into() };
    let root = scratch("built");
    let built = build(pack, &origin, &members(), &mut read, &root).unwrap();

    assert_eq!(built.songs.len(), 3);
    assert_eq!(
        built.todo,
        [
            "Aud\\Music\\music.pb: titles and artists are music.pb's; not yet compared with the jukebox's screen in PCSX2 (T7)",
            "Aud/Music/*.vig (all 3): Clip::vig and Adpcm::vig play 1 block pair(s) (28 frames) past the end flag, which the game's SPU presumably stops at",
        ]
    );
    for (name, bytes) in &files {
        if name.ends_with("streams.juk") {
            assert!(!built.dir.join("Aud/streams.juk").exists(), "the table is read, not packed");
            continue;
        }
        assert_eq!(&std::fs::read(built.dir.join(name.replace('\\', "/"))).unwrap(), bytes, "{name}");
    }
    let back = check(&built.dir).unwrap();
    assert_eq!(back.iter().map(|(f, r)| (f.as_str(), r.frames)).collect::<Vec<_>>(), [("Aud/Music/Zed_Song.vig", 31 * 28), ("Aud/Music/ALPHA.vig", 21 * 28), ("Aud/Music/beta.vig", 11 * 28)]);

    // The standard's checks, all but the copy against a disc this test does not have.
    let r = modcheck::check(&built.dir, &mut modcheck::Discs::new(None));
    let fails: Vec<_> = r.fail.iter().filter(|f| !f.starts_with("cannot prove the copy")).collect();
    assert!(fails.is_empty(), "{fails:?}");
    assert_eq!(r.id.as_deref(), Some("mxvatv_music"));

    // Built again over itself, it is replaced; over a folder that is not its own, it stops.
    build(pack, &origin, &members(), &mut read, &root).unwrap();
    std::fs::write(built.dir.join("mod.json"), "{\"id\": \"someone_else\"}").unwrap();
    let e = build(pack, &origin, &members(), &mut read, &root).unwrap_err();
    assert!(e.contains("not this tool's mxvatv_music pack"), "{e}");
    std::fs::remove_dir_all(&root).unwrap();
}

#[test]
fn a_stream_that_disagrees_with_streams_juk_is_a_todo() {
    let pack = &PACKS[0];
    let mut files: HashMap<String, Vec<u8>> = HashMap::new();
    files.insert("Aud\\Music\\music.pb".into(), "[Songs]\nNumberOfSongs=1\n[Artists]\nNumberOfArtists=1\nArtist1=A\n[Song 1]\nFilename=Music\\beta\nTitle=B\nArtist=1\n".into());
    files.insert("Aud\\Music\\beta.vig".into(), vig(10, 44100, false, 0, false));
    files.insert("Aud\\streams.juk".into(), "[Song 1]\nFilename=Aud\\Music\\beta.vig\nSampleRate=44100\nSize=5\nDuration=1.0\n".into());
    let mut read = |name: &str| files.get(name).cloned().ok_or_else(|| format!("{name}: no such member"));
    let origin = Origin { game: "MX vs. ATV Unleashed".into(), boot: "SLUS_211.04".into() };
    let root = scratch("juk");
    let built = build(pack, &origin, &members(), &mut read, &root).unwrap();
    assert_eq!(built.todo.len(), 3, "{:?}", built.todo);
    assert!(built.todo[1].starts_with("Aud/Music/beta.vig: Aud\\streams.juk has 5 bytes at 44100 Hz from 0, the file"), "{:?}", built.todo);
    assert!(built.todo[2].contains("lasts 1 s, the header"), "{:?}", built.todo);
    std::fs::remove_dir_all(&root).unwrap();
}
