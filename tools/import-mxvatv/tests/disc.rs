//! A disc built here, with one MX vs. ATV-style map on it, imported and read back; then each thing the
//! importer must refuse. Ignored by default: every map on the user's MX vs. ATV Unleashed disc
//! imported and read back, and the MX Unleashed scene keys read again off its disc.
//!
//! The map: a track archive (a scene with MX vs. ATV's own keys, a terrain with texture pages between
//! its blocks and its tree, a mesh `.col`, a class-1 `.col`, a sky that does not decode) and a
//! material archive sharing the sky; loose beside them a copy of the scene, a race file and an
//! `.atyList`; loose in `Col\` a shape the scene places that neither archive has.

use import_mxvatv::{free_ride, import, load_map, map_folders, mxu_keys::MXU_SCENE_KEYS, scene_key_set};
use modcheck::{Discs, check, json::Json};
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::process::Command;

// ---- the disc's containers, as modcheck's tests build them (tools/modcheck/tests/disc.rs) ----

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

// ---- the map's files, as modcheck's tests build them ----

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

const BOOT: &str = "SLUS_211.04";

/// A scene in MX vs. ATV's style: `TerrainFile` without the platform tag, keys no MX Unleashed
/// scene has (`EcoAmbientScale`, a prop's `Activity` and `Hide`), and four collision placements.
fn scene() -> Vec<u8> {
    "[SceneInfo]\r\nSceneName=Test\r\nDefaultStartPosition=100.0,50.0,100.0\r\nDefaultStartDirection=0.0,0.0,1.0\r\n\r\n\
     [Environment]\r\nEcoAmbientScale=1.0,1.0,1.0\r\nTerrainFile=Test.trn\r\nTerrainWidth=1\r\nTerrainBreadth=1\r\n\
     TerrainWidthScale=9.000000\r\nTerrainBreadthScale=9.000000\r\nTerrainBorderWidth=0.000000\r\nMinDivisionSize=96.000000\r\n\
     CubeFile=sky1.cub\r\nDetailMapTilingMultiplier=8\r\n\r\n\
     [Model1]\r\nSLT=Rock\r\nPosition=1.0,2.0,3.0\r\n\r\n\
     [Model2]\r\nSLT=Type_National\r\nPosition=1.0,2.0,3.0\r\n;-- Extra params\r\nType=StartZone\r\nActivity=\"Free Ride\"\r\nHide=True\r\n\r\n\
     [Object1]\r\nCOL=Rock.col\r\nPosition=1.0,2.0,3.0\r\n\r\n\
     [Object2]\r\nCOL=rock.col\r\nPosition=4.0,2.0,3.0\r\n\r\n\
     [Object3]\r\nCOL=Blade.col\r\nPosition=5.0,2.0,3.0\r\n\r\n\
     [Object4]\r\nCOL=Loose.col\r\nPosition=6.0,2.0,3.0\r\n"
        .as_bytes()
        .to_vec()
}

const ATYLIST: &str = "; Available Activities\r\n\r\n[General]\r\nDisplayName = \"Test Map\"\r\nActivity = \"Test Race\"\r\nActivity = \"Free Ride\"\r\n\r\n\
     [Test Race]\r\nType    = \"national\"\r\nStartPosition = 1.0,2.0,3.0\r\nStartDirection = 0.0,0.0,1.0\r\n\r\n\
     [Free Ride]\r\nType   = \"free ride\"\r\nStartPosition = 2465.528320,401.135345,1255.091919\r\nStartDirection = 0.984868,0.000000,0.173304\r\n\
     ATYFile = Test.aty\r\nAllowRadar = FALSE\r\n";

struct Disc {
    root: PathBuf,
    iso: PathBuf,
    image: PathBuf,
    track: Vec<(&'static str, Vec<u8>)>,
    mat: Vec<(&'static str, Vec<u8>)>,
}

/// The map's archives as they are given, on a disc with this boot ELF, in its own folder.
fn disc_with(tag: &str, boot: &str, track: Vec<(&'static str, Vec<u8>)>, mat: Vec<(&'static str, Vec<u8>)>, atylist: &str) -> Disc {
    let root = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(format!("import-mxvatv-{tag}"));
    let _ = std::fs::remove_dir_all(&root);
    let iso = root.join("iso");
    std::fs::create_dir_all(&iso).unwrap();
    let mut psr = vec![
        (r"Env\Freeride.aty", bytes(3, 100)),
        (r"Env\Test\test.res", build_res(&track)),
        (r"Env\Test\test_mat.res", build_res(&mat)),
        (r"Env\Test\Test.scn", scene()),
        (r"Env\Test\Test.tdf", bytes(4, 300)),
        (r"Col\Loose.col", mesh_col(5)),
    ];
    if !atylist.is_empty() {
        psr.push((r"Env\Test\Test.atyList", atylist.as_bytes().to_vec()));
    }
    let cnf = format!("BOOT2 = cdrom0:\\{boot};1\r\nVER = 1.02\r\n").into_bytes();
    let img = build_iso(&[(boot, bytes(6, 3000)), ("SYSTEM.CNF", cnf), ("MXVSATV.PSR", build_psr(&psr))]);
    let image = iso.join("disc.iso");
    std::fs::write(&image, img).unwrap();
    Disc { root, iso, image, track, mat }
}

fn track() -> Vec<(&'static str, Vec<u8>)> {
    vec![
        (r"Env\test\Test.scn", scene()),
        (r"Env\test\Test.ps2.trn", terrain(1, 2)),
        (r"Env\test\Test.aud", bytes(10, 64)),
        (r"Col\Rock.col", mesh_col(1)),
        (r"Col\Blade.col", class1_col()),
        (r"Skies\sky1.ps2.cub", bytes(8, 900)),
        (r"Part\Testparticles.prt", bytes(11, 40)),
    ]
}

fn mat() -> Vec<(&'static str, Vec<u8>)> {
    vec![(r"Tex\rock.pst", bytes(9, 5000)), (r"Skies\sky1.ps2.cub", bytes(8, 900))]
}

fn the_disc(tag: &str) -> Disc {
    disc_with(tag, BOOT, track(), mat(), ATYLIST)
}

/// Import the disc's one map into `<root>/stage`.
fn import_disc(d: &Disc) -> Result<import_mxvatv::Imported, String> {
    let mut disc = discx::Disc::open(&d.image)?;
    assert_eq!(map_folders(&disc), vec!["Test".to_string()]);
    let map = load_map(&mut disc, "Test")?;
    import(&map, &d.root.join("stage"))
}

fn todo_has(m: &import_mxvatv::Imported, all: &[&str]) {
    assert!(
        m.todo.iter().any(|t| all.iter().all(|w| t.contains(w))),
        "no todo entry has all of {all:?}: {:#?}",
        m.todo
    );
}

// ---- the tests ----

#[test]
fn a_map_imports_as_the_disc_has_it_and_reads_back() {
    let d = the_disc("ok");
    let m = import_disc(&d).unwrap();
    assert_eq!(m.id, "mxvatv_test");
    assert_eq!(m.name, "Test Map");
    assert_eq!(m.dir, d.root.join("stage").join("mxvatv_test"));

    // Every member at its own path, byte for byte; the shared sky once; nothing loose copied.
    for (name, data) in d.track.iter().chain(&d.mat) {
        let have = std::fs::read(m.dir.join(name.replace('\\', "/"))).unwrap();
        assert!(&have == data, "{name} differs");
    }
    assert_eq!(m.files, d.track.len() + d.mat.len() - 1);
    assert!(!m.dir.join("Env/Test/Test.tdf").exists() && !m.dir.join("Col/Loose.col").exists());

    // mod.json: the standard's fields, the spawn from the .atyList, the lookups.
    let j = Json::parse(&std::fs::read_to_string(m.dir.join("mod.json")).unwrap()).unwrap();
    let s = |k: &str| j.get(k).and_then(Json::str).map(str::to_owned);
    assert_eq!(s("id").as_deref(), Some("mxvatv_test"));
    assert_eq!(s("kind").as_deref(), Some("track"));
    assert_eq!(s("name").as_deref(), Some("Test Map"));
    assert_eq!(s("permission").as_deref(), Some("own-disc"));
    let src = j.get("source").unwrap();
    assert_eq!(src.get("boot").and_then(Json::str), Some(BOOT));
    let members: Vec<&str> = src.get("members").and_then(Json::arr).unwrap().iter().filter_map(Json::str).collect();
    assert_eq!(members, [r"Env\Test\test.res", r"Env\Test\test_mat.res"]);
    let sp = j.get("spawn").unwrap();
    let nums = |k: &str| sp.get(k).and_then(Json::arr).unwrap().iter().map(|x| x.num().unwrap()).collect::<Vec<_>>();
    assert_eq!(nums("position"), [2465.528320, 401.135345, 1255.091919]);
    assert_eq!(nums("direction"), [0.984868, 0.0, 0.173304]);
    assert_eq!(sp.get("from").and_then(Json::str), Some(r"Env\Test\Test.atyList [Free Ride] StartPosition, StartDirection"));
    let l = j.get("lookups").unwrap();
    assert_eq!(l.get("terrain").and_then(Json::str), Some("Env/test/Test.ps2.trn"));
    assert_eq!(l.get("sky").and_then(Json::str), Some("Skies/sky1.ps2.cub"));

    // The todo: each gap, by file and reason.
    todo_has(&m, &["Env/test/Test.ps2.trn", "openmx MD1", "2 texture pages"]);
    todo_has(&m, &["Col/Blade.col", "class 1", "openmx MD5"]);
    todo_has(&m, &["Loose.col", "none of the map's archives", r"loose on the PSR as Col\Loose.col"]);
    todo_has(&m, &["Test.scn: Activity= in 1 [Model<n>] sections", "\"Free Ride\" ×1", "no MX Unleashed scene has this key"]);
    todo_has(&m, &["Test.scn: Hide= in 1 [Model<n>]", "draws every one of these props"]);
    todo_has(&m, &["Test.scn: EcoAmbientScale= in 1 [Environment]"]);
    todo_has(&m, &["Skies/sky1.ps2.cub: sky::Sky::parse"]);
    todo_has(&m, &["Tex/rock.pst: pst::Texture::parse"]);
    todo_has(&m, &["Rock.psg (1 placed", "Type_National.psg"]);
    todo_has(&m, &["Env/test/Test.aud", "openmx MD2"]);
    todo_has(&m, &["*.prt (1 files: Part/Testparticles.prt)"]);
    todo_has(&m, &["ATYFile=Test.aty"]);
    todo_has(&m, &["AllowRadar=FALSE"]);
    todo_has(&m, &[r"Env\Test\: 1 loose members", "1 .tdf", "1 more are the same bytes"]);
    // What agrees with MX Unleashed, or with the terrain, is not a gap.
    for quiet in ["TerrainWidth", "DetailMapTilingMultiplier", "Rock.col", "DefaultStartPosition", "SLT="] {
        assert!(!m.todo.iter().any(|t| t.contains(quiet)), "{quiet} is in todo: {:#?}", m.todo);
    }

    // Read back by modcheck against the same disc (§5).
    let r = check(&m.dir, &mut Discs::new(Some(d.iso.clone())));
    assert!(r.ok(), "{:#?}", r.fail);
}

#[test]
fn a_scene_that_disagrees_with_its_terrain_is_kept_and_named() {
    let mut t = track();
    let wide = String::from_utf8(scene()).unwrap().replace("TerrainWidth=1", "TerrainWidth=2").replace("TerrainBreadthScale=9.000000", "TerrainBreadthScale=8.000000");
    t[0].1 = wide.clone().into_bytes();
    let d = disc_with("disagree", BOOT, t, mat(), ATYLIST);
    let m = import_disc(&d).unwrap();
    assert_eq!(std::fs::read(m.dir.join("Env/test/Test.scn")).unwrap(), wide.as_bytes(), "the scene is never edited");
    todo_has(&m, &["Test.scn: TerrainWidth=2 on the disc", "256 samples"]);
    todo_has(&m, &["Test.scn: TerrainBreadthScale=8 on the disc", "9 ft"]);
    let r = check(&m.dir, &mut Discs::new(Some(d.iso.clone())));
    assert!(r.ok(), "{:#?}", r.fail);
}

#[test]
fn two_archives_that_disagree_on_a_path_stop_the_import() {
    let mut m = mat();
    m[1].1 = bytes(99, 900);
    let d = disc_with("differ", BOOT, track(), m, ATYLIST);
    let e = import_disc(&d).unwrap_err();
    assert!(e.contains("Skies/sky1.ps2.cub") && e.contains("differ"), "{e}");
    assert!(!d.root.join("stage/mxvatv_test").exists(), "a failed import leaves no folder");
}

#[test]
fn one_path_spelt_two_ways_stops_the_import() {
    let mut m = mat();
    m[1].0 = r"Skies\SKY1.ps2.cub";
    let d = disc_with("spelt", BOOT, track(), m, ATYLIST);
    let e = import_disc(&d).unwrap_err();
    assert!(e.contains("spelt two ways"), "{e}");
}

#[test]
fn a_map_without_one_scene_does_not_import() {
    let mut t = track();
    t.remove(0);
    let e = import_disc(&disc_with("noscene", BOOT, t, mat(), ATYLIST)).unwrap_err();
    assert!(e.contains("no .scn"), "{e}");
    let mut t = track();
    t.push((r"Env\test\Other.scn", scene()));
    let e = import_disc(&disc_with("twoscenes", BOOT, t, mat(), ATYLIST)).unwrap_err();
    assert!(e.contains("2 .scn files"), "{e}");
}

#[test]
fn without_a_start_the_scene_spawns_and_todo_says_so() {
    let commented = ATYLIST
        .replace("StartPosition = 2465", ";StartPosition = 2465")
        .replace("StartDirection = 0.98", ";StartDirection = 0.98");
    let d = disc_with("nostart", BOOT, track(), mat(), &commented);
    let m = import_disc(&d).unwrap();
    assert!(m.spawn.is_none());
    let j = std::fs::read_to_string(m.dir.join("mod.json")).unwrap();
    assert!(!j.contains("\"spawn\""), "{j}");
    todo_has(&m, &["Test.scn: the spawn is the scene's DefaultStartPosition (100.0,50.0,100.0)", "T7"]);
    // No .atyList at all: the folder's name, and todo says so.
    let d = disc_with("noaty", BOOT, track(), mat(), "");
    let m = import_disc(&d).unwrap();
    assert_eq!(m.name, "Test");
    todo_has(&m, &["no .atyList"]);
    assert!(check(&m.dir, &mut Discs::new(Some(d.iso.clone()))).ok());
}

#[test]
fn a_free_ride_start_is_read_as_written() {
    let f = free_ride("x.atyList", b"[General]\nActivity=\"Free Ride\"\n[Free Ride]\nType=\"free ride\"\nStartPosition=120.00,90.00f,-804.5\nStartDirection=0.58,-0.01,-0.82\n").unwrap();
    let (p, d, from) = f.spawn.unwrap();
    assert_eq!(p, "[120.00, 90.00, -804.5]");
    assert_eq!(d, "[0.58, -0.01, -0.82]");
    assert_eq!(from, "x.atyList [Free Ride] StartPosition, StartDirection");
    for bad in ["1.0,2.0", "1.0,two,3.0", ".5,1,2", "01,1,2", "1e3,1,2"] {
        let text = format!("[General]\nActivity=\"Free Ride\"\n[Free Ride]\nType=\"free ride\"\nStartPosition={bad}\nStartDirection=0,0,1\n");
        assert!(free_ride("x", text.as_bytes()).is_err(), "{bad} was taken");
    }
    // A Free Ride the [General] list does not name is not one.
    let f = free_ride("x", b"[General]\nActivity=\"Race\"\n[Race]\nType=\"national\"\n[Free Ride]\nType=\"free ride\"\n").unwrap();
    assert!(f.spawn.is_none() && f.todo.iter().any(|t| t.contains("no activity of Type")));
}

#[test]
fn the_scene_key_set_is_by_section_kind() {
    let k = scene_key_set("pre=1\n[Model12]\nSLT=a\nHide=True\n[ Model3 ]\nslt=b\n[Environment]\nCubeFile=x\n");
    let want: BTreeSet<(String, String)> =
        [("model", "slt"), ("model", "hide"), ("environment", "cubefile")].iter().map(|(a, b)| (a.to_string(), b.to_string())).collect();
    assert_eq!(k, want);
    assert!(MXU_SCENE_KEYS.contains(&("model", "slt")) && !MXU_SCENE_KEYS.contains(&("model", "hide")));
}

#[test]
fn the_binary_imports_a_disc_and_skips_another_game() {
    let bin = env!("CARGO_BIN_EXE_import-mxvatv");
    let d = the_disc("bin");
    let out = d.root.join("bin-stage");
    let o = Command::new(bin).arg("--disc").arg(&d.image).arg("--out").arg(&out).output().unwrap();
    let text = String::from_utf8_lossy(&o.stdout);
    assert!(o.status.success(), "{text}{}", String::from_utf8_lossy(&o.stderr));
    assert!(text.contains("ok   mxvatv_test \"Test Map\"") && text.contains("read back") && text.contains("1 maps imported, 0 failed"), "{text}");
    assert!(out.join("mxvatv_test/mod.json").is_file());

    let o = Command::new(bin).arg("--disc").arg(&d.image).arg("--out").arg(&out).args(["--map", "nat99"]).output().unwrap();
    assert!(!o.status.success());

    let u = disc_with("untamed", "SLUS_217.01", track(), mat(), ATYLIST);
    let uout = u.root.join("stage");
    let o = Command::new(bin).arg("--disc").arg(&u.image).arg("--out").arg(&uout).output().unwrap();
    assert!(o.status.success() && String::from_utf8_lossy(&o.stdout).contains("nothing imported"));
    assert!(std::fs::read_dir(&uout).map_or(true, |mut r| r.next().is_none()));
}

// ---- loose PSR members the game opens for a map (M15) ----

/// A version-1 `.esb` with these species and no plants: each a model name and a billboard.
fn esb(models: &[&str]) -> Vec<u8> {
    let mut e = 1u32.to_le_bytes().to_vec();
    for slot in 0..256 {
        let Some(m) = models.get(slot) else {
            e.push(0);
            continue;
        };
        e.push(1);
        for name in [format!("{m}\0"), format!("{m}BB.tga\0")] {
            e.push(name.len() as u8);
            e.extend_from_slice(name.as_bytes());
        }
        for f in [10.0f32, 8.0, 12.0, 2.0, 1.0, 3.0, 0.0, 1.0, 0.0, 1.0, 0.5] {
            e.extend_from_slice(&f.to_le_bytes());
        }
        e.extend_from_slice(&[0u8; 12]);
        e.extend_from_slice(&0u32.to_le_bytes()); // no solids
    }
    e.push(0); // no shrubbery
    e.extend_from_slice(&0u32.to_le_bytes()); // no plants
    e
}

/// A `.psm` with one shader pass naming these textures.
fn psm(textures: &[&str]) -> Vec<u8> {
    let mut t = format!("[General]\r\nTotalShaderPasses = 1\r\n[ShaderPass_1]\r\ntotalTextureMaps = {}\r\n", textures.len());
    for (n, x) in textures.iter().enumerate() {
        t += &format!("TextureMap_{} = {x}\r\n", n + 1);
    }
    t.into_bytes()
}

/// [`disc_with`]'s disc plus a `_mst.res` and more loose members.
fn disc_loose(
    tag: &str,
    track: Vec<(&'static str, Vec<u8>)>,
    mat: Vec<(&'static str, Vec<u8>)>,
    mst: Vec<(&'static str, Vec<u8>)>,
    loose: Vec<(&'static str, Vec<u8>)>,
) -> Disc {
    let root = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(format!("import-mxvatv-{tag}"));
    let _ = std::fs::remove_dir_all(&root);
    let iso = root.join("iso");
    std::fs::create_dir_all(&iso).unwrap();
    let mut psr = vec![
        (r"Env\Freeride.aty", bytes(3, 100)),
        (r"Env\Test\test.res", build_res(&track)),
        (r"Env\Test\test_mat.res", build_res(&mat)),
        (r"Env\Test\test_mst.res", build_res(&mst)),
        (r"Env\Test\Test.atyList", ATYLIST.as_bytes().to_vec()),
    ];
    psr.extend(loose);
    let cnf = format!("BOOT2 = cdrom0:\\{BOOT};1\r\nVER = 1.02\r\n").into_bytes();
    let img = build_iso(&[(BOOT, bytes(6, 3000)), ("SYSTEM.CNF", cnf), ("MXVSATV.PSR", build_psr(&psr))]);
    let image = iso.join("disc.iso");
    std::fs::write(&image, img).unwrap();
    Disc { root, iso, image, track, mat }
}

#[test]
fn loose_members_the_game_opens_are_copied_and_listed() {
    // The scene names its ecosystem without `.esb`, which the game adds.
    let eco_scene = String::from_utf8(scene()).unwrap().replace("CubeFile=sky1.cub", "CubeFile=sky1.cub\r\nEcosystemFile=test");
    let mut t = track();
    t[0].1 = eco_scene.into_bytes();
    t.push((r"Env\test\test.esb", esb(&["Tree", "Shrub"])));
    t.push((r"Eco\Shrub.psg", bytes(20, 300)));
    let mut m = mat();
    m.push((r"Tex\Held.psm", psm(&["Held"])));
    m.push((r"Tex\Held.pst", bytes(21, 400)));
    let table = "#include \"MST\\Global.mst\"\r\n; the map's materials\r\nTex\\Held\r\nTex\\Loose ; a note\r\nTex\\Bare\r\nTex\\Nowhere\r\n";
    let mst = vec![(r"MST\Env\Test.mst", table.as_bytes().to_vec())];
    let loose = vec![
        (r"Eco\Tree.psg", bytes(22, 300)),           // a species' model the archives lack
        (r"Eco\Shrub.psg", bytes(23, 300)),          // the archive's is opened, not this
        (r"Tex\Loose.psm", psm(&["LooseTex", "RenderTarget"])),
        (r"Tex\LooseTex.pss", bytes(24, 200)),       // .pst first, then .pss: this one
        (r"Tex\LooseTex.tga", bytes(25, 200)),       // never reached
        (r"Tex\Bare.tga", bytes(26, 200)),           // a material with no .psm is its own texture
        (r"Tex\Unasked.pst", bytes(27, 200)),        // nothing names it
        (r"MST\Global.mst", b"Tex\\Shared\r\n".to_vec()), // the shared table, not the map's
    ];
    let want = [r"Eco\Tree.psg", r"Tex\Loose.psm", r"Tex\LooseTex.pss", r"Tex\Bare.tga"];
    let d = disc_loose("loose", t, m, mst, loose.clone());
    let got = import_disc(&d).unwrap();
    assert_eq!(got.pulled, want);
    for (name, data) in &loose {
        let path = got.dir.join(name.replace('\\', "/"));
        if want.contains(name) {
            assert_eq!(&std::fs::read(&path).unwrap(), data, "{name}");
        } else if !name.starts_with(r"Eco\Shrub") {
            assert!(!path.exists(), "{name} was copied");
        }
    }
    assert_eq!(std::fs::read(got.dir.join("Eco/Shrub.psg")).unwrap(), bytes(20, 300), "the archive's member wins");

    let j = Json::parse(&std::fs::read_to_string(got.dir.join("mod.json")).unwrap()).unwrap();
    let members: Vec<&str> = j.get("source").and_then(|s| s.get("members")).and_then(Json::arr).unwrap().iter().filter_map(Json::str).collect();
    let mut all = vec![r"Env\Test\test.res", r"Env\Test\test_mat.res", r"Env\Test\test_mst.res"];
    all.extend(want);
    assert_eq!(members, all);
    let why = j.get("loose").unwrap();
    assert_eq!(why.get(r"Eco\Tree.psg").and_then(Json::str), Some("species Tree of env/test/test.esb"));
    assert_eq!(why.get(r"Tex\LooseTex.pss").and_then(Json::str), Some(r"texture Tex\LooseTex of material Tex\Loose in mst/env/test.mst"));
    // The model the ecosystem names is found now, so todo no longer says it is missing.
    assert!(!got.todo.iter().any(|t| t.contains("tree.psg (")), "{:#?}", got.todo);

    let r = check(&got.dir, &mut Discs::new(Some(d.iso.clone())));
    assert!(r.ok(), "{:#?}", r.fail);
}

// ---- against the user's discs (`cargo test -p import-mxvatv -- --ignored`) ----

/// The image with this boot ELF in `$DISCX_ISO`, or else this repo's `iso/`.
fn real_disc(boot: &str) -> (PathBuf, PathBuf) {
    let dir = std::env::var_os("DISCX_ISO")
        .map(PathBuf::from)
        .unwrap_or_else(|| Path::new(env!("CARGO_MANIFEST_DIR")).join("../../iso"));
    let (image, _) = discx::images(&dir)
        .unwrap_or_else(|e| panic!("{e}: set DISCX_ISO to the folder with the disc images"))
        .into_iter()
        .find(|(_, b)| b.as_deref().is_some_and(|b| b.eq_ignore_ascii_case(boot)))
        .unwrap_or_else(|| panic!("no disc with boot ELF {boot} in {}", dir.display()));
    (dir, image)
}

#[test]
#[ignore = "needs the MX vs. ATV Unleashed disc (SLUS_211.04) in iso/ or $DISCX_ISO"]
fn every_freeride_map_imports_and_reads_back() {
    let (dir, image) = real_disc(BOOT);
    let mut disc = discx::Disc::open(&image).unwrap();
    let out = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("import-mxvatv-disc");
    let _ = std::fs::remove_dir_all(&out);
    let mut discs = Discs::new(Some(dir));
    let folders = map_folders(&disc);
    let mut bad = Vec::new();
    let mut freeride = 0;
    for f in &folders {
        let map = load_map(&mut disc, f).unwrap();
        // A Free Ride map is one whose .atyList offers an activity of that type (M8).
        let Some((member, bytes)) = &map.atylist else {
            bad.push(format!("{f}: no .atyList"));
            continue;
        };
        if free_ride(member, bytes).unwrap().todo.iter().any(|t| t.contains("no activity of Type")) {
            continue;
        }
        freeride += 1;
        match import(&map, &out) {
            Err(e) => bad.push(format!("{f}: {e}")),
            Ok(m) => {
                let r = check(&m.dir, &mut discs);
                if !r.ok() {
                    bad.push(format!("{}: {:?}", m.id, r.fail));
                }
                std::fs::remove_dir_all(&m.dir).unwrap();
            }
        }
    }
    assert!(bad.is_empty(), "{bad:#?}");
    // notes/mxvatv.md (M8): all 56 maps offer a Free Ride.
    assert_eq!((folders.len(), freeride), (56, 56));
}

#[test]
#[ignore = "needs the MX Unleashed disc (SLUS_207.67) in iso/ or $DISCX_ISO"]
fn mxu_scene_keys_are_the_discs() {
    let (_, image) = real_disc("SLUS_207.67");
    let mut disc = discx::Disc::open(&image).unwrap();
    let mut have = BTreeSet::new();
    let mut scenes = 0;
    for e in disc.matching(r"Tracks\*.res").into_iter().cloned().collect::<Vec<_>>() {
        let res = disc.res(&e).unwrap();
        for s in res.matching("**.scn").into_iter().cloned().collect::<Vec<_>>() {
            have.extend(scene_key_set(&game_render::ini::text(&res.read(&s).unwrap())));
            scenes += 1;
        }
    }
    let listed: BTreeSet<(String, String)> = MXU_SCENE_KEYS.iter().map(|(a, b)| (a.to_string(), b.to_string())).collect();
    let print: Vec<String> = have.iter().map(|(a, b)| format!("    (\"{a}\", \"{b}\"),")).collect();
    assert_eq!(have, listed, "src/mxu_keys.rs should be:\n{}", print.join("\n"));
    assert_eq!(scenes, 57);
}

#[test]
#[ignore = "needs the MX vs. ATV Unleashed disc (SLUS_211.04) in iso/ or $DISCX_ISO"]
fn the_loose_members_the_maps_lack_are_pulled() {
    let (_, image) = real_disc(BOOT);
    let mut disc = discx::Disc::open(&image).unwrap();
    let pulled = |disc: &mut discx::Disc, f: &str| -> Vec<String> {
        load_map(disc, f).unwrap().pulled.iter().map(|p| p.member.to_ascii_lowercase()).collect()
    };
    // Nat02's ten plants, loose only in Eco\ (notes/mxvatv.md, M9).
    let nat02 = pulled(&mut disc, "Nat02");
    for m in ["busha", "bushb", "bushc", "bushf", "bushg", "treea", "treeb", "treec", "treed", "treee"] {
        assert!(nat02.contains(&format!(r"eco\nat02_{m}.psg")), "Nat02 lacks {m}: {nat02:?}");
    }
    // The maps that borrow another map's plants, and its two eco materials with them.
    for (f, from) in [("MC03", "nat11"), ("MC04", "nat03"), ("MC05", "nat11"), ("MC09", "nat11"), ("MC10", "nat15"), ("Nat13", "nat11"), ("SM01", "nat11")] {
        let p = pulled(&mut disc, f);
        for m in [format!(r"tex\{from}bb.psm"), format!(r"tex\{from}eco.psm")] {
            assert!(p.contains(&m), "{f} lacks {m}: {p:?}");
        }
    }
    let sm03 = pulled(&mut disc, "SM03");
    assert!(sm03.contains(&r"tex\nat17eco.psm".to_string()), "{sm03:?}");
}
