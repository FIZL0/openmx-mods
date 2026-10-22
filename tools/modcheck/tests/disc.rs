//! A disc built here, with one MX vs. ATV-style map on it: a scene, a terrain with texture pages
//! between its blocks and its tree (as every MX vs. ATV terrain has), a mesh `.col`, a class-1 `.col`
//! the reader refuses, and a material archive. The mod packaged from it must pass `modcheck`, and
//! each thing the standard forbids, done to a copy of it, must fail.

use game_sim::track::Terrain;
use modcheck::{Discs, Report, check, terrain::md1_view};
use std::path::{Path, PathBuf};
use std::process::Command;

// ---- the disc's containers, as discx's own tests build them (tools/discx/tests/image.rs) ----

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

// ---- the map's files ----

const BLOCK_BYTES: usize = 2129;
const NODE_BYTES: usize = 97;

fn node(height_scale: f32, block: u16, children_at: u32) -> Vec<u8> {
    let mut n = Vec::new();
    for f in [1.0f32, height_scale, 0.0, 100.0] {
        n.extend_from_slice(&f.to_le_bytes());
    }
    n.extend_from_slice(&0u16.to_le_bytes());
    n.extend_from_slice(&block.to_le_bytes());
    n.extend_from_slice(&[0u8; 4 + 64 + 4 + 1]);
    n.extend_from_slice(&children_at.to_le_bytes());
    assert_eq!(n.len(), NODE_BYTES);
    n
}

/// A PLN v7 terrain of two levels over `cells` × `cells` root cells (256 samples each, 9 ft a sample),
/// every leaf sharing one block, as the disc shares identical ones. With `pages`, that many one-mip
/// 16 × 16 texture pages sit between the blocks and the tree, with their offsets in the header.
fn terrain(cells: u32, pages: usize) -> Vec<u8> {
    let mut h = Vec::new();
    h.extend_from_slice(b"PLN\x007ps2\0");
    let mut name = [0u8; 32];
    name[..4].copy_from_slice(b"Test");
    h.extend_from_slice(&name);
    h.extend_from_slice(&0u32.to_le_bytes()); // no palette
    let fixed_at = h.len();
    h.extend_from_slice(&[0u8; 20]); // scale, levels, tree, declared, blocks: filled in below
    h.extend_from_slice(&0u32.to_le_bytes()); // the one declared word
    h.extend_from_slice(&(pages as u32).to_le_bytes());
    h.extend_from_slice(&(pages as u32).to_le_bytes());
    let words_at = h.len();
    h.extend(std::iter::repeat_n(0u8, pages * 4));
    let mut block = bytes(7, BLOCK_BYTES);
    block[0..4].copy_from_slice(&0i32.to_le_bytes());
    h.extend_from_slice(&block);
    for p in 0..pages {
        let at = h.len() as u32;
        h[words_at + p * 4..][..4].copy_from_slice(&at.to_le_bytes());
        for w in [0x27u32, 1, 16, 16, 256, 0] {
            h.extend_from_slice(&w.to_le_bytes());
        }
        h.extend(bytes(100 + p as u32, 256));
    }
    let tree = h.len() as u32;
    // root, its 256-word table, one middle node per root cell with its table, one leaf node.
    let table = 256 * 4;
    let root_table = tree + NODE_BYTES as u32;
    let mid0 = root_table + table as u32;
    let mids = cells * cells;
    let leaf = mid0 + mids * (NODE_BYTES + table) as u32;
    h.extend(node(1.0, 0, root_table));
    let mut t = vec![0u8; table];
    for cz in 0..cells {
        for cx in 0..cells {
            let k = cz * cells + cx;
            let at = mid0 + k * (NODE_BYTES + table) as u32;
            t[((cx + cz * 16) * 4) as usize..][..4].copy_from_slice(&at.to_le_bytes());
        }
    }
    h.extend(t);
    for k in 0..mids {
        let at = mid0 + k * (NODE_BYTES + table) as u32;
        h.extend(node(1.0, 0, at + NODE_BYTES as u32));
        for _ in 0..256 {
            h.extend_from_slice(&leaf.to_le_bytes());
        }
    }
    h.extend(node(1.0, 0, 0));
    let mut f = Vec::new();
    for w in [9.0f32.to_bits(), 2, tree, 1, 1] {
        f.extend_from_slice(&w.to_le_bytes());
    }
    h[fixed_at..fixed_at + 20].copy_from_slice(&f);
    h
}

/// A one-triangle collision mesh, version 2.
fn mesh_col(seed: u32) -> Vec<u8> {
    let mut c = Vec::new();
    for w in [1234u32] {
        c.extend_from_slice(&w.to_le_bytes());
    }
    c.extend_from_slice(b"col\0");
    for w in [2u32, 0, seed] {
        c.extend_from_slice(&w.to_le_bytes());
    }
    for f in [1.0f32, 0., 0., 0., 1., 0., 0., 0., 1., 0., 0., 0., 0.] {
        c.extend_from_slice(&f.to_le_bytes());
    }
    for w in [3u32, 0, 1, 0] {
        c.extend_from_slice(&w.to_le_bytes());
    }
    for f in [0.0f32, 0., 0., 1., 0., 0., 0., 0., 1.] {
        c.extend_from_slice(&f.to_le_bytes());
    }
    c.extend_from_slice(&0u32.to_le_bytes()); // no normals
    for f in [0.0f32, 0., 1., 0.] {
        c.extend_from_slice(&f.to_le_bytes());
    }
    for i in [0u16, 1, 2] {
        c.extend_from_slice(&i.to_le_bytes());
    }
    c
}

/// A class-1 `.col`, which openmx refuses (MD5).
fn class1_col() -> Vec<u8> {
    let mut c = 1234u32.to_le_bytes().to_vec();
    c.extend_from_slice(b"col\0");
    for w in [2u32, 1, 0, 0, 0] {
        c.extend_from_slice(&w.to_le_bytes());
    }
    c
}

fn scene(width: u32) -> Vec<u8> {
    format!(
        "[SceneInfo]\r\nSceneName=Test\r\nDefaultStartPosition=100.0,50.0,100.0\r\nDefaultStartDirection=0.0,0.0,1.0\r\n\r\n\
         [Environment]\r\nTerrainFile=Test.trn\r\nTerrainWidth={width}\r\nTerrainBreadth=1\r\n\
         TerrainWidthScale=9.000000\r\nTerrainBreadthScale=9.000000\r\nTerrainBorderWidth=0.000000\r\nMinDivisionSize=96.000000\r\n\
         CubeFile=sky1.cub\r\n\r\n\
         [Model1]\r\nSLT=Rock\r\nPosition=1.0,2.0,3.0\r\n\r\n\
         [Object1]\r\nCOL=Rock.col\r\nPosition=1.0,2.0,3.0\r\n\r\n\
         [Object2]\r\nCOL=rock.col\r\nPosition=4.0,2.0,3.0\r\n\r\n\
         [Object3]\r\nCOL=Blade.col\r\nPosition=5.0,2.0,3.0\r\n"
    )
    .into_bytes()
}

struct Map {
    root: PathBuf,
    iso: PathBuf,
    /// The two archives' members, in the order an importer would copy them.
    members: Vec<(&'static str, Vec<u8>)>,
}

const BOOT: &str = "SLUS_211.04";
const MEMBERS: &str = r#"["Env\\Test\\test.res", "Env\\Test\\test_mat.res"]"#;

/// The disc, written to its own folder (`iso/disc.iso`), with a scene of this width.
fn map(tag: &str, width: u32) -> Map {
    let root = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(format!("modcheck-{tag}"));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(root.join("iso")).unwrap();
    let track = vec![
        (r"Env\test\Test.scn", scene(width)),
        (r"Env\test\Test.ps2.trn", terrain(1, 2)),
        (r"Col\Rock.col", mesh_col(1)),
        (r"Col\Blade.col", class1_col()),
        (r"Skies\sky1.ps2.cub", bytes(8, 900)),
    ];
    let mat = vec![(r"Tex\rock.pst", bytes(9, 5000)), (r"Skies\sky1.ps2.cub", bytes(8, 900))];
    let psr = vec![
        (r"Env\Freeride.aty", bytes(3, 100)),
        (r"Env\Test\test.res", build_res(&track)),
        (r"Env\Test\test_mat.res", build_res(&mat)),
    ];
    let cnf = format!("BOOT2 = cdrom0:\\{BOOT};1\r\nVER = 1.02\r\n").into_bytes();
    let img = build_iso(&[(BOOT, bytes(6, 3000)), ("SYSTEM.CNF", cnf), ("MXVSATV.PSR", build_psr(&psr))]);
    let iso = root.join("iso");
    std::fs::write(iso.join("disc.iso"), img).unwrap();
    let mut members = track;
    members.extend(mat);
    Map { root, iso, members }
}

/// Package the map as an importer would: every member at its path, and a mod.json with this `todo`.
fn package(m: &Map, id: &str, todo: &str) -> PathBuf {
    let dir = m.root.join("mods").join("tracks").join(id);
    let _ = std::fs::remove_dir_all(&dir);
    for (name, data) in &m.members {
        let p = dir.join(name.replace('\\', "/"));
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(p, data).unwrap();
    }
    write_json(&dir, id, MEMBERS, todo, "");
    dir
}

fn write_json(dir: &Path, id: &str, members: &str, todo: &str, extra: &str) {
    let j = format!(
        r#"{{"standard": 1, "kind": "track", "id": "{id}", "name": "Test map",
  "source": {{"game": "MX vs. ATV Unleashed", "boot": "{BOOT}", "members": {members}}},
  "version": "1.0.0", "author": "modcheck tests", "permission": "own-disc", "todo": {todo}{extra}}}"#
    );
    std::fs::write(dir.join("mod.json"), j).unwrap();
}

const TODO: &str = r#"["Col\\Blade.col: collision class 1, refused (openmx MD5)"]"#;

fn run(m: &Map, dir: &Path) -> Report {
    check(dir, &mut Discs::new(Some(m.iso.clone())))
}

fn fails_with(r: &Report, what: &str) {
    assert!(!r.ok(), "passed, but should fail with {what:?}: {:#?}", r.note);
    assert!(r.fail.iter().any(|f| f.contains(what)), "no fail mentions {what:?}: {:#?}", r.fail);
}

// ---- the tests ----

#[test]
fn the_md1_view_reads_a_paged_terrain_exactly() {
    let paged = terrain(2, 3);
    let refusal = Terrain::parse(&paged).unwrap_err();
    assert!(refusal.contains("but the tree starts at"), "{refusal}");
    let view = md1_view(&paged).unwrap();
    assert_eq!((view.pages, view.gap), (3, 3 * (8 + 16 + 256)));
    assert_eq!(view.bytes.len(), paged.len());
    let t = Terrain::parse(&view.bytes).unwrap();
    let plain_bytes = terrain(2, 0);
    let plain = Terrain::parse(&plain_bytes).unwrap();
    assert_eq!((t.levels, t.blocks, t.scale), (plain.levels, plain.blocks, plain.scale));
    // The same ground everywhere, read through the view and from the file without pages.
    for (sx, sz) in [(0, 0), (5, 9), (255, 3), (300, 500), (511, 511)] {
        let (a, b) = (t.cell(sx, sz, 1), plain.cell(sx, sz, 1));
        assert!(a.is_some() || sx % 16 == 15, "no cell at {sx},{sz}");
        assert_eq!(a, b, "cell {sx},{sz}");
    }
    // The tree after the pages is untouched.
    let tree = t.root_at() as usize;
    assert_eq!(&view.bytes[tree..], &paged[tree..]);
    // A plain terrain needs no view, and a gap that is not the header's pages gets none.
    assert!(md1_view(&plain_bytes).is_err());
    let mut wrong = paged.clone();
    let words_at = 4 + 1 + 4 + 32 + 4 + 20 + 4 + 8;
    wrong[words_at..words_at + 4].copy_from_slice(&7u32.to_le_bytes());
    assert!(md1_view(&wrong).unwrap_err().contains("offset word"));
}

#[test]
fn a_packaged_map_passes() {
    let m = map("ok", 1);
    let dir = package(&m, "mxvatv_test", TODO);
    let r = run(&m, &dir);
    assert!(r.ok(), "{:#?}", r.fail);
    let notes = r.note.join("\n");
    assert!(notes.contains("read through the MD1 view: 2 texture pages"), "{notes}");
    assert!(notes.contains("1 × 1 root cells, 2 levels, 256 × 256 samples at 9 ft"), "{notes}");
    assert!(notes.contains("2 shapes: 1 read, 1 listed in todo"), "{notes}");
    // Seven members, one path in both archives with the same bytes: six files.
    assert!(notes.contains("7 files compared byte for byte"), "{notes}");
}

#[test]
fn a_copy_that_differs_from_the_disc_fails() {
    let m = map("differs", 1);
    let dir = package(&m, "mxvatv_test", TODO);
    let p = dir.join("Tex/rock.pst");
    let mut b = std::fs::read(&p).unwrap();
    b[4000] ^= 1;
    std::fs::write(&p, b).unwrap();
    fails_with(&run(&m, &dir), "Tex/rock.pst: differs from Env\\Test\\test_mat.res:Tex\\rock.pst");
    std::fs::remove_file(&p).unwrap();
    fails_with(&run(&m, &dir), "Tex/rock.pst: missing");
}

#[test]
fn a_file_no_member_accounts_for_fails_unless_todo_names_it() {
    let m = map("extra", 1);
    let dir = package(&m, "mxvatv_test", TODO);
    std::fs::write(dir.join("Env/test/Test.scn.txt"), b"x").unwrap();
    fails_with(&run(&m, &dir), "Env/test/Test.scn.txt");
    write_json(&dir, "mxvatv_test", MEMBERS, r#"["Col\\Blade.col: class 1", "Test.scn.txt: decoded by M10"]"#, "");
    assert!(run(&m, &dir).ok());
}

#[test]
fn paths_that_differ_only_in_case_fail() {
    let m = map("case", 1);
    let dir = package(&m, "mxvatv_test", TODO);
    std::fs::copy(dir.join("Col/Rock.col"), dir.join("Col/rock.col")).unwrap();
    let r = run(&m, &dir);
    fails_with(&r, "Col/Rock.col and Col/rock.col differ only in case");
    fails_with(&r, "Rock.col finds 2 files");
}

#[test]
fn a_refused_or_missing_shape_must_be_in_todo() {
    let m = map("col", 1);
    let dir = package(&m, "mxvatv_test", r#"["scratch"]"#);
    fails_with(&run(&m, &dir), "Col/Blade.col: Collision::parse: collision class 1 is not understood, and todo does not name it");
    let dir = package(&m, "mxvatv_test", TODO);
    std::fs::remove_file(dir.join("Col/Rock.col")).unwrap();
    fails_with(&run(&m, &dir), "Rock.col: placed by the scene, not in the folder, and todo does not name it");
}

#[test]
fn a_scene_that_disagrees_with_its_terrain_fails_unless_todo_says_so() {
    let m = map("width", 2);
    let dir = package(&m, "mxvatv_test", TODO);
    fails_with(&run(&m, &dir), "TerrainWidth=2 is 512 samples, but Env/test/Test.ps2.trn has 256");
    write_json(
        &dir,
        "mxvatv_test",
        MEMBERS,
        r#"["Col\\Blade.col: class 1", "Test.scn: TerrainWidth=2 on the disc, the terrain is 1 root cell wide"]"#,
        "",
    );
    let r = run(&m, &dir);
    assert!(r.ok(), "{:#?}", r.fail);
}

#[test]
fn mod_json_is_held_to_the_standard() {
    let m = map("json", 1);
    let dir = package(&m, "mxvatv_test", TODO);
    let cases: &[(&str, &str, &str, &str)] = &[
        ("mxvatv_other", MEMBERS, TODO, "is not the folder's name"),
        ("mxvatv_test", MEMBERS, "[]", "todo is [] but T7 has not checked it"),
        ("mxvatv_test", "[]", TODO, "non-empty source.members"),
        ("mxvatv_test", r#"["Env\\Test\\nope.res"]"#, TODO, "not on the SLUS_211.04 disc"),
    ];
    for (id, members, todo, want) in cases {
        write_json(&dir, id, members, todo, "");
        fails_with(&run(&m, &dir), want);
    }
    write_json(&dir, "mxvatv_test", MEMBERS, TODO, r#", "spawn": {"position": [1, 2, 3], "direction": [0, 0, 1]}"#);
    fails_with(&run(&m, &dir), "a spawn with no source is a guess");
    write_json(&dir, "mxvatv_test", MEMBERS, TODO, r#", "spawn": {"position": [1, 2, 3], "direction": [0, 0, 1], "from": "Test.atyList [Freeride]"}"#);
    assert!(run(&m, &dir).ok());

    // A wrong prefix, and an MX Unleashed track that is not a world's tile.
    let other = m.root.join("mods/tracks/untamed_test");
    std::fs::rename(&dir, &other).unwrap();
    write_json(&other, "untamed_test", MEMBERS, TODO, "");
    fails_with(&run(&m, &other), "does not start with \"mxvatv_\"");
    std::fs::write(other.join("mod.json"), "{\"standard\": 1,").unwrap();
    fails_with(&run(&m, &other), "mod.json does not parse");
}

#[test]
fn without_its_disc_a_copy_cannot_be_proved() {
    let m = map("nodisc", 1);
    let dir = package(&m, "mxvatv_test", TODO);
    let r = check(&dir, &mut Discs::new(Some(m.root.join("mods"))));
    fails_with(&r, "cannot prove the copy: no disc with boot ELF SLUS_211.04");
}

#[test]
fn the_binary_checks_a_tree_of_mods() {
    let m = map("bin", 1);
    package(&m, "mxvatv_test", TODO);
    let bad = package(&m, "mxvatv_bad", TODO);
    std::fs::remove_file(bad.join("Env/test/Test.ps2.trn")).unwrap();
    let out = Command::new(env!("CARGO_BIN_EXE_modcheck"))
        .args(["--iso", m.iso.to_str().unwrap(), m.root.join("mods").to_str().unwrap()])
        .output()
        .unwrap();
    let text = String::from_utf8_lossy(&out.stdout);
    assert!(!out.status.success(), "{text}");
    assert!(text.contains("TerrainFile=Test.trn finds no file (tried Test.trn, Test.ps2.trn)"), "{text}");
    assert!(text.contains("modcheck: 1 ok, 1 failed"), "{text}");
}
