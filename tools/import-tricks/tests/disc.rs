//! Each game's stunt table read back off its disc, and both trick packs built, read back and proved
//! by modcheck byte for byte against the disc.
//!
//! `cargo test -p import-tricks --release -- --ignored`. The images are found by boot ELF in
//! `$DISCX_ISO`, else `iso/` at the repo root (a worktree has none: set `DISCX_ISO`).

use import_tricks::*;
use std::path::PathBuf;

fn iso_dir() -> PathBuf {
    std::env::var_os("DISCX_ISO")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../iso"))
}

/// A table off the disc, as its pack's loader reads it, and the bytes it was read from.
fn table(pack: &Pack, member: &str) -> (Table, Vec<u8>) {
    let image = discx::resolve_image(pack.game, &iso_dir()).unwrap();
    let mut disc = discx::Disc::open(&image).unwrap();
    let e = disc.find(member).cloned().unwrap_or_else(|| panic!("{member}"));
    let bytes = disc.read(&e).unwrap();
    (Table::parse(&ini_text(member, &bytes).unwrap(), pack).unwrap(), bytes)
}

/// The loose tables are the ones the loader opens: each equals its member of the pack's `StuntInfo.res`.
#[test]
#[ignore = "needs the MX vs. ATV Unleashed and Untamed disc images"]
fn the_loose_tables_are_the_archives() {
    for pack in &PACKS {
        let image = discx::resolve_image(pack.game, &iso_dir()).unwrap();
        let mut disc = discx::Disc::open(&image).unwrap();
        let res_entry = disc.find(pack.res).cloned().unwrap();
        let res = disc.res(&res_entry).unwrap();
        assert_eq!(res.entries().len(), pack.tables.len(), "{}: {:?}", pack.res, res.entries());
        for &t in pack.tables {
            let inner = res.entries().iter().find(|e| e.name.eq_ignore_ascii_case(t)).cloned().unwrap_or_else(|| panic!("{t}"));
            let loose = disc.find(t).cloned().unwrap();
            assert_eq!(res.read(&inner).unwrap(), disc.read(&loose).unwrap(), "{t}");
        }
    }
}

#[test]
#[ignore = "needs the MX vs. ATV Unleashed disc image"]
fn mxvatv_tables_read_back() {
    let pack = Pack::by_game("mxvatv").unwrap();
    let (mx, _) = table(pack, pack.tables[0]);
    assert_eq!(mx.stunts.len(), 46);
    assert!(mx.unread.is_empty(), "{:?}", mx.unread);
    let names: Vec<&str> = mx.stunts.iter().map(|s| s.name.as_str()).collect();
    assert_eq!(&names[..4], ["Taunt", "lazyboy", "cancan", "nothing"]);
    assert_eq!(&names[41..], ["backflip", "wheelie", "stoppie", "threesixtyL", "threesixtyR"]);

    // The 24 gestures: six button sets by four directions, every one different, none R1 alone
    // (that is the backflip's and the 360's), and none past the 24th stunt.
    let mut seen = std::collections::BTreeSet::new();
    for (i, s) in mx.stunts.iter().enumerate() {
        match (&s.key, i < 24) {
            (Some(k), true) => {
                let b = key_bits(k);
                assert!(seen.insert(b), "{} repeats {k}", s.name);
                assert_ne!(b & 0x0f, R1, "{}: R1 alone", s.name);
                assert_eq!((b & 0xf0).count_ones(), 1, "{}: one direction", s.name);
            }
            (None, false) => {}
            (k, _) => panic!("{} (#{}) has gesture {k:?}", s.name, i + 1),
        }
    }
    let groups: std::collections::BTreeSet<u8> = seen.iter().map(|b| b & 0x0f).collect();
    assert_eq!(groups.into_iter().collect::<Vec<_>>(), [CIRCLE, TRIANGLE, CIRCLE | TRIANGLE, R1 | CIRCLE, R1 | TRIANGLE, R1 | CIRCLE | TRIANGLE]);

    let n = |v: &str| game_sim::phy::number(v).0;
    let l = mx.by_name("threesixtyL").unwrap();
    assert_eq!((l.hold_time, l.safe_time, l.duration), (n("0.72"), n("1.26"), n("1.50")));
    assert_eq!((l.point_value, l.hold_point_value, l.used_for_bike), (2000.0, 15000.0, Some(true)));
    let b = mx.by_name("backflip").unwrap();
    assert_eq!((b.hold_time, b.safe_time, b.duration, b.hold_point_value), (n("0.60"), n("0.80"), n("1.00"), 10000.0));
    let w = mx.by_name("wheelie").unwrap();
    assert_eq!((w.duration, w.has_animation, w.point_value_given, w.hold_point_value), (0.0, Some(false), false, 1000.0));

    // MXF's is the MX table but for the backflip's HoldTime; ATV's but for saranwrap's SafeTime,
    // and it adds [Depreciation].
    let differ = |a: &Table, b: &Table| -> Vec<String> {
        a.stunts.iter().zip(&b.stunts).filter(|(x, y)| x != y).map(|(x, _)| x.name.clone()).collect()
    };
    let (mxf, _) = table(pack, pack.tables[1]);
    assert_eq!(differ(&mxf, &mx), ["backflip"]);
    assert_eq!(mxf.by_name("backflip").unwrap().hold_time, n("0.20"));
    let (atv, _) = table(pack, pack.tables[2]);
    assert_eq!(differ(&atv, &mx), ["saranwrap"]);
    assert_eq!(atv.by_name("saranwrap").unwrap().safe_time, n("1.33"));
    let d = atv.depreciation.unwrap();
    assert_eq!((d.pct, d.restore_one, d.max_depletion), (n("0.8"), 4, 4));

    // What openmx's own loader makes of it: 57 sections wanted, 46 there.
    let (_, bytes) = table(pack, pack.tables[0]);
    assert!(openmx_reads(&String::from_utf8_lossy(&bytes)).unwrap_err().contains("[stunt47]"));
}

#[test]
#[ignore = "needs the MX vs. ATV Unleashed and Untamed disc images"]
fn untamed_table_reads_back() {
    let pack = Pack::by_game("untamed").unwrap();
    let (u, bytes) = table(pack, pack.tables[0]);
    assert_eq!(u.stunts.len(), 54);
    let mxvatv = Pack::by_game("mxvatv").unwrap();
    let (mx, _) = table(mxvatv, mxvatv.tables[0]);
    // The first 41 are MX vs. ATV Unleashed's, the same names and gestures (the Classic mapping),
    // the clocks one ulp apart at most: the text is truncated to a float by the loader, BPB holds
    // the nearest.
    for (a, b) in u.stunts[..41].iter().zip(&mx.stunts[..41]) {
        assert_eq!((&a.name, &a.display_name, &a.key), (&b.name, &b.display_name, &b.key));
        for (x, y) in [(a.hold_time, b.hold_time), (a.safe_time, b.safe_time), (a.duration, b.duration)] {
            assert!(x.to_bits().abs_diff(y.to_bits()) <= 1, "{}: {x} against {y}", a.name);
        }
    }
    // Then eight repeats (rows for the Wii's extra gestures), then the same last five.
    for s in &u.stunts[41..49] {
        assert!(mx.stunts[..41].iter().any(|m| m.name == s.name), "{} is a repeat", s.name);
        assert_eq!(s.key, None);
    }
    let tail: Vec<&str> = u.stunts[49..].iter().map(|s| s.name.as_str()).collect();
    assert_eq!(tail, ["backflip", "wheelie", "stoppie", "threesixtyL", "threesixtyR"]);
    let text = ini_text(pack.tables[0], &bytes).unwrap();
    assert!(openmx_reads(&text).is_err());
}

#[test]
#[ignore = "needs the MX Unleashed disc image"]
fn mx_unleashed_has_57_and_lacks_five() {
    let names = mxu_names_from(&iso_dir()).unwrap();
    assert_eq!(names.len(), 57);
    let mxvatv = Pack::by_game("mxvatv").unwrap();
    let (mx, _) = table(mxvatv, mxvatv.tables[0]);
    let new: Vec<&str> =
        mx.stunts.iter().map(|s| s.name.as_str()).filter(|n| !names.iter().any(|m| m.eq_ignore_ascii_case(n))).collect();
    assert_eq!(new, ["Varial", "wheelie", "stoppie", "threesixtyL", "threesixtyR"]);
}

#[test]
#[ignore = "needs all three disc images"]
fn both_packs_build_read_back_and_prove() {
    let root = std::env::temp_dir().join(format!("import-tricks-disc-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    for pack in &PACKS {
        let image = discx::resolve_image(pack.game, &iso_dir()).unwrap();
        let built = import(&image, pack, &iso_dir(), &root).unwrap();
        assert_eq!(built.new, ["Varial", "wheelie", "stoppie", "threesixtyL", "threesixtyR"], "{}", pack.id);

        // Every clip the stunts play is in the pack, and openmx reads exactly the version-3 ones.
        for c in &built.clips {
            let bytes = std::fs::read(built.dir.join(file(&c.member))).unwrap();
            let version = -i32::from_le_bytes(bytes[..4].try_into().unwrap());
            assert_eq!(c.read.is_ok(), version == 3, "{} is version {version}: {:?}", c.member, c.read);
            assert!(matches!(version, 3 | 6 | 7), "{}: version {version}", c.member);
        }
        for r in pack.riders {
            for n in ["ThreeSixtyL", "ThreeSixtyR", "Varial"] {
                let want = format!("Rider\\Anim\\{}_{n}.mot", r.prefix);
                assert!(built.clips.iter().any(|c| c.member.eq_ignore_ascii_case(&want)), "{}: {want}", pack.id);
                let bike = format!("Veh\\Anim\\{}bike_{n}.mot", r.bike);
                assert!(built.clips.iter().any(|c| c.member.eq_ignore_ascii_case(&bike)), "{}: {bike}", pack.id);
            }
        }

        // Every packed .phy tunes the 360.
        let phys: Vec<String> = built.members.iter().filter(|m| m.to_ascii_lowercase().contains(".phy")).cloned().collect();
        assert!(phys.len() >= 24, "{}: {} .phy", pack.id, phys.len());
        for m in &phys {
            let text = ini_text(m, &std::fs::read(built.dir.join(file(m))).unwrap()).unwrap();
            let secs = sections(&text);
            let (_, keys) = secs.iter().find(|(t, _)| t == "Three Sixty").unwrap_or_else(|| panic!("{m}: no [Three Sixty]"));
            assert_eq!(keys.len(), 8, "{m}");
            assert!(secs.iter().any(|(t, _)| t == "Backflip"), "{m}");
        }

        let (tables, clips) = check(&built.dir).unwrap();
        for (a, b) in tables.iter().zip(&built.tables) {
            assert_eq!(a.table, b.table, "{}", a.member);
        }
        assert_eq!(clips.len(), built.clips.len());

        let r = modcheck::check(&built.dir, &mut modcheck::Discs::new(Some(iso_dir())));
        assert!(r.ok(), "{}: {:?}", pack.id, r.fail);
        let compared = format!("{} files compared byte for byte", built.members.len());
        assert!(r.note.iter().any(|n| n.starts_with(&compared)), "{:?}", r.note);
    }
    std::fs::remove_dir_all(&root).unwrap();
}
