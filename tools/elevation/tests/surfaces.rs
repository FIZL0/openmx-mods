//! M16: surface ids from the aerial picture, written into the terrain and read back with openmx's
//! reader; and the freeride survey's counting, on a generated terrain and mask.

use elevation::surfaces::{Cover, DIRT, GRASS, Ground, SAND, Seen, layers, survey};
use elevation::{Request, build_mod};
use game_render::pst::Texture;
use game_sim::track::Terrain;
use std::path::{Path, PathBuf};
use tiff::encoder::{TiffEncoder, colortype};
use tiff::tags::Tag;

fn dir(name: &str) -> PathBuf {
    let d = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("elevation-surfaces")
        .join(name);
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).unwrap();
    d
}

fn geo(img: &mut tiff::encoder::DirectoryEncoder<'_, std::fs::File, tiff::encoder::TiffKindStandard>, px: f64) {
    img.write_tag(Tag::Unknown(33550), &[px, px, 0.0][..]).unwrap();
    img.write_tag(Tag::Unknown(33922), &[0.0f64, 0.0, 0.0, 0.0, 300.0, 0.0][..])
        .unwrap();
    img.write_tag(
        Tag::Unknown(34735),
        &[1u16, 1, 0, 4, 1024, 0, 1, 1, 1025, 0, 1, 1, 3072, 0, 1, 26911, 3076, 0, 1, 9001][..],
    )
    .unwrap();
}

/// 300 × 300 m of gentle ground, its corner at (0, 300).
fn dem(path: &Path) {
    let (w, h) = (300u32, 300u32);
    let px: Vec<f32> = (0..w * h).map(|k| 100.0 + (k % 300) as f32 * 0.01).collect();
    let mut e = TiffEncoder::new(std::fs::File::create(path).unwrap()).unwrap();
    let mut img = e.new_image::<colortype::Gray32Float>(w, h).unwrap();
    geo(img.encoder(), 1.0);
    img.write_data(&px).unwrap();
}

/// A four-band picture over the same ground at 0.5 m: east of x = 150 m green (red 40, NIR 160:
/// NDVI 0.6), west of it bare (red 200, NIR 100: NDVI −0.33).
fn naip(path: &Path) {
    let (w, h) = (600u32, 600u32);
    let px: Vec<u8> = (0..w * h)
        .flat_map(|k| if k % w >= 300 { [40, 120, 40, 160] } else { [200, 180, 150, 100] })
        .collect();
    let mut e = TiffEncoder::new(std::fs::File::create(path).unwrap()).unwrap();
    let mut img = e.new_image::<colortype::RGBA8>(w, h).unwrap();
    geo(img.encoder(), 0.5);
    img.write_data(&px).unwrap();
}

fn request(d: &Path, ground: Option<(Ground, f32)>) -> Request {
    Request {
        id: "elev_test".into(),
        name: "Generated half green".into(),
        dem: vec![d.join("dem.tif")],
        naip: vec![d.join("naip.tif")],
        centre: (150.0, 150.0),
        cells: (1, 1),
        texture: 64,
        surface: 7,
        ground,
        data: "generated GeoTIFF (test)".into(),
        date: "2026-10-22".into(),
        sky: None,
    }
}

/// Every sample's surface id as openmx's `Terrain` reads it from the built file.
fn ids(b: &elevation::Built) -> Vec<u8> {
    let trn = &b.files.iter().find(|(n, _)| n.ends_with(".ps2.trn")).unwrap().1;
    let t = Terrain::parse(trn).unwrap();
    let n = 257u32;
    let mut out = Vec::new();
    for z in 0..n {
        for x in 0..n {
            let (cx, dx) = if x + 1 < n { (x, 0) } else { (x - 1, 1) };
            let (cz, dz) = if z + 1 < n { (z, 0) } else { (z - 1, 1) };
            out.push(t.cell(cx, cz, 1).unwrap().surface[(dx + 2 * dz) as usize]);
        }
    }
    out
}

#[test]
fn the_picture_gives_each_sample_its_surface_and_the_terrain_reads_it_back() {
    let d = dir("build");
    dem(&d.join("dem.tif"));
    naip(&d.join("naip.tif"));
    // The window: sample x's centre is at 22.5 + x m east, so the green edge (150 m) falls between
    // samples 127 and 128.
    let edge = 128usize;

    let dunes = build_mod(&request(&d, Some((Ground::Dunes, 0.03)))).unwrap();
    let forest = build_mod(&request(&d, Some((Ground::Forest, -0.2)))).unwrap();
    let desert = build_mod(&request(&d, Some((Ground::Desert, f32::INFINITY)))).unwrap();
    let fixed = build_mod(&request(&d, None)).unwrap();
    let (a, b, c, e) = (ids(&dunes), ids(&forest), ids(&desert), ids(&fixed));
    for z in 0..257usize {
        for x in 0..257usize {
            let k = x + z * 257;
            // Past the averaging square's reach of the edge, each side is its own.
            if x + 3 <= edge {
                assert_eq!((a[k], b[k]), (SAND, DIRT), "{x},{z}");
            } else if x >= edge + 2 {
                assert_eq!((a[k], b[k]), (DIRT, GRASS), "{x},{z}");
            }
            assert_eq!(c[k], DIRT);
            assert_eq!(e[k], 7);
        }
    }
    // The facts count what was written, and mod.json and todo carry them.
    let count = |v: &[u8], id: u8| v.iter().filter(|&&s| s == id).count() as u64;
    assert_eq!(
        dunes.facts.surfaces,
        vec![(DIRT, count(&a, DIRT)), (SAND, count(&a, SAND))]
    );
    assert_eq!(fixed.facts.surfaces, vec![(7, 257 * 257)]);
    let json = String::from_utf8(dunes.files[0].1.clone()).unwrap();
    assert!(json.contains(&format!("\"surfaces\": {{\"0\": {}, \"4\": {}}}", count(&a, 0), count(&a, 4))));
    assert!(dunes.todo.iter().any(|t| t.contains("4 (zone 5, sandy dirt)") && t.contains("dunes")));
    assert!(fixed.todo.iter().any(|t| t.contains("every surface id is 7")));

    // Without a picture the ground cannot be classified: the fixed id.
    let mut r = request(&d, Some((Ground::Dunes, 0.03)));
    r.naip.clear();
    let none = build_mod(&r).unwrap();
    assert!(ids(&none).iter().all(|&s| s == 7));
}

#[test]
fn a_lone_green_sample_is_averaged_away_and_a_patch_is_not() {
    let (w, h) = (20usize, 20usize);
    let bare: Seen = [200.0, 180.0, 150.0, 100.0];
    let green: Seen = [40.0, 120.0, 40.0, 160.0];
    let mut seen = vec![Some(bare); w * h];
    seen[5 + 5 * w] = Some(green);
    for z in 12..18 {
        for x in 12..18 {
            seen[x + z * w] = Some(green);
        }
    }
    // A sample with no picture adds nothing to its neighbours' average.
    seen[0] = None;
    let c = Ground::Forest.classify(&seen, w, 0.0);
    assert_eq!(c[5 + 5 * w], Cover::Dirt);
    assert_eq!(c[15 + 15 * w], Cover::Grass);
    assert_eq!(c[0], Cover::Dirt);
    assert_eq!(c[1 + w], Cover::Dirt);
    assert_eq!(Cover::Sand.id(), 4);
    assert_eq!(Cover::Grass.id(), 3);
    assert_eq!(Cover::Dirt.id(), 0);
}

const PSM: &str = "[General]\r\nTotalShaderPasses = 3\r\n\
[ShaderPass_1]\r\nTextureMap_1 = Test_Grass\r\nNumAnimators = 1\r\n\
[ShaderPass_1_Animator_1]\r\nAlgorithm = UVAnimation\r\nUScale = -0.01\r\nVScale = 0.01\r\n\
[ShaderPass_2]\r\nTextureMap_1 = Test_Sand_Mask\r\nNumAnimators = 1\r\n\
[ShaderPass_2_Animator_1]\r\nAlgorithm = UVAnimation\r\nUScale = 0.0078125\r\nVScale = -0.0078125\r\n\
[ShaderPass_3]\r\nTextureMap_1 = Test_Sand\r\n";

#[test]
fn the_survey_counts_each_surface_under_the_layer_that_shows_most() {
    let (base, masked) = layers(PSM).unwrap();
    assert_eq!(base, "test_grass");
    assert_eq!(masked.len(), 1);
    assert_eq!((masked[0].0.as_str(), masked[0].1.as_str()), ("test_sand_mask", "test_sand"));
    assert_eq!(masked[0].2, [0.0078125, -0.0078125]);

    // A 257-sample terrain at 0.5 ft (128 ft across, the mask's whole span): surface 9 on the west
    // half, 0 on the east; the mask opaque on the west half.
    let n = 257u32;
    let feet = vec![0.0f32; (n * n) as usize];
    let (mut g, hs) = trnwrite::Grid::from_feet(n, n, &feet, 0.5).unwrap();
    g.surfaces = (0..n * n).map(|k| if k % n < 128 { 9 } else { 0 }).collect();
    let t = trnwrite::build(&g, &trnwrite::Options::new(0.5, hs)).unwrap();
    let (trn, _) = trnwrite::write(&t).unwrap();
    let mask = Texture {
        width: 64,
        height: 64,
        rgba: (0..64 * 64)
            .flat_map(|k| [0, 0, 0, if k % 64 < 32 { 255 } else { 0 }])
            .collect(),
    };
    let s = survey(&trn, PSM, |name| (name == "test_sand_mask").then(|| mask.clone()), n, 0.5).unwrap();
    assert_eq!(s.layers, ["test_grass", "test_sand"]);
    assert_eq!(s.samples, u64::from(n * n));
    // Away from the edge (the mask is read bilinear) the west is sand and 9, the east grass and 0.
    let sand9 = s.count.get(&(9, 1)).copied().unwrap_or(0);
    let grass0 = s.count.get(&(0, 0)).copied().unwrap_or(0);
    assert!(sand9 >= u64::from(n * 126), "{:?}", s.count);
    assert!(grass0 >= u64::from(n * 126), "{:?}", s.count);
    assert!(s.table().contains("test_sand"));
}
