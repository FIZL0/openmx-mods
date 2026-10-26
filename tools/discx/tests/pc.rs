//! MX vs. ATV Unleashed's PC install read as a source beside its disc (T12).
//!
//! The PC install is never in git: it is found by `discx::pc_install` (`$DISCX_PC`, else openmx's
//! `context/mxvatv-pc/data`), the disc by boot ELF in `$DISCX_ISO`, else `iso/` at the repo root. A
//! test whose source is not here says so on stderr and passes. The paks' tables are read in the gate;
//! comparing the two sources whole is ignored (`cargo test -p discx --release -- --ignored`).

use discx::{Disc, Source, compare, pak, pc_install, resolve_image};
use std::path::PathBuf;

fn iso_dir() -> PathBuf {
    std::env::var_os("DISCX_ISO")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../iso"))
}

fn pc() -> Option<PathBuf> {
    let p = pc_install();
    if p.is_none() {
        eprintln!("skipped: no MX vs. ATV Unleashed PC install ($DISCX_PC, or openmx/context/mxvatv-pc/data)");
    }
    p
}

fn disc() -> Option<PathBuf> {
    match resolve_image("mxvatv", &iso_dir()) {
        Ok(p) => Some(p),
        Err(e) => {
            eprintln!("skipped: {e}");
            None
        }
    }
}

/// The 70 paks open as one directory of 20,415 members, no name in two of them, every pak's last
/// member ending at its end (`tools/mxvatv_pak.py check`; research/mxvatv-pc.md's member count, though
/// it says 63 paks). The path may be `data/`, the install above it, one pak, or paks joined as `$PATH`
/// is.
#[test]
fn the_paks_open_as_one_directory() {
    let Some(data) = pc() else { return };
    let d = Disc::open_any(&data).unwrap();
    assert_eq!(d.source, Source::Paks);
    assert_eq!(d.game.map(|g| g.key), Some("mxvatv"));
    assert_eq!(d.boot, None);
    assert_eq!(d.root.len(), 70);
    assert_eq!(d.entries().len(), 20_415);
    assert!(d.entries().iter().all(|e| !e.name.to_ascii_lowercase().starts_with("data\\")));

    let install = data.parent().unwrap();
    if pak::is_paks(install) {
        assert_eq!(Disc::open_any(install).unwrap().entries().len(), 20_415);
    }
    let nat = data.join("Nat01.pak");
    let one = Disc::open_any(&nat).unwrap();
    assert_eq!(one.entries().len(), 104);
    let two = std::env::join_paths([nat, data.join("Env.pak")]).unwrap();
    let mut two = Disc::open_any(two.as_ref()).unwrap();
    assert_eq!(two.entries().len(), 104 + 30);
    // A member of the second pak is read from the second file.
    let last = two.entries().last().cloned().unwrap();
    assert_eq!(two.read(&last).unwrap().len(), last.size as usize);
}

/// The disc's files against the paks', by name, every PSR member and every member of every `.res`
/// against the loose file of its path. The counts are the ones notes/mxvatv.md (*The PC install*)
/// gives, so a change on either side shows here first.
#[test]
#[ignore = "needs the MX vs. ATV Unleashed disc image and its PC install"]
fn the_disc_against_the_paks() {
    let (Some(image), Some(data)) = (disc(), pc()) else { return };
    let mut a = Disc::open(&image).unwrap();
    let mut b = Disc::open_any(&data).unwrap();
    let c = compare(&mut a, &mut b, "**", false).unwrap();
    assert_eq!((c.same.len(), c.differ.len(), c.only_a.len(), c.only_b.len()), (6239, 256, 13_896, 13_920));
    let r = compare(&mut a, &mut b, "**", true).unwrap();
    assert_eq!((r.same.len(), r.differ.len(), r.only_a.len(), r.only_b.len()), (24_011, 761, 29_263, 13_919));
    // The PC has no .res at all.
    assert!(b.matching("**.res").is_empty());
}
