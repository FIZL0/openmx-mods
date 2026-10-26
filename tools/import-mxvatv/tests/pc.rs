//! MX vs. ATV Unleashed's maps against its PC paks (T12): what a map import reads off the disc,
//! looked for in the paks, and every map refused off the paks with the reason.
//!
//! The disc is found by boot ELF in `$DISCX_ISO`, else `iso/` at the repo root; the PC install by
//! `discx::pc_install` (`$DISCX_PC`, else openmx's `context/mxvatv-pc/data`). Without one, a test
//! says so on stderr and passes. The disc's side is ignored (`cargo test -p import-mxvatv --release
//! -- --ignored`).

use import_mxvatv::{MXVATV, load_map, map_folders, pc_refusal};
use std::collections::BTreeMap;
use std::path::PathBuf;

fn iso_dir() -> PathBuf {
    std::env::var_os("DISCX_ISO")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../iso"))
}

fn pc() -> Option<PathBuf> {
    let p = discx::pc_install();
    if p.is_none() {
        eprintln!("skipped: no MX vs. ATV Unleashed PC install ($DISCX_PC, or openmx/context/mxvatv-pc/data)");
    }
    p
}

/// The paks' 64 map folders (a `<MAP>.scn` in `Env\<MAP>\`), each refused with the reason and
/// nothing read: the paks hold no `.res`.
#[test]
fn every_map_off_the_paks_is_refused_with_the_reason() {
    let Some(data) = pc() else { return };
    let mut paks = discx::Disc::open_any(&data).unwrap();
    let folders = map_folders(&paks);
    assert_eq!(folders.len(), 64);
    for f in &folders {
        let err = load_map(&mut paks, f).err().unwrap_or_else(|| panic!("{f} loaded off the paks"));
        assert_eq!(err, pc_refusal(f, &MXVATV));
    }
}

/// Every file an import of each of the disc's 56 maps reads (its archives' members, its
/// `.atyList`, the loose members it pulls) looked for in the paks at the same path: the counts
/// notes/mxvatv.md (*The PC install*) gives. The paks have every map the disc has, and eight more.
#[test]
#[ignore = "needs the MX vs. ATV Unleashed disc image and its PC install"]
fn what_a_map_import_reads_against_the_paks() {
    let Some(data) = pc() else { return };
    let image = match discx::resolve_image("mxvatv", &iso_dir()) {
        Ok(p) => p,
        Err(e) => return eprintln!("skipped: {e}"),
    };
    let mut disc = discx::Disc::open(&image).unwrap();
    let mut paks = discx::Disc::open_any(&data).unwrap();
    let ours = map_folders(&disc);
    assert_eq!(ours.len(), 56);
    let lower = |v: &[String]| v.iter().map(|f| f.to_ascii_lowercase()).collect::<Vec<_>>();
    let theirs = lower(&map_folders(&paks));
    let extra: Vec<&String> = theirs.iter().filter(|f| !lower(&ours).contains(f)).collect();
    assert_eq!(extra, ["fs09", "fs10", "fs11", "fs12", "fs13", "nat19", "sx19", "sx20"]);

    // By extension: (same, differ, absent), counting a file once per map that reads it.
    let mut by_ext: BTreeMap<String, (usize, usize, usize)> = BTreeMap::new();
    let mut differ_once: std::collections::BTreeSet<String> = Default::default();
    let mut render_same: std::collections::BTreeSet<String> = Default::default();
    for f in &ours {
        let m = load_map(&mut disc, f).unwrap();
        let mut read: Vec<(String, Vec<u8>)> = Vec::new();
        for (_, r) in &m.archives {
            for e in r.entries() {
                read.push((e.name.clone(), r.read(e).unwrap()));
            }
        }
        if let Some((n, b)) = &m.atylist {
            read.push((n.clone(), b.clone()));
        }
        for p in &m.pulled {
            read.push((p.member.clone(), p.bytes.clone()));
        }
        for (name, bytes) in read {
            let lower = name.to_ascii_lowercase();
            let ext = if lower.ends_with(".ps2.trn") { "ps2.trn".to_string() } else { lower.rsplit('.').next().unwrap().to_string() };
            let slot = by_ext.entry(ext.clone()).or_default();
            match paks.find(&name).cloned() {
                None => slot.2 += 1,
                Some(e) if paks.read(&e).unwrap() == bytes => {
                    slot.0 += 1;
                    if ["psg", "psm", "pst", "ps2.trn", "cub", "atm"].contains(&ext.as_str()) {
                        render_same.insert(name.clone());
                    }
                }
                Some(_) => {
                    slot.1 += 1;
                    differ_once.insert(discx::normalise(&name));
                }
            }
        }
    }
    let total = by_ext.values().fold((0, 0, 0), |t, v| (t.0 + v.0, t.1 + v.1, t.2 + v.2));
    for (ext, v) in &by_ext {
        eprintln!("{ext}\t{v:?}");
    }
    eprintln!("total\t{total:?}, {} distinct files differ: {differ_once:?}", differ_once.len());
    assert_eq!(total, (TOTAL.0, TOTAL.1, TOTAL.2));
    // Of what the PS2 renders with, the paks have these three as the disc has them, and nothing else.
    eprintln!("{render_same:?}");
    assert_eq!(render_same.into_iter().collect::<Vec<_>>(), RENDER_SAME);
}

/// notes/mxvatv.md, *The PC install*: (same, differ, absent).
const TOTAL: (usize, usize, usize) = (15_799, 342, 13_676);
/// The PS2's render files the paks hold too, left behind in the PC build, byte for byte.
const RENDER_SAME: [&str; 3] = [r"Env\nat10\anim\nat10_millwheel.psg", r"Env\nat10\anim\rusticwindmill.psg", r"Env\nat12\Nat12.ps2.trn"];
