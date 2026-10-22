//! A disc built here, byte by byte: an ISO 9660 image whose root holds SYSTEM.CNF and a small PSR,
//! and in the PSR a `.res` beside loose members. Listing and extracting it, through the library and
//! the `discx` binary, must give back exactly the bytes that went in.

use discx::{Disc, Game, ResFile, glob_match, res_member_path};
use std::path::{Path, PathBuf};
use std::process::Command;

const SECTOR: usize = 2048;
const BLOCK: usize = 24576;

/// Distinct, non-repeating-looking bytes, so a shifted or truncated copy cannot compare equal.
fn bytes(seed: u32, len: usize) -> Vec<u8> {
    let mut x = seed.wrapping_mul(2654435761).wrapping_add(1);
    (0..len)
        .map(|_| {
            x ^= x << 13;
            x ^= x >> 17;
            x ^= x << 5;
            x as u8
        })
        .collect()
}

fn adler32(d: &[u8]) -> u32 {
    let (mut a, mut b) = (1u32, 0u32);
    for &c in d {
        a = (a + c as u32) % 65521;
        b = (b + a) % 65521;
    }
    (b << 16) | a
}

/// One `.res` block: a zlib stream of one stored deflate block, the one kind writable by hand.
fn zlib_stored(data: &[u8]) -> Vec<u8> {
    let len = data.len() as u16;
    let mut z = vec![0x78, 0x01, 0x01];
    z.extend_from_slice(&len.to_le_bytes());
    z.extend_from_slice(&(!len).to_le_bytes());
    z.extend_from_slice(data);
    z.extend_from_slice(&adler32(data).to_be_bytes());
    z
}

/// A `.res` in the layout `game_sim::archive::Res::parse` documents: each member starts on a block,
/// the stream is padded to whole blocks, each block is followed by 0xff.
fn build_res(members: &[(&str, Vec<u8>)]) -> Vec<u8> {
    let mut stream = Vec::new();
    let mut dir = Vec::new();
    for (name, data) in members {
        let at = stream.len();
        dir.extend_from_slice(&(name.len() as u32).to_le_bytes());
        dir.extend_from_slice(name.as_bytes());
        dir.push(0);
        dir.extend_from_slice(&((at / 256) as u32).to_le_bytes()[..3]);
        dir.extend_from_slice(&(data.len() as u32).to_le_bytes());
        stream.extend_from_slice(data);
        stream.resize(stream.len().div_ceil(BLOCK) * BLOCK, 0);
    }
    let blocks: Vec<Vec<u8>> = stream.chunks(BLOCK).map(zlib_stored).collect();
    let table_bytes = (blocks.len() + 1) * 2;
    let mut f = Vec::new();
    for w in [3, table_bytes as u32 + 12, 832, 0, table_bytes as u32] {
        f.extend_from_slice(&w.to_le_bytes());
    }
    for b in &blocks {
        f.extend_from_slice(&(b.len() as u16).to_le_bytes());
    }
    f.extend_from_slice(&0u16.to_le_bytes());
    f.extend_from_slice(&(members.len() as u32).to_le_bytes());
    f.extend_from_slice(&(dir.len() as u32).to_le_bytes());
    f.extend_from_slice(&dir);
    for b in &blocks {
        f.extend_from_slice(b);
        f.push(0xff);
    }
    f
}

/// A PSR in the layout `Psr::parse` documents, with a `<PAD>` entry it must drop.
fn build_psr(members: &[(&str, Vec<u8>)]) -> Vec<u8> {
    let mut dir = Vec::new();
    let mut data = Vec::new();
    let entry = |dir: &mut Vec<u8>, name: &str, size: u32, sector: u32| {
        dir.extend_from_slice(&(name.len() as u32).to_le_bytes());
        dir.extend_from_slice(name.as_bytes());
        dir.extend_from_slice(&size.to_le_bytes());
        dir.extend_from_slice(&sector.to_le_bytes());
    };
    // Members start after the directory; one sector each of front is plenty for this test.
    let first = 2usize;
    for (k, (name, bytes)) in members.iter().enumerate() {
        if k == 1 {
            entry(&mut dir, "<PAD>", 0, 0);
        }
        let sector = first + data.len() / SECTOR;
        entry(&mut dir, name, bytes.len() as u32, sector as u32);
        data.extend_from_slice(bytes);
        data.resize(data.len().div_ceil(SECTOR) * SECTOR, 0);
    }
    let mut f = Vec::new();
    for w in [2, members.len() as u32 + 1, 0, 0, dir.len() as u32, 0] {
        f.extend_from_slice(&w.to_le_bytes());
    }
    f.extend_from_slice(&dir);
    assert!(f.len() <= first * SECTOR, "the test's PSR directory outgrew its front");
    f.resize(first * SECTOR, 0);
    f.extend_from_slice(&data);
    f
}

fn both32(v: u32) -> [u8; 8] {
    let mut b = [0u8; 8];
    b[..4].copy_from_slice(&v.to_le_bytes());
    b[4..].copy_from_slice(&v.to_be_bytes());
    b
}

fn record(name: &[u8], extent: u32, size: u32, dir: bool) -> Vec<u8> {
    let len = (33 + name.len()).next_multiple_of(2);
    let mut r = vec![0u8; len];
    r[0] = len as u8;
    r[2..10].copy_from_slice(&both32(extent));
    r[10..18].copy_from_slice(&both32(size));
    r[25] = if dir { 2 } else { 0 };
    r[28..32].copy_from_slice(&[1, 0, 0, 1]);
    r[32] = name.len() as u8;
    r[33..33 + name.len()].copy_from_slice(name);
    r
}

/// An ISO 9660 image with these files in its root. A dummy file comes first, so the root takes
/// more than one record and the files do not start where a careless reader might guess.
fn build_iso(files: &[(&str, Vec<u8>)]) -> Vec<u8> {
    let root_sector = 18u32;
    let mut next = root_sector + 1;
    let mut root = Vec::new();
    root.extend(record(&[0], root_sector, SECTOR as u32, true));
    root.extend(record(&[1], root_sector, SECTOR as u32, true));
    let mut placed = Vec::new();
    for (name, data) in files {
        root.extend(record(format!("{name};1").as_bytes(), next, data.len() as u32, false));
        placed.push((next, data));
        next += data.len().div_ceil(SECTOR).max(1) as u32;
    }
    assert!(root.len() <= SECTOR);
    let mut img = vec![0u8; next as usize * SECTOR];
    let pvd = &mut img[16 * SECTOR..17 * SECTOR];
    pvd[0] = 1;
    pvd[1..6].copy_from_slice(b"CD001");
    pvd[6] = 1;
    pvd[156..156 + 34].copy_from_slice(&record(&[0], root_sector, SECTOR as u32, true));
    let term = &mut img[17 * SECTOR..18 * SECTOR];
    term[0] = 255;
    term[1..6].copy_from_slice(b"CD001");
    term[6] = 1;
    img[root_sector as usize * SECTOR..][..root.len()].copy_from_slice(&root);
    for (sector, data) in placed {
        img[sector as usize * SECTOR..][..data.len()].copy_from_slice(data);
    }
    img
}

struct Fixture {
    dir: PathBuf,
    iso: PathBuf,
    psr: Vec<(&'static str, Vec<u8>)>,
    res: Vec<(&'static str, Vec<u8>)>,
}

/// A fresh directory under the target dir, per test, so tests can run in parallel.
fn fixture(tag: &str) -> Fixture {
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(format!("discx-{tag}"));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    // One member spanning two blocks, one on its own, one empty: the block arithmetic's edges.
    let res = vec![
        (r"Nat01.ps2.trn", bytes(1, BLOCK + 1000)),
        (r"Col\rock_a.col", bytes(2, 433)),
        (r"Col\empty.col", Vec::new()),
    ];
    let psr = vec![
        (r"Env\Freeride.aty", bytes(3, 100)),
        (r"Env\Nat01\nat01.res", build_res(&res)),
        (r"Env\Nat01\Nat01.scn", bytes(4, SECTOR * 2 + 1)),
        (r"UI\fonts.res", bytes(5, 10)),
    ];
    let cnf = b"BOOT2 = cdrom0:\\SLUS_211.04;1\r\nVER = 1.02\r\nVMODE = NTSC\r\n".to_vec();
    let iso_bytes = build_iso(&[
        ("SLUS_211.04", bytes(6, 3000)),
        ("SYSTEM.CNF", cnf),
        ("MXVSATV.PSR", build_psr(&psr)),
    ]);
    let iso = dir.join("disc.iso");
    std::fs::write(&iso, iso_bytes).unwrap();
    Fixture { dir, iso, psr, res }
}

fn discx(args: &[&str]) -> (String, String, bool) {
    let out = Command::new(env!("CARGO_BIN_EXE_discx"))
        .args(args)
        .output()
        .expect("discx runs");
    (
        String::from_utf8_lossy(&out.stdout).into_owned(),
        String::from_utf8_lossy(&out.stderr).into_owned(),
        out.status.success(),
    )
}

fn path(p: &Path) -> &str {
    p.to_str().unwrap()
}

#[test]
fn the_disc_is_known_by_its_boot_elf_and_its_psr_lists() {
    let f = fixture("lib");
    let mut d = Disc::open(&f.iso).expect("the built image opens");
    assert_eq!(d.boot.as_deref(), Some("SLUS_211.04"));
    assert_eq!(d.game, Game::by_boot("SLUS_211.04"));
    assert_eq!(d.psr_name, "MXVSATV.PSR");
    let names: Vec<&str> = d.entries().iter().map(|e| e.name.as_str()).collect();
    let want: Vec<&str> = f.psr.iter().map(|(n, _)| *n).collect();
    assert_eq!(names, want, "every member, in order, <PAD> dropped");

    let res: Vec<String> = d.matching(r"Env\*\*.res").iter().map(|e| e.name.clone()).collect();
    assert_eq!(res, [r"Env\Nat01\nat01.res"]);

    for (name, data) in &f.psr {
        let e = d.find(name).unwrap().clone();
        assert_eq!(&d.read(&e).unwrap(), data, "{name}");
        let mut copied = Vec::new();
        d.copy(&e, &mut copied).unwrap();
        assert_eq!(&copied, data, "{name}, streamed");
    }

    let e = d.find("env/nat01/NAT01.RES").unwrap().clone();
    let r = d.res(&e).unwrap();
    for (name, data) in &f.res {
        let m = r.entries().iter().find(|m| m.name == *name).unwrap();
        assert_eq!(&r.read(m).unwrap(), data, "{name} out of the .res");
    }
    let cols: Vec<&str> = r.matching(r"Col\*.col").iter().map(|m| m.name.as_str()).collect();
    assert_eq!(cols, [r"Col\rock_a.col", r"Col\empty.col"]);
    assert_eq!(
        res_member_path(&e, &r.entries()[1]),
        PathBuf::from("Env/Nat01/nat01.res.d/Col/rock_a.col")
    );
}

#[test]
fn ls_prints_members_and_res_members() {
    let f = fixture("ls");
    let (out, err, ok) = discx(&["ls", path(&f.iso), r"Env\*\*.res"]);
    assert!(ok, "{err}");
    assert_eq!(out, format!("{}\tEnv\\Nat01\\nat01.res\n", f.psr[1].1.len()));

    let (out, err, ok) = discx(&["ls", path(&f.iso), r"Env\*\*.res:Col\*"]);
    assert!(ok, "{err}");
    assert_eq!(
        out,
        "433\tEnv\\Nat01\\nat01.res:Col\\rock_a.col\n0\tEnv\\Nat01\\nat01.res:Col\\empty.col\n"
    );

    let (out, err, ok) = discx(&["ls", path(&f.iso)]);
    assert!(ok, "{err}");
    assert_eq!(out.lines().count(), f.psr.len());

    let (_, err, ok) = discx(&["ls", path(&f.iso), "nothing*"]);
    assert!(!ok && err.contains("matches"), "{err}");

    let (out, err, ok) = discx(&["info", path(&f.iso)]);
    assert!(ok, "{err}");
    assert!(out.contains("MX vs. ATV Unleashed") && out.contains("members\t4"), "{out}");
}

#[test]
fn x_extracts_byte_exact_and_reaches_into_res() {
    let f = fixture("x");
    let out = f.dir.join("out");
    let (_, err, ok) = discx(&["x", path(&f.iso), r"**", path(&out)]);
    assert!(ok, "{err}");
    for (name, data) in &f.psr {
        let got = std::fs::read(out.join(name.replace('\\', "/"))).unwrap();
        assert_eq!(&got, data, "{name}");
    }

    let (_, err, ok) = discx(&["x", path(&f.iso), r"env\*\*.res:**", path(&out)]);
    assert!(ok, "{err}");
    for (name, data) in &f.res {
        let p = out.join("Env/Nat01/nat01.res.d").join(name.replace('\\', "/"));
        assert_eq!(&std::fs::read(&p).unwrap(), data, "{}", p.display());
    }

    // And the extracted .res read loose, the way a later tool would.
    let loose = out.join("Env/Nat01/nat01.res");
    let (list, err, ok) = discx(&["res", path(&loose)]);
    assert!(ok, "{err}");
    assert_eq!(list.lines().count(), f.res.len());
    let again = f.dir.join("again");
    let (_, err, ok) = discx(&["res", path(&loose), "**", path(&again)]);
    assert!(ok, "{err}");
    for (name, data) in &f.res {
        let got = std::fs::read(again.join(name.replace('\\', "/"))).unwrap();
        assert_eq!(&got, data, "{name}, loose");
    }
    assert!(ResFile::open(&loose).is_ok());
}

#[test]
fn a_game_name_finds_its_disc_in_the_iso_folder() {
    let f = fixture("byname");
    let iso_dir = f.dir.join("iso");
    std::fs::create_dir_all(&iso_dir).unwrap();
    std::fs::rename(&f.iso, iso_dir.join("whatever it is called.iso")).unwrap();
    std::fs::write(iso_dir.join("not-a-disc.bin"), b"hello").unwrap();
    let out = Command::new(env!("CARGO_BIN_EXE_discx"))
        .args(["ls", "mxvatv", r"UI\*"])
        .env("DISCX_ISO", &iso_dir)
        .output()
        .unwrap();
    assert!(out.status.success(), "{}", String::from_utf8_lossy(&out.stderr));
    assert_eq!(String::from_utf8_lossy(&out.stdout), "10\tUI\\fonts.res\n");

    let out = Command::new(env!("CARGO_BIN_EXE_discx"))
        .args(["ls", "untamed"])
        .env("DISCX_ISO", &iso_dir)
        .output()
        .unwrap();
    assert!(!out.status.success());
    assert!(String::from_utf8_lossy(&out.stderr).contains("SLUS_217.01"));
}

#[test]
fn globs_fold_slashes_and_case() {
    assert!(glob_match(r"ENV/*/NAT01.*", r"Env\Nat01\nat01.res"));
}

/// The user's own discs, if `iso/` (or $DISCX_ISO) has them: each known one opens, its PSR parses,
/// and its first member reads. `cargo test -p discx -- --ignored`.
#[test]
#[ignore = "needs the user's discs in iso/"]
fn every_disc_in_iso_opens() {
    let dir = std::env::var_os("DISCX_ISO")
        .map(PathBuf::from)
        .unwrap_or_else(|| Path::new(env!("CARGO_MANIFEST_DIR")).join("../../iso"));
    let images = discx::images(&dir).expect("iso/ is readable");
    let mut known = 0;
    for (path, boot) in images {
        let Some(game) = boot.as_deref().and_then(Game::by_boot) else {
            eprintln!("skip {}: boot ELF {boot:?} is not one of ours", path.display());
            continue;
        };
        for psr in game.psrs {
            let mut d = Disc::open_psr(&path, Some(psr)).unwrap();
            let first = d.entries()[0].clone();
            let data = d.read(&first).unwrap();
            eprintln!(
                "{} {}: {} members, first {} ({} bytes)",
                game.boot,
                psr,
                d.entries().len(),
                first.name,
                data.len()
            );
        }
        known += 1;
    }
    assert!(known > 0, "no known disc in {}", dir.display());
}
