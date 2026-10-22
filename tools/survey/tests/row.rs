//! A map built here, byte by byte: a `.res` holding a PLN v7 terrain with two texture pages between its
//! blocks and its tree (the layout openmx's `Terrain::parse` refuses, MD1), an INI scene, a mesh `.col`
//! and a class-1 `.col`. Surveying it must give exactly the row written down below.

use game_sim::track::Terrain;
use survey::{BLOCK_BYTES, layout, map_res, md_row, res_view, terrain, terrain_view, textures};

const BLOCK: usize = 24576;

fn adler32(d: &[u8]) -> u32 {
    let (mut a, mut b) = (1u32, 0u32);
    for &c in d {
        a = (a + c as u32) % 65521;
        b = (b + a) % 65521;
    }
    (b << 16) | a
}

/// One `.res` block: a zlib stream of one stored deflate block.
fn zlib_stored(data: &[u8]) -> Vec<u8> {
    let len = data.len() as u16;
    let mut z = vec![0x78, 0x01, 0x01];
    z.extend_from_slice(&len.to_le_bytes());
    z.extend_from_slice(&(!len).to_le_bytes());
    z.extend_from_slice(data);
    z.extend_from_slice(&adler32(data).to_be_bytes());
    z
}

/// A `.res` in the layout `game_sim::archive::Res::parse` documents, its directory followed by `pad`
/// zero bytes that the header's directory length counts (Untamed's map archives do this).
fn build_res(members: &[(&str, Vec<u8>)], pad: usize) -> Vec<u8> {
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
    dir.resize(dir.len() + pad, 0);
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

fn w(v: &mut Vec<u8>, x: u32) {
    v.extend_from_slice(&x.to_le_bytes());
}

fn f(v: &mut Vec<u8>, x: f32) {
    w(v, x.to_bits());
}

/// A 97-byte terrain node.
fn node(v: &mut Vec<u8>, block: u16, children_at: u32) {
    for x in [1.0, 0.25, 0.0, 100.0] {
        f(v, x);
    }
    v.extend_from_slice(&0u16.to_le_bytes());
    v.extend_from_slice(&block.to_le_bytes());
    v.extend_from_slice(&[0; 4 + 64 + 4 + 1]);
    w(v, children_at);
}

/// One leaf block whose bytes are all distinct-ish, so a block moved by a byte reads differently.
fn block(seed: u8) -> Vec<u8> {
    let mut b: Vec<u8> = (0..BLOCK_BYTES)
        .map(|i| (i as u8).wrapping_mul(31).wrapping_add(seed))
        .collect();
    b[..4].copy_from_slice(&7i32.to_le_bytes());
    b
}

/// PLN v7, one level, 3×2 leaves at 3.5 ft, two blocks, two 4×4 texture pages before the tree.
fn build_trn() -> Vec<u8> {
    let mut t = b"PLN\0".to_vec();
    t.push(b'7');
    t.extend_from_slice(b"ps2\0");
    let mut name = b"test".to_vec();
    name.resize(32, 0);
    t.extend_from_slice(&name);
    w(&mut t, 0); // no palette
    f(&mut t, 3.5);
    w(&mut t, 1); // levels
    let tree_at_slot = t.len();
    w(&mut t, 0); // the tree offset, filled in below
    w(&mut t, 2); // declared
    w(&mut t, 2); // blocks
    w(&mut t, 0xaaaa_0001);
    w(&mut t, 0xaaaa_0002);
    w(&mut t, 2); // textures
    w(&mut t, 2); // textures again
    let table_at = t.len();
    w(&mut t, 0);
    w(&mut t, 0);
    t.extend(block(1));
    t.extend(block(2));
    let mut pages = Vec::new();
    for k in 0..2u32 {
        pages.push(t.len() as u32);
        w(&mut t, 0x27);
        w(&mut t, 1);
        for x in [4, 4, 16, 0] {
            w(&mut t, x);
        }
        t.extend((0..16).map(|i| (i + k * 16) as u8));
    }
    for (k, p) in pages.iter().enumerate() {
        t[table_at + k * 4..table_at + k * 4 + 4].copy_from_slice(&p.to_le_bytes());
    }
    let tree_at = t.len() as u32;
    t[tree_at_slot..tree_at_slot + 4].copy_from_slice(&tree_at.to_le_bytes());
    let table = tree_at + 97;
    let leaves_at = table + 256 * 4;
    node(&mut t, 0, table);
    let cells = [(0, 0), (1, 0), (2, 0), (0, 1), (1, 1), (2, 1)];
    let mut children = vec![0u32; 256];
    for (i, (cx, cz)) in cells.iter().enumerate() {
        children[cx + cz * 16] = leaves_at + i as u32 * 97;
    }
    for c in children {
        w(&mut t, c);
    }
    for i in 0..cells.len() {
        node(&mut t, (i % 2) as u16, 0);
    }
    t
}

/// A version-2 mesh `.col`: one triangle.
fn build_col(class: u32) -> Vec<u8> {
    let mut c = Vec::new();
    w(&mut c, 1234);
    c.extend_from_slice(b"col\0");
    w(&mut c, 2);
    w(&mut c, class);
    w(&mut c, 5); // surface
    for x in [
        1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0,
    ] {
        f(&mut c, x);
    }
    for n in [3, 0, 1, 0] {
        w(&mut c, n); // vertices, nodes, triangles, not packed
    }
    for x in [0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0] {
        f(&mut c, x);
    }
    w(&mut c, 0); // no normals
    for x in [0.0, 0.0, 1.0, 0.0] {
        f(&mut c, x);
    }
    for i in [0u16, 1, 2] {
        c.extend_from_slice(&i.to_le_bytes());
    }
    c
}

const SCENE: &str = "[SceneInfo]\r\nSceneName=TestMap\r\n\r\n[Environment]\r\nTerrainFile=test.ps2.trn\r\n\
TerrainWidth=1\r\nTerrainBreadth=1\r\nTerrainWidthScale=3.500000\r\nTerrainBreadthScale=3.500000\r\n\r\n\
[Model1]\r\nSLT=rock.slt\r\nPosition=1.0,2.0,3.0\r\n\r\n\
[Object1]\r\nCOL=rock_a.col\r\nPosition=1.0,2.0,3.0\r\n\r\n\
[Object2]\r\nCOL=Rock_A.col\r\nPosition=4.0,2.0,3.0\r\n\r\n\
[Object3]\r\nCOL=shared.col\r\nPosition=5.0,2.0,3.0\r\n\r\n\
[Object4]\r\nCOL=gone.col\r\nPosition=6.0,2.0,3.0\r\n";

fn members() -> Vec<(&'static str, Vec<u8>)> {
    vec![
        (r"Env\test\test.scn", SCENE.as_bytes().to_vec()),
        (r"Env\test\test.ps2.trn", build_trn()),
        (r"Geo\rock.psg", vec![1, 2, 3]),
        (r"Col\rock_a.col", build_col(0)),
        (r"Env\test\anim\Swing.col", build_col(1)),
    ]
}

const EXPECTED: &[&str] = &[
    "test",
    "1",
    "3×2",
    "3.5",
    "168",
    "2",
    "2 × 4×4 fmt 0x27, 1 mip, 40 B †",
    "TestMap",
    "1×1 @ 3.5",
    "1",
    "4 (3 files: 1 here, 1 loose, missing gone.col)",
    "1 of 2",
    "Swing.col",
];

fn survey(res: Vec<u8>) -> survey::MapRow {
    map_res("test", r"Env\test\test.res", res, &mut |name| {
        name == "shared.col"
    })
    .unwrap()
}

#[test]
fn a_generated_res_gives_the_expected_row() {
    let row = survey(build_res(&members(), 0));
    assert_eq!(row.cells(), EXPECTED);
    assert_eq!(row.res_padding, 0);
    assert_eq!(row.source_note, None);
    let t = row.terrain.as_ref().unwrap();
    assert!(
        t.viewed
            .as_ref()
            .unwrap()
            .contains("but the tree starts at"),
        "{:?}",
        t.viewed
    );
    let refused: Vec<_> = row.refused().collect();
    assert_eq!(
        refused,
        [(
            r"Env\test\anim\Swing.col",
            "collision class 1 is not understood"
        )]
    );
    assert_eq!(
        md_row(&row.cells()).matches(" | ").count(),
        survey::COLUMNS.len() - 1
    );
}

#[test]
fn a_padded_directory_is_read_through_a_view_and_gives_the_same_row() {
    let padded = build_res(&members(), 260);
    assert!(
        game_sim::archive::Res::parse(&padded).is_err(),
        "openmx now takes padding: drop the view"
    );
    let row = survey(padded);
    assert_eq!(row.cells(), EXPECTED);
    assert_eq!(row.res_padding, 260);
    assert!(row.source_note.is_some());
    // Without any padding to take off, a refused .res stays refused.
    let mut broken = build_res(&members(), 0);
    broken[0] = 4;
    assert!(res_view(broken).is_err());
}

#[test]
fn the_view_moves_only_the_blocks_and_the_reader_sees_the_files_own() {
    let trn = build_trn();
    let l = layout(&trn).unwrap();
    assert_eq!(
        (l.declared, l.blocks, l.textures, l.table.len()),
        (2, 2, 2, 2)
    );
    let tex = textures(&trn, &l).unwrap();
    assert_eq!(tex.gap, 80);
    assert_eq!(tex.pages[1].at, tex.pages[0].at + 40);
    assert!(Terrain::parse(&trn).is_err());

    let view = terrain_view(&trn, &l).unwrap();
    assert_eq!(view.len(), trn.len());
    assert_eq!(
        view[l.tree_at..],
        trn[l.tree_at..],
        "the tree must not move"
    );
    let t = Terrain::parse(&view).unwrap();
    for k in 0..2u16 {
        let (a, b) = (k as usize * BLOCK_BYTES, (k as usize + 1) * BLOCK_BYTES);
        let own = &trn[l.blocks_at + a..l.blocks_at + b];
        let blk = t.block(k).unwrap();
        assert_eq!(blk.base, 7);
        for (x, z) in [(0, 0), (16, 0), (3, 9), (16, 16)] {
            let i = 4 + (x + z * 20) * 2;
            assert_eq!(
                blk.raw_height(x as u32, z as u32),
                Some(u16::from_le_bytes([own[i], own[i + 1]]))
            );
        }
    }
    let row = terrain("test.ps2.trn", &trn).unwrap();
    assert_eq!(
        (row.leaves, row.across, row.along, row.levels),
        (6, 3, 2, 1)
    );
}

#[test]
fn a_page_table_that_disagrees_with_the_pages_is_refused() {
    let mut trn = build_trn();
    let l = layout(&trn).unwrap();
    let at = l.blocks_at - 4; // the second table entry
    let bad = l.table[1] + 4;
    trn[at..at + 4].copy_from_slice(&bad.to_le_bytes());
    let l = layout(&trn).unwrap();
    let why = textures(&trn, &l).unwrap_err();
    assert!(why.contains("texture 1 is listed at"), "{why}");
}

/// The user's discs, if `iso/` (or $DISCX_ISO) has them: on every map terrain of the two MX vs. ATV
/// discs, every block the reader sees through the view is the file's own block, and the tree did not
/// move. `cargo test -p survey --release -- --ignored`.
#[test]
#[ignore = "needs the user's discs in iso/"]
fn on_the_discs_the_view_shows_the_files_own_blocks() {
    let dir = std::env::var_os("DISCX_ISO")
        .map(std::path::PathBuf::from)
        .unwrap_or("iso".into());
    for (game, pattern) in [("mxvatv", r"Env\*\*.res"), ("untamed", r"Env\*\*.ps2.trn")] {
        let Ok(path) = discx::resolve_image(game, &dir) else {
            eprintln!("{game}: no disc in {}", dir.display());
            continue;
        };
        let mut disc = discx::Disc::open(&path).unwrap();
        let entries: Vec<_> = disc.matching(pattern).into_iter().cloned().collect();
        let mut n = 0;
        for e in entries {
            let lower = e.name.to_ascii_lowercase();
            if lower.ends_with("_mat.res") || lower.ends_with("_mst.res") {
                continue;
            }
            let mut trns = Vec::new();
            if lower.ends_with(".res") {
                let v = res_view(disc.read(&e).unwrap()).unwrap();
                for (name, b) in v.files().unwrap() {
                    if name.to_ascii_lowercase().ends_with(".trn") {
                        trns.push(b);
                    }
                }
            } else {
                trns.push(disc.read(&e).unwrap());
            }
            for trn in trns {
                let l = layout(&trn).unwrap();
                textures(&trn, &l).unwrap();
                let view = terrain_view(&trn, &l).unwrap();
                assert_eq!(view[l.tree_at..], trn[l.tree_at..]);
                let t = Terrain::parse(&view).unwrap();
                for k in 0..t.blocks {
                    let own = &trn[l.blocks_at + k as usize * BLOCK_BYTES..][..BLOCK_BYTES];
                    let blk = t.block(k as u16).unwrap();
                    assert_eq!(blk.base, i32::from_le_bytes(own[..4].try_into().unwrap()));
                    for z in 0..17u32 {
                        for x in 0..17u32 {
                            let i = 4 + (x + z * 20) as usize * 2;
                            let raw = u16::from_le_bytes([own[i], own[i + 1]]);
                            assert_eq!(blk.raw_height(x, z), Some(raw), "{} block {k}", e.name);
                        }
                    }
                }
                n += 1;
            }
        }
        eprintln!("{game}: {n} terrains, every block the file's own through the view");
    }
}

/// Untamed keeps each map's terrain twice: loose under `Env\<MAP>\` and in `res\env\<MAP>\<MAP>.res`.
/// The survey reads the archive's; this says whether the loose copies are the same bytes.
#[test]
#[ignore = "needs the user's discs in iso/"]
fn on_untamed_the_loose_terrains_are_the_archives_own() {
    let dir = std::env::var_os("DISCX_ISO")
        .map(std::path::PathBuf::from)
        .unwrap_or("iso".into());
    let Ok(path) = discx::resolve_image("untamed", &dir) else {
        return;
    };
    let mut disc = discx::Disc::open(&path).unwrap();
    let loose: Vec<_> = disc
        .matching(r"Env\*\*.ps2.trn")
        .into_iter()
        .cloned()
        .collect();
    let (mut same, mut alone) = (0, Vec::new());
    for e in loose {
        let stem = e
            .name
            .rsplit('\\')
            .next()
            .unwrap()
            .split('.')
            .next()
            .unwrap()
            .to_owned();
        let Some(res) = disc.find(&format!(r"res\env\{stem}\{stem}.res")).cloned() else {
            alone.push(stem);
            continue;
        };
        let v = res_view(disc.read(&res).unwrap()).unwrap();
        let inner = v
            .files()
            .unwrap()
            .into_iter()
            .find(|(n, _)| n.to_ascii_lowercase().ends_with(".trn"));
        assert_eq!(inner.unwrap().1, disc.read(&e).unwrap(), "{stem}");
        same += 1;
    }
    eprintln!("untamed: {same} loose terrains identical to their archive's; no archive: {alone:?}");
}

/// [`SCENE`] as Untamed holds a scene: the same sections and keys compiled to a BPB by `tools/bpb`.
fn scene_bpb() -> Vec<u8> {
    use bpb::{Bpb, Key, Section, Value, hash};
    let key = |name: &str, value: Value| Key {
        hash: hash(name.as_bytes()),
        name: name.as_bytes().to_vec(),
        value,
    };
    let text = |s: &str| Value::Text(s.as_bytes().to_vec());
    let section = |name: &str, index: Option<u16>, keys: Vec<Key>| Section {
        hash: hash(name.as_bytes()),
        name: name.as_bytes().to_vec(),
        index,
        keys,
    };
    let mut sections = vec![
        section("SceneInfo", None, vec![key("SceneName", text("TestMap"))]),
        section(
            "Environment",
            None,
            vec![
                key("TerrainFile", text("test.ps2.trn")),
                key("TerrainWidth", Value::Int(1)),
                key("TerrainBreadth", Value::Int(1)),
                key("TerrainWidthScale", Value::Float(3.5)),
                key("TerrainBreadthScale", Value::Float(3.5)),
            ],
        ),
        section(
            "Model",
            Some(1),
            vec![
                key("SLT", text("rock.slt")),
                key("Position", text("1.0,2.0,3.0")),
            ],
        ),
    ];
    for (i, (col, x)) in [
        ("rock_a.col", 1),
        ("Rock_A.col", 4),
        ("shared.col", 5),
        ("gone.col", 6),
    ]
    .into_iter()
    .enumerate()
    {
        sections.push(section(
            "Object",
            Some(i as u16 + 1),
            vec![
                key("COL", text(col)),
                key("Position", text(&format!("{x}.0,2.0,3.0"))),
            ],
        ));
    }
    bpb::write(&Bpb { sections }).unwrap()
}

#[test]
fn an_untamed_bpb_scene_is_decoded_and_gives_the_same_row() {
    let scene = scene_bpb();
    let mut m = members();
    m[0] = (r"Env\test\test.scn.bpb", scene.clone());
    let row = survey(build_res(&m, 100));
    let mut want: Vec<String> = EXPECTED.iter().map(|s| s.to_string()).collect();
    want[7] = format!("TestMap (BPB, {} B)", scene.len());
    assert_eq!(row.cells(), want);

    // A BPB the decoder refuses is a refused scene, not a missing one.
    m[0] = (r"Env\test\test.scn.bpb", b"BPC\0".to_vec());
    let row = survey(build_res(&m, 0));
    assert!(
        row.cells()[7].starts_with("refused: bpb: not a BPB"),
        "{:?}",
        row.cells()[7]
    );
}

/// Every Untamed map's scene is its `<MAP>.scn.bpb`, decoded by `tools/bpb` and read by `Scene::parse`:
/// the model and collision placements it reads are the `[ModelN]` with an `SLT` and the `[ObjectN]`
/// with a `COL` the BPB holds (77,542 in all, as M10 counted on the loose scenes). A `.col` a scene
/// places that neither its archive nor the PSR's `Col\` has is looked for in every other map archive.
#[test]
#[ignore = "needs the user's discs in iso/"]
fn on_untamed_every_scene_is_read_from_its_bpb() {
    use survey::SceneRow;
    let dir = std::env::var_os("DISCX_ISO")
        .map(std::path::PathBuf::from)
        .unwrap_or("iso".into());
    let Ok(path) = discx::resolve_image("untamed", &dir) else {
        eprintln!("untamed: no disc in {}", dir.display());
        return;
    };
    let mut disc = discx::Disc::open(&path).unwrap();
    let loose: std::collections::BTreeSet<String> = disc
        .matching(r"Col\*.col")
        .into_iter()
        .map(|e| e.name.rsplit('\\').next().unwrap().to_ascii_lowercase())
        .collect();
    let mut archives: Vec<_> = disc
        .matching(r"res\env\*\*.res")
        .into_iter()
        .filter(|e| {
            let n = e.name.to_ascii_lowercase();
            !n.ends_with("_mat.res") && !n.ends_with("_mst.res")
        })
        .cloned()
        .collect();
    archives.extend(disc.find(r"Env\SM_IP\SM_IP.scn.bpb").cloned());
    let (mut maps, mut placed, mut every_col) = (0, 0, std::collections::BTreeSet::new());
    let mut missing = Vec::new();
    for e in archives {
        let b = disc.read(&e).unwrap();
        let (row, scene) = if e.name.ends_with(".bpb") {
            let files = vec![(e.name.clone(), b.clone())];
            (
                survey::map("SM_IP", &e.name, &files, &mut |n| loose.contains(n)),
                b,
            )
        } else {
            let v = res_view(b.clone()).unwrap();
            let files = v.files().unwrap();
            let scene = files
                .iter()
                .find(|(n, _)| n.to_ascii_lowercase().ends_with(".scn.bpb"))
                .unwrap_or_else(|| panic!("{}: no .scn.bpb", e.name))
                .1
                .clone();
            every_col.extend(files.iter().filter_map(|(n, _)| {
                let n = n.to_ascii_lowercase();
                n.ends_with(".col")
                    .then(|| n.rsplit('\\').next().unwrap().to_owned())
            }));
            (
                map_res(&e.name, &e.name, b, &mut |n| loose.contains(n)).unwrap(),
                scene,
            )
        };
        let SceneRow::Ini {
            models,
            placements,
            bpb: Some(bytes),
            missing: gone,
            ..
        } = &row.scene
        else {
            panic!("{}: {:?}", e.name, row.scene);
        };
        let held = bpb::parse(&scene).unwrap();
        let count = |prefix: &[u8], file: &[u8]| {
            held.sections
                .iter()
                .filter(|s| s.name.eq_ignore_ascii_case(prefix) && s.index.is_some())
                .filter(|s| {
                    s.keys
                        .iter()
                        .any(|k| k.name.eq_ignore_ascii_case(file) && !k.value.text().is_empty())
                })
                .count()
        };
        assert_eq!(*bytes, scene.len(), "{}", e.name);
        assert_eq!(*models, count(b"Model", b"SLT"), "{}", e.name);
        assert_eq!(*placements, count(b"Object", b"COL"), "{}", e.name);
        placed += models + placements;
        missing.extend(gone.iter().map(|g| (row.map.clone(), g.clone())));
        maps += 1;
    }
    let elsewhere: Vec<_> = missing
        .iter()
        .filter(|(_, g)| every_col.contains(g))
        .collect();
    eprintln!(
        "untamed: {maps} scenes read from their BPB, {placed} placements; placed but not found: \
         {missing:?}; of those in another map's archive: {elsewhere:?}"
    );
    assert_eq!(maps, 49);
    assert_eq!(placed, 77_542);
}
