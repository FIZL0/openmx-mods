//! A disc built here in MX vs. ATV Untamed's layout (M4), with one map to import and one the master
//! activity list gives no Free Ride; the map imported and read back. Ignored by default: every map
//! chosen on the user's Untamed disc imported and read back, and every `.col` on that disc through
//! openmx's reader.
//!
//! The map: three archives under `res\env\Test\`, each with its directory padded by zero bytes as
//! the disc pads them; a binary scene, a binary material and a material table; and loose on the PSR
//! the members the game would open for it (a placed model, a placed shape, the sky, a material only
//! loose and its texture only as `.atm`) beside decoys it would not.

use import_mxvatv::{Map, import, untamed};
use modcheck::{Discs, check, json::Json};
use std::path::PathBuf;
use std::process::Command;

// ---- the disc's containers, as tests/disc.rs builds them ----

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

// ---- Untamed's own ----

const BOOT: &str = "SLUS_217.01";

/// A `.res` whose directory is followed by `pad` zero bytes that the header's directory length
/// counts, as Untamed's map archives are.
fn build_res_padded(members: &[(&str, Vec<u8>)], pad: usize) -> Vec<u8> {
    let plain = build_res(members);
    let table = u32::from_le_bytes(plain[0x10..0x14].try_into().unwrap()) as usize;
    let count_at = 0x14 + table;
    let dir_bytes = u32::from_le_bytes(plain[count_at + 4..count_at + 8].try_into().unwrap()) as usize;
    let end = count_at + 8 + dir_bytes;
    let mut f = plain[..end].to_vec();
    f[count_at + 4..count_at + 8].copy_from_slice(&((dir_bytes + pad) as u32).to_le_bytes());
    f.extend(std::iter::repeat_n(0u8, pad));
    f.extend_from_slice(&plain[end..]);
    f
}

/// INI text compiled to BPB with tools/bpb's encoder: a title's trailing digits are its index, as the
/// game splits them (`[Model1]` is `Model` 1), and every value is a string.
fn bpb_of(ini: &str) -> Vec<u8> {
    let mut b = bpb::Bpb::default();
    for line in ini.lines().map(str::trim).filter(|l| !l.is_empty() && !l.starts_with(';')) {
        if let Some(title) = line.strip_prefix('[').and_then(|l| l.strip_suffix(']')) {
            let name = title.trim_end_matches(|c: char| c.is_ascii_digit());
            let index = (name.len() < title.len()).then(|| title[name.len()..].parse().unwrap());
            b.sections.push(bpb::Section { hash: bpb::hash(name.as_bytes()), name: name.as_bytes().to_vec(), index, keys: Vec::new() });
        } else {
            let (k, v) = line.split_once('=').unwrap();
            let s = b.sections.last_mut().unwrap();
            s.keys.push(bpb::Key { hash: bpb::hash(k.as_bytes()), name: k.as_bytes().to_vec(), value: bpb::Value::Text(v.as_bytes().to_vec()) });
        }
    }
    bpb::write(&b).unwrap()
}

fn scene() -> Vec<u8> {
    bpb_of(
        "[SceneInfo]\nSceneName=Test\nDefaultStartPosition=100.0,50.0,100.0\nDefaultStartDirection=0.0,0.0,1.0\n\
         [Environment]\nEcoAmbientScale=1.0,1.0,1.0\nTerrainFile=Test.trn\nTerrainWidth=1\nTerrainBreadth=1\n\
         TerrainWidthScale=9.000000\nTerrainBreadthScale=9.000000\nTerrainBorderWidth=0.000000\nMinDivisionSize=96.000000\n\
         CubeFile=sky1.cub\nDetailMapTilingMultiplier=8\n\
         [Model1]\nSLT=Prop\nPosition=1.0,2.0,3.0\n\
         [Object1]\nCOL=Rock.col\nPosition=1.0,2.0,3.0\n\
         [Object2]\nCOL=Shared.col\nPosition=4.0,2.0,3.0\n",
    )
}

fn psm(texture: &str) -> Vec<u8> {
    bpb_of(&format!("[General]\nTotalShaderPasses=1\n[ShaderPass_1]\ntotalTextureMaps=1\nTextureMap_1={texture}\n"))
}

const MASTER: &str = "[Test General]\nDisplayName=Test Track\nActivity=Test Race\nActivity=Test Free Ride\n\
     [Test Race]\nType=national\nStartPosition=1.0,2.0,3.0\nStartDirection=0.0,0.0,1.0\n\
     [Test Free Ride]\nType=free ride\nStartPosition=2465.528320,401.135345,1255.091919\nStartDirection=0.984868,0.000000,0.173304\nATYFile=Test.aty\n\
     [Other General]\nDisplayName=Other\nActivity=Other Race\n[Other Race]\nType=supercross\n";

fn track() -> Vec<(&'static str, Vec<u8>)> {
    vec![
        (r"env\test\Test.scn.bpb", scene()),
        (r"env\test\Test.ps2.trn", terrain(1, 2)),
        (r"env\test\Test.toy.bpb", bpb_of("[Toys]\nNumberOfToys=0\n")),
        (r"col\Rock.col", mesh_col(1)),
    ]
}

fn mat() -> Vec<(&'static str, Vec<u8>)> {
    vec![(r"tex\Rock.psm.bpb", psm("Rock")), (r"tex\Rock.pst", bytes(9, 5000))]
}

fn mst() -> Vec<(&'static str, Vec<u8>)> {
    vec![(r"env\test\Test.mst", b"#include \"MST\\Global.mst\"\r\n; a comment\r\nTex\\Rock\r\nTex\\Loose\r\nTex\\Gone\r\n".to_vec())]
}

/// What the game opens for the map and only the PSR has, in the order the import follows them.
fn wanted_loose() -> Vec<(&'static str, Vec<u8>)> {
    vec![
        (r"Geo\Prop.psg", bytes(20, 700)),
        (r"Col\Shared.col", mesh_col(2)),
        (r"Skies\sky1.ps2.cub", bytes(21, 900)),
        (r"Tex\Loose.psm.bpb", psm("LooseTex")),
        (r"Tex\LooseTex.atm", bytes(22, 300)),
    ]
}

struct Built {
    root: PathBuf,
    image: PathBuf,
}

fn build(tag: &str) -> Built {
    build_with(tag, Vec::new())
}

/// [`build`] with more PSR members.
fn build_with(tag: &str, extra: Vec<(&'static str, Vec<u8>)>) -> Built {
    let root = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(format!("import-untamed-{tag}"));
    let _ = std::fs::remove_dir_all(&root);
    let iso = root.join("iso");
    std::fs::create_dir_all(&iso).unwrap();
    let mut psr = vec![
        (r"Env\Master.atyList.bpb", bpb_of(MASTER)),
        (r"res\env\Other\Other.res", build_res_padded(&[(r"env\other\Other.scn.bpb", scene())], 64)),
        (r"res\env\Test\Test.res", build_res_padded(&track(), 300)),
        (r"res\env\Test\Test_mat.res", build_res_padded(&mat(), 57)),
        (r"res\env\Test\Test_mst.res", build_res_padded(&mst(), 2003)),
        (r"Env\Test\Test.tdf", bytes(4, 300)),
        (r"Env\Test\Test.ps2.trn", terrain(1, 2)),
        // Decoys: loose, but nothing of the map's names them.
        (r"Col\Unplaced.col", mesh_col(3)),
        (r"Geo\Other.psg", bytes(23, 100)),
        (r"Skies\sky2.ps2.cub", bytes(24, 100)),
        (r"Tex\Rock.psm.bpb", psm("Decoy")),
        (r"Tex\Gone.tga", bytes(25, 100)),
    ];
    psr.extend(wanted_loose());
    psr.extend(extra);
    let cnf = format!("BOOT2 = cdrom0:\\{BOOT};1\r\nVER = 1.01\r\n").into_bytes();
    let aud = build_psr(&[(r"Aud\x.vig", bytes(26, 100)), (r"Aud\y.vig", bytes(27, 100))]);
    let img = build_iso(&[(BOOT, bytes(6, 3000)), ("SYSTEM.CNF", cnf), ("MX08PS2.PSR", build_psr(&psr)), ("MX08AUD.PSR", aud)]);
    let image = iso.join("disc.iso");
    std::fs::write(&image, img).unwrap();
    Built { root, image }
}

fn load(b: &Built) -> Map {
    let mut disc = discx::Disc::open(&b.image).unwrap();
    let master = untamed::master(&mut disc).unwrap();
    untamed::load_map(&mut disc, "Test", &master).unwrap()
}

// ---- the tests ----

/// The shadow sets' archive (M13): every ride opens `res\Shadows.res`, so each map's mod holds its
/// members as the map's own archives' are, and mod.json names it among the members.
#[test]
fn every_map_takes_the_shadow_archive() {
    let shadows: Vec<(&str, Vec<u8>)> = vec![
        (r"tex\shadowCaster1.psm.bpb", psm("RenderTarget")),
        (r"tex\terrainShadowReceiver1.psm.bpb", psm("RenderTarget")),
        (r"tex\shadowReceiver1.psm.bpb", psm("RenderTarget")),
    ];
    let b = build_with("shadows", vec![(r"res\Shadows.res", build_res_padded(&shadows, 1250))]);
    let map = load(&b);
    assert_eq!(map.archives.last().map(|(n, r)| (n.as_str(), r.padding)), Some((untamed::SHADOWS, 1250)));
    let m = import(&map, &b.root.join("stage")).unwrap();
    for (path, bytes) in &shadows {
        assert_eq!(&std::fs::read(m.dir.join(path.replace('\\', "/"))).unwrap(), bytes, "{path}");
    }
    let j = Json::parse(&std::fs::read_to_string(m.dir.join("mod.json")).unwrap()).unwrap();
    let members: Vec<&str> = j.get("source").and_then(|s| s.get("members")).and_then(Json::arr).unwrap().iter().filter_map(Json::str).collect();
    assert_eq!(members[3], untamed::SHADOWS);
    assert!(m.todo.iter().any(|t| t.starts_with(r"res\Shadows.res") && t.contains("1250 zero bytes")), "{:#?}", m.todo);
    let r = check(&m.dir, &mut Discs::new(Some(b.image.parent().unwrap().to_path_buf())));
    assert!(r.ok(), "{:#?}", r);
}

/// A Free Ride with no StartPosition in the master list starts at the scene's DefaultStartPosition on
/// Untamed too (M13, every map ridden): no "not looked up" entry, and one only when that start is under
/// the ground (SM_MB on the disc; here the generated ground is above y = 50).
#[test]
fn without_a_list_start_the_scene_spawns_as_the_game_does() {
    let b = build("nostart");
    let mut map = load(&b);
    map.ride.as_mut().unwrap().spawn = None;
    let m = import(&map, &b.root.join("stage")).unwrap();
    assert!(m.spawn.is_none());
    assert!(!m.todo.iter().any(|t| t.contains("was not looked up")), "{:#?}", m.todo);
    assert!(
        m.todo.iter().any(|t| t.starts_with("Test.scn: the spawn, the scene's DefaultStartPosition (100.0,50.0,100.0), is")
            && t.contains("under the ground")
            && t.contains("M13")),
        "{:#?}",
        m.todo
    );
}

#[test]
fn the_master_list_chooses_the_free_ride_and_national_maps() {
    let b = build("chosen");
    let mut disc = discx::Disc::open(&b.image).unwrap();
    assert_eq!(untamed::map_folders(&disc), vec!["Other".to_string(), "Test".to_string()]);
    let (member, ini) = untamed::master(&mut disc).unwrap();
    assert_eq!(member, r"Env\Master.atyList.bpb");
    assert_eq!(untamed::activity_types(&ini, "test"), vec!["national", "free ride"]);
    let chosen = untamed::chosen(&disc, &ini);
    assert_eq!(chosen[1], ("Test".to_string(), Ok(())));
    assert_eq!(chosen[0].0, "Other");
    assert!(chosen[0].1.as_ref().unwrap_err().contains("supercross"), "{chosen:?}");
}

#[test]
fn a_map_imports_with_its_scene_decoded_and_reads_back() {
    let b = build("import");
    let map = load(&b);
    assert_eq!(map.archives.iter().map(|(_, r)| r.padding).collect::<Vec<_>>(), vec![300, 57, 2003]);
    let m = import(&map, &b.root.join("stage")).unwrap();
    assert_eq!(m.id, "untamed_test");
    assert_eq!(m.name, "Test Track");

    // Every archive member, byte for byte, and the scene decoded beside its BPB.
    for (path, bytes) in track().into_iter().chain(mat()).chain(mst()) {
        assert_eq!(std::fs::read(m.dir.join(path.replace('\\', "/"))).unwrap(), bytes, "{path}");
    }
    let scn = std::fs::read(m.dir.join("env/test/Test.scn")).unwrap();
    assert_eq!(scn, bpb::parse(&scene()).unwrap().to_ini());
    let s = game_sim::track::Scene::parse(&scn).unwrap();
    assert_eq!((s.terrain_file.as_str(), s.models.len(), s.collision.len()), ("Test.trn", 1, 2));

    // The loose members the game opens, in order, and nothing else of the PSR's.
    let want = wanted_loose();
    assert_eq!(m.pulled, want.iter().map(|(p, _)| p.to_string()).collect::<Vec<_>>());
    for (p, bytes) in &want {
        assert_eq!(&std::fs::read(m.dir.join(p.replace('\\', "/"))).unwrap(), bytes, "{p}");
    }
    for decoy in ["Col/Unplaced.col", "Geo/Other.psg", "Skies/sky2.ps2.cub", "Tex/Gone.tga", "Env/Test/Test.tdf"] {
        assert!(!m.dir.join(decoy).exists(), "{decoy} was copied");
    }
    // The archive's material, not the loose decoy at the same path.
    assert_eq!(std::fs::read(m.dir.join("tex/Rock.psm.bpb")).unwrap(), psm("Rock"));

    let j = Json::parse(&std::fs::read_to_string(m.dir.join("mod.json")).unwrap()).unwrap();
    let src = j.get("source").unwrap();
    assert_eq!(src.get("game").and_then(Json::str), Some("MX vs. ATV Untamed"));
    assert_eq!(src.get("boot").and_then(Json::str), Some(BOOT));
    let members: Vec<&str> = src.get("members").and_then(Json::arr).unwrap().iter().filter_map(Json::str).collect();
    assert_eq!(&members[..3], [r"res\env\Test\Test.res", r"res\env\Test\Test_mat.res", r"res\env\Test\Test_mst.res"]);
    assert_eq!(members.len(), 3 + want.len());
    let loose = j.get("loose").unwrap();
    assert_eq!(loose.get(r"Tex\LooseTex.atm").and_then(Json::str), Some(r"texture Tex\LooseTex of material Tex\Loose in env/test/test.mst"));
    assert_eq!(loose.get(r"Col\Shared.col").and_then(Json::str), Some("COL=Shared.col in test.scn.bpb"));
    let sp = j.get("spawn").unwrap();
    assert_eq!(sp.get("from").and_then(Json::str), Some(r"Env\Master.atyList.bpb [Test Free Ride] StartPosition, StartDirection"));

    let has = |all: &[&str]| assert!(m.todo.iter().any(|t| all.iter().all(|w| t.contains(w))), "no todo has all of {all:?}: {:#?}", m.todo);
    has(&["env/test/Test.scn: not a disc member", "Test.scn.bpb", "tools/bpb"]);
    has(&[r"res\env\Test\Test_mst.res", "2003 zero bytes"]);
    has(&["*.psm.bpb (2 files", "BPB"]);
    has(&["*.toy.bpb (1 files"]);
    has(&["Test.toy.bpb: the toys are BPB"]);
    has(&["[Test Free Ride] ATYFile=Test.aty"]);
    // The pulled files are found where the scene names them: no entry calls one missing.
    assert!(!m.todo.iter().any(|t| t.contains("Shared.col") || t.contains("finds no file") || t.contains("in no archive")), "{:#?}", m.todo);

    let r = check(&m.dir, &mut Discs::new(Some(b.image.parent().unwrap().to_path_buf())));
    assert!(r.ok(), "{:#?}", r);
    assert!(r.note.iter().any(|n| n.contains("2003 zero bytes padding its directory")), "{:#?}", r.note);
}

#[test]
fn a_free_ride_default_start_is_named_and_not_used() {
    let ini = "[FS_A General]\nDisplayName=Cleaver FMX\nActivity=FS_A Free Ride\n[FS_A Free Ride]\nType=free ride\nDefaultStartPosition=764.09,0.15,471.03\n";
    let fr = untamed::free_ride("M", ini.as_bytes(), "FS_A").unwrap();
    assert_eq!(fr.name, "Cleaver FMX");
    assert!(fr.spawn.is_none());
    assert!(fr.todo.iter().any(|t| t.contains("[FS_A Free Ride] DefaultStartPosition=764.09,0.15,471.03") && t.contains("0x002cb110")), "{:?}", fr.todo);
}

#[test]
fn the_binary_imports_untamed_and_says_what_it_skips() {
    let b = build("bin");
    let out = b.root.join("bin-stage");
    let o = Command::new(env!("CARGO_BIN_EXE_import-mxvatv")).arg("--disc").arg(&b.image).arg("--out").arg(&out).output().unwrap();
    let text = String::from_utf8_lossy(&o.stdout);
    assert!(o.status.success(), "{text}{}", String::from_utf8_lossy(&o.stderr));
    assert!(text.contains("skip Other: its activities are supercross"), "{text}");
    assert!(text.contains("ok   untamed_test \"Test Track\"") && text.contains("read back") && text.contains("1 maps imported, 0 failed"), "{text}");
    assert!(out.join("untamed_test/mod.json").is_file() && !out.join("untamed_other").exists());
}

// ---- the user's disc ----

fn real_disc() -> PathBuf {
    let iso = PathBuf::from(std::env::var("DISCX_ISO").unwrap_or_else(|_| concat!(env!("CARGO_MANIFEST_DIR"), "/../../iso").into()));
    discx::images(&iso)
        .unwrap()
        .into_iter()
        .find(|(_, b)| b.as_deref() == Some(BOOT))
        .map(|(p, _)| p)
        .unwrap_or_else(|| panic!("no {BOOT} disc in {}", iso.display()))
}

#[test]
#[ignore = "needs the MX vs. ATV Untamed disc (SLUS_217.01) in iso/ or $DISCX_ISO"]
fn every_chosen_untamed_map_imports_and_reads_back() {
    let image = real_disc();
    let mut disc = discx::Disc::open(&image).unwrap();
    let master = untamed::master(&mut disc).unwrap();
    let chosen: Vec<String> = untamed::chosen(&disc, &master.1).into_iter().filter(|(_, c)| c.is_ok()).map(|(f, _)| f).collect();
    assert_eq!(chosen.len(), 40, "{chosen:?}");
    let out = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("import-untamed-disc");
    let _ = std::fs::remove_dir_all(&out);
    let mut discs = Discs::new(image.parent().map(PathBuf::from));
    let mut failed = Vec::new();
    let mut pulled = 0;
    for f in &chosen {
        let map = untamed::load_map(&mut disc, f, &master).unwrap();
        match import(&map, &out) {
            Err(e) => failed.push(format!("{f}: {e}")),
            Ok(m) => {
                pulled += m.pulled.len();
                let r = check(&m.dir, &mut discs);
                if !r.ok() {
                    failed.push(format!("{f}: {:?}", r.fail));
                }
                let _ = std::fs::remove_dir_all(&m.dir);
            }
        }
    }
    println!("{} maps, {pulled} loose members pulled", chosen.len());
    assert!(failed.is_empty(), "{failed:#?}");
}

#[test]
#[ignore = "needs the MX vs. ATV Untamed disc (SLUS_217.01) in iso/ or $DISCX_ISO"]
fn every_untamed_col_goes_through_openmx_first() {
    // The 1,604 loose `.col` (`Col\`, `Veh\Col\`, `Env\Nat_DVCA\Anim\`) and the 1,472 in the map archives.
    let mut disc = discx::Disc::open(&real_disc()).unwrap();
    let (mut read, mut refused) = (0, Vec::new());
    let loose: Vec<_> = disc.entries().iter().filter(|e| e.name.to_ascii_lowercase().ends_with(".col")).cloned().collect();
    for e in &loose {
        match game_sim::track::Collision::parse(&disc.read(e).unwrap()) {
            Ok(_) => read += 1,
            Err(why) => refused.push(format!("{}: {why}", e.name)),
        }
    }
    assert_eq!(loose.len(), 1604);
    refused.sort();
    assert_eq!(refused.len(), 2, "{refused:#?}");
    assert!(refused[0].starts_with(r"Col\FS_Panel_Blue01.col") && refused[1].starts_with(r"Env\Nat_DVCA\Anim\Nat05_Gallows_Door.col"), "{refused:#?}");
    let mut in_res = 0;
    for f in untamed::map_folders(&disc) {
        for suffix in ["", "_mat", "_mst"] {
            let Some(e) = disc.find(&format!(r"res\env\{f}\{f}{suffix}.res")).cloned() else { continue };
            let r = disc.res(&e).unwrap();
            for m in r.entries().iter().filter(|m| m.name.to_ascii_lowercase().ends_with(".col")) {
                game_sim::track::Collision::parse(&r.read(m).unwrap()).unwrap_or_else(|why| panic!("{}:{}: {why}", e.name, m.name));
                in_res += 1;
            }
        }
    }
    println!("{read} loose .col read, {} refused; {in_res} in the map archives, all read", refused.len());
    assert_eq!(in_res, 1472);
}
