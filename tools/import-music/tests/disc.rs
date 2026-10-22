//! Every song on both discs: packaged, read back with openmx's reader, held against the game's own
//! `streams.juk`, and the pack proved by modcheck, byte for byte against the disc.
//!
//! `cargo test -p import-music --release -- --ignored`. The images are found by boot ELF in
//! `$DISCX_ISO`, else `iso/` at the repo root (a worktree has none: set `DISCX_ISO`).

use import_music::{PACKS, import};
use std::path::PathBuf;

fn iso_dir() -> PathBuf {
    std::env::var_os("DISCX_ISO")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../iso"))
}

#[test]
#[ignore = "needs the MX vs. ATV Unleashed and Untamed disc images"]
fn every_song_on_both_discs_packages_and_reads_back() {
    let root = std::env::temp_dir().join(format!("import-music-disc-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    for (pack, count) in PACKS.iter().zip([20, 22]) {
        let image = discx::resolve_image(pack.game, &iso_dir()).unwrap();
        let built = import(&image, pack, &root).unwrap();
        assert_eq!(built.songs.len(), count, "{}", pack.id);
        for p in &built.songs {
            let r = &p.read;
            assert_eq!((r.channels, r.rate, r.header_rate), (2, 44100, 44100), "{}", p.song.file());
            assert_eq!(r.unread, -32, "{}: the size runs one pair past the file", p.song.file());
            assert!(r.correlation > 0.3, "{}: L/R {}", p.song.file(), r.correlation);
            assert!(r.seconds() > 90.0, "{}: {} s", p.song.file(), r.seconds());
        }
        // The only gaps: the jukebox's screen (T7), and on MX vs. ATV Unleashed the filler pair.
        let expect = if pack.game == "mxvatv" { 2 } else { 1 };
        assert_eq!(built.todo.len(), expect, "{}: {:?}", pack.id, built.todo);
        assert_eq!(import_music::check(&built.dir).unwrap().len(), count);

        let r = modcheck::check(&built.dir, &mut modcheck::Discs::new(Some(iso_dir())));
        assert!(r.ok(), "{}: {:?}", pack.id, r.fail);
        assert!(r.note.iter().any(|n| n.starts_with(&format!("{} files compared byte for byte", count + 1))), "{:?}", r.note);
    }
    std::fs::remove_dir_all(&root).unwrap();
}
