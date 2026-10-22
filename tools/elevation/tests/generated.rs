//! Generated GeoTIFFs (striped and compressed by the `tiff` encoder, tiled by hand as 3DEP's are) through
//! the reader, the mosaic and the whole build, read back with openmx's readers and the standard's check.

use elevation::dem::Mosaic;
use elevation::geotiff::GeoTiff;
use elevation::{FOOT_M, Request, build_mod, write_mod};
use game_sim::track::{Scene, Terrain};
use std::path::{Path, PathBuf};
use tiff::encoder::{Compression, TiffEncoder, colortype};
use tiff::tags::Tag;

fn dir(name: &str) -> PathBuf {
    let d = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("elevation")
        .join(name);
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).unwrap();
    d
}

/// GeoKeys: projected, corner tiepoint, EPSG `epsg`, metres.
fn keys(epsg: u16, model: u16) -> Vec<u16> {
    vec![
        1, 1, 0, 4, 1024, 0, 1, model, 1025, 0, 1, 1, 3072, 0, 1, epsg, 3076, 0, 1, 9001,
    ]
}

/// A one-band f32 GeoTIFF in strips, LZW, the `tiff` crate's encoder: `w × h` from `f(i, j)`, its corner at
/// (`left`, `top`) and `px` units a pixel.
fn striped(path: &Path, w: u32, h: u32, left: f64, top: f64, px: f64, f: impl Fn(u32, u32) -> f32) {
    let data: Vec<f32> = (0..h)
        .flat_map(|j| (0..w).map(move |i| (i, j)))
        .map(|(i, j)| f(i, j))
        .collect();
    let file = std::fs::File::create(path).unwrap();
    let mut e = TiffEncoder::new(file)
        .unwrap()
        .with_compression(Compression::Lzw);
    let mut img = e.new_image::<colortype::Gray32Float>(w, h).unwrap();
    img.rows_per_strip(7).unwrap();
    img.encoder()
        .write_tag(Tag::Unknown(33550), &[px, px, 0.0][..])
        .unwrap();
    img.encoder()
        .write_tag(Tag::Unknown(33922), &[0.0, 0.0, 0.0, left, top, 0.0][..])
        .unwrap();
    img.encoder()
        .write_tag(Tag::Unknown(34735), &keys(26911, 1)[..])
        .unwrap();
    img.encoder()
        .write_tag(Tag::Unknown(42113), "-999999")
        .unwrap();
    img.write_data(&data).unwrap();
}

/// An RGB u8 GeoTIFF, uncompressed strips.
fn rgb(path: &Path, w: u32, h: u32, left: f64, top: f64, px: f64, f: impl Fn(u32, u32) -> [u8; 3]) {
    let data: Vec<u8> = (0..h)
        .flat_map(|j| (0..w).map(move |i| (i, j)))
        .flat_map(|(i, j)| f(i, j))
        .collect();
    let file = std::fs::File::create(path).unwrap();
    let mut e = TiffEncoder::new(file).unwrap();
    let mut img = e.new_image::<colortype::RGB8>(w, h).unwrap();
    img.encoder()
        .write_tag(Tag::Unknown(33550), &[px, px, 0.0][..])
        .unwrap();
    img.encoder()
        .write_tag(Tag::Unknown(33922), &[0.0, 0.0, 0.0, left, top, 0.0][..])
        .unwrap();
    img.encoder()
        .write_tag(Tag::Unknown(34735), &keys(26911, 1)[..])
        .unwrap();
    img.write_data(&data).unwrap();
}

/// A tiled, uncompressed, little-endian one-band f32 GeoTIFF written byte by byte (the encoder writes
/// strips only), with `tile × tile` tiles, partial ones at the right and bottom edges.
fn tiled(
    path: &Path,
    w: u32,
    h: u32,
    tile: u32,
    left: f64,
    top: f64,
    keys: &[u16],
    f: impl Fn(u32, u32) -> f32,
) {
    let (tx, ty) = (w.div_ceil(tile), h.div_ceil(tile));
    let mut body = Vec::new();
    let mut offsets = Vec::new();
    let data_at = 8u32;
    for b in 0..ty {
        for a in 0..tx {
            offsets.push(data_at + body.len() as u32);
            for y in 0..tile {
                for x in 0..tile {
                    let (i, j) = (a * tile + x, b * tile + y);
                    let v = if i < w && j < h { f(i, j) } else { 0.0 };
                    body.extend_from_slice(&v.to_le_bytes());
                }
            }
        }
    }
    let n = (tx * ty) as usize;
    let tile_bytes = tile * tile * 4;
    // Out-of-line values after the IFD.
    let mut extra: Vec<u8> = Vec::new();
    let ifd_at = data_at + body.len() as u32;
    let entries: u32 = 16;
    let extra_at = ifd_at + 2 + entries * 12 + 4;
    let mut out_of_line = |bytes: Vec<u8>| -> u32 {
        let at = extra_at + extra.len() as u32;
        extra.extend_from_slice(&bytes);
        if extra.len() % 2 == 1 {
            extra.push(0);
        }
        at
    };
    let le32 = |v: &[u32]| v.iter().flat_map(|x| x.to_le_bytes()).collect::<Vec<u8>>();
    let f64s = |v: &[f64]| v.iter().flat_map(|x| x.to_le_bytes()).collect::<Vec<u8>>();
    let u16s = |v: &[u16]| v.iter().flat_map(|x| x.to_le_bytes()).collect::<Vec<u8>>();
    let offs_at = out_of_line(le32(&offsets));
    let counts_at = out_of_line(le32(&vec![tile_bytes; n]));
    let scale_at = out_of_line(f64s(&[1.0, 1.0, 0.0]));
    let tie_at = out_of_line(f64s(&[0.0, 0.0, 0.0, left, top, 0.0]));
    let keys_at = out_of_line(u16s(keys));
    let nodata_at = out_of_line(b"-999999\0".to_vec());
    // (tag, type, count, value or offset): SHORT 3, LONG 4, ASCII 2, DOUBLE 12.
    let inline16 = |v: u16| u32::from(v);
    let tags: Vec<(u16, u16, u32, u32)> = vec![
        (256, 4, 1, w),
        (257, 4, 1, h),
        (258, 3, 1, inline16(32)),
        (259, 3, 1, inline16(1)),
        (262, 3, 1, inline16(1)),
        (277, 3, 1, inline16(1)),
        (284, 3, 1, inline16(1)),
        (322, 4, 1, tile),
        (323, 4, 1, tile),
        (324, 4, n as u32, if n == 1 { offsets[0] } else { offs_at }),
        (
            325,
            4,
            n as u32,
            if n == 1 { tile_bytes } else { counts_at },
        ),
        (339, 3, 1, inline16(3)),
        (33550, 12, 3, scale_at),
        (33922, 12, 6, tie_at),
        (34735, 3, keys.len() as u32, keys_at),
        (42113, 2, 8, nodata_at),
    ];
    assert_eq!(tags.len() as u32, entries);
    let mut file = b"II*\0".to_vec();
    file.extend_from_slice(&ifd_at.to_le_bytes());
    file.extend_from_slice(&body);
    file.extend_from_slice(&(entries as u16).to_le_bytes());
    for (tag, ty, count, v) in tags {
        file.extend_from_slice(&tag.to_le_bytes());
        file.extend_from_slice(&ty.to_le_bytes());
        file.extend_from_slice(&count.to_le_bytes());
        file.extend_from_slice(&v.to_le_bytes());
    }
    file.extend_from_slice(&0u32.to_le_bytes());
    file.extend_from_slice(&extra);
    std::fs::write(path, file).unwrap();
}

/// Dunes: a slope, two crossing waves and a crest, in metres around 40–110, with a tall marker at
/// pixel (`mi`, `mj`).
fn dunes(i: u32, j: u32, mark: (u32, u32)) -> f32 {
    let (x, y) = (i as f32, j as f32);
    if (i, j) == mark {
        return 160.25;
    }
    60.0 + 0.02 * x - 0.01 * y
        + 18.0 * (x * 0.05).sin() * (y * 0.031).cos()
        + 9.0 * ((x + 2.0 * y) * 0.017).sin()
        + 0.001 * ((i * 7 + j * 13) % 97) as f32
}

#[test]
fn a_striped_lzw_geotiff_reads_back_with_its_place() {
    let d = dir("striped");
    let p = d.join("a.tif");
    striped(&p, 50, 30, 670000.0, 3660000.0, 1.0, |i, j| {
        (i * 100 + j) as f32 + 0.25
    });
    let mut t = GeoTiff::open(&p).unwrap();
    let g = t.geo.clone();
    assert_eq!((g.width, g.height, g.bands), (50, 30, 1));
    assert_eq!((g.left, g.top, g.dx, g.dy), (670000.0, 3660000.0, 1.0, 1.0));
    assert_eq!(
        (g.epsg, g.model, g.linear_units, g.vertical_units),
        (Some(26911), 1, Some(9001), None)
    );
    assert_eq!(g.nodata, Some(-999999.0));
    assert_eq!(g.x(0.0), 670000.5);
    assert_eq!(g.y(0.0), 3659999.5);
    // A window across strip boundaries (7 rows a strip).
    let w = t.window(3, 5, 20, 11).unwrap();
    for j in 0..11 {
        for i in 0..20 {
            assert_eq!(
                w[(i + j * 20) as usize],
                ((i + 3) * 100 + j + 5) as f32 + 0.25
            );
        }
    }
    assert!(t.window(40, 0, 20, 1).is_err());
}

#[test]
fn a_tiled_geotiff_reads_across_tiles_and_partial_edges() {
    let d = dir("tiled");
    let p = d.join("t.tif");
    tiled(&p, 37, 29, 16, 500.0, 900.0, &keys(26911, 1), |i, j| {
        (i * 1000 + j) as f32
    });
    let mut t = GeoTiff::open(&p).unwrap();
    assert_eq!(t.geo.width, 37);
    let w = t.window(10, 12, 27, 17).unwrap();
    for j in 0..17 {
        for i in 0..27 {
            assert_eq!(
                w[(i + j * 27) as usize],
                ((i + 10) * 1000 + j + 12) as f32,
                "{i},{j}"
            );
        }
    }
}

#[test]
fn a_mosaic_joins_tiles_on_one_grid_and_refuses_others() {
    let d = dir("mosaic");
    // Two 40×40 tiles, the second 34 pixels east (six overlap), the same values where they overlap.
    let f = |i: u32, j: u32| (i * 3 + j * 1000) as f32;
    striped(&d.join("w.tif"), 40, 40, 1000.0, 2000.0, 1.0, f);
    striped(
        &d.join("e.tif"),
        40,
        40,
        1034.0,
        2000.0,
        1.0,
        move |i, j| f(i + 34, j),
    );
    let mut m = Mosaic::open(&[d.join("w.tif"), d.join("e.tif")]).unwrap();
    assert_eq!(m.at, vec![(0, 0), (34, 0)]);
    let (px, o) = m.window(20, 5, 50, 10).unwrap();
    for j in 0..10 {
        for i in 0..50 {
            assert_eq!(px[(i + j * 50) as usize], f(i + 20, j + 5));
        }
    }
    assert_eq!((o.pixels, o.differ), (6 * 10, 0));
    // Past both: NaN.
    let (px, _) = m.window(70, 0, 10, 1).unwrap();
    assert!(px[0].is_finite() && px[3].is_finite() && px[4].is_nan());

    // Half a pixel off: not one grid.
    striped(&d.join("off.tif"), 10, 10, 1040.5, 2000.0, 1.0, f);
    assert!(Mosaic::open(&[d.join("w.tif"), d.join("off.tif")]).is_err());
    // Another pixel size.
    striped(&d.join("big.tif"), 10, 10, 1040.0, 2000.0, 2.0, f);
    assert!(Mosaic::open(&[d.join("w.tif"), d.join("big.tif")]).is_err());
}

fn request(
    d: &Path,
    dem: Vec<PathBuf>,
    naip: Vec<PathBuf>,
    centre: (f64, f64),
    cells: (u32, u32),
) -> Request {
    let _ = d;
    Request {
        id: "elev_test".into(),
        name: "Generated dunes".into(),
        dem,
        naip,
        centre,
        cells,
        texture: 64,
        surface: 3,
        data: "generated GeoTIFF (test)".into(),
        date: "2026-10-22".into(),
        sky: None,
    }
}

#[test]
fn a_generated_dem_builds_a_mod_that_reads_back_sample_for_sample() {
    let d = dir("build");
    // 600 × 400 pixels, tiled like 3DEP, its corner at (670000, 3660000), the marker near the north-east.
    let (w, h) = (600, 400);
    let mark = (450, 100);
    tiled(
        &d.join("dem.tif"),
        w,
        h,
        64,
        670000.0,
        3660000.0,
        &keys(26911, 1),
        |i, j| dunes(i, j, mark),
    );
    // The aerial picture: 0.5 m pixels over the same ground, red in the north-east quarter, blue in the
    // south-west, grey elsewhere.
    rgb(
        &d.join("naip.tif"),
        1200,
        800,
        670000.0,
        3660000.0,
        0.5,
        |i, j| match (i >= 600, j < 400) {
            (true, true) => [220, 20, 20],
            (false, false) => [20, 20, 220],
            _ => [128, 128, 128],
        },
    );
    // Two root cells across, one along: 513 × 257 samples, two levels. The centre puts the window's
    // columns at 44..557 and rows 72..329.
    let req = request(
        &d,
        vec![d.join("dem.tif")],
        vec![d.join("naip.tif")],
        (670300.5, 3659799.5),
        (2, 1),
    );
    let b = build_mod(&req).unwrap();
    let f = &b.facts;
    assert_eq!(f.samples, (513, 257));
    assert_eq!(f.corner_pixel, (300 - 256, 200 + 128));
    assert_eq!(f.corner, (670044.5, 3660000.0 - 328.5));
    assert_eq!(f.scale, 3.28084);
    assert!(f.worst_ft <= f.step_ft / 2.0 + 1e-3, "{f:?}");

    let out = d.join("mods/tracks/elev_test");
    write_mod(&out, &b).unwrap();
    // The standard's own check (§5): mod.json, one scene whose stem is the id, the terrain it names
    // read by openmx, and the layout agreeing with it.
    let r = modcheck::check(&out, &mut modcheck::Discs::new(None));
    assert!(r.ok(), "{:?}", r.fail);

    // Every sample, from the files on disk: openmx's terrain against the DEM's own pixel in feet.
    let trn = std::fs::read(out.join("tracks/elev_test/elev_test.ps2.trn")).unwrap();
    let t = Terrain::parse(&trn).unwrap();
    let scene =
        Scene::parse(&std::fs::read(out.join("tracks/elev_test/elev_test.scn")).unwrap()).unwrap();
    let _ = scene;
    let datum = f.datum;
    assert!(datum <= 40.0 && datum >= 25.0, "datum {datum}");
    let mut top = (0.0f32, 0, 0);
    for sz in 0..256 {
        for sx in 0..512 {
            let q = t.cell(sx, sz, 1).unwrap();
            for (k, (dx, dz)) in [(0, 0), (1, 0), (0, 1), (1, 1)].into_iter().enumerate() {
                let (x, z) = (sx + dx, sz + dz);
                // Sample z = 0 is the window's south row, DEM row 328.
                let (i, j) = (44 + x, 328 - z);
                let want = ((f64::from(dunes(i, j, mark)) - datum) / FOOT_M) as f32;
                let got = (t.scale * q.height[k]).0;
                assert!(
                    (got - want).abs() <= f.step_ft as f32 / 2.0 + 1e-3,
                    "{x},{z}: {got} vs {want}"
                );
                assert_eq!(q.surface[k], 3);
                if got > top.0 {
                    top = (got, x, z);
                }
            }
        }
    }
    // The marker is the highest sample, at x = 450 - 44 east and z = 328 - 100 north: not mirrored.
    assert_eq!((top.1, top.2), (406, 228));

    // The spawn is the centre sample, on the ground.
    let c = (256usize, 128usize);
    let want_y = ((f64::from(dunes(44 + 256, 328 - 128, mark)) - datum) / FOOT_M) as f32;
    assert_eq!(
        f.spawn,
        [c.0 as f32 * 3.28084, want_y, c.1 as f32 * 3.28084]
    );

    // The drape: game-render decodes the .pst; its top-right is red (north-east), bottom-left blue.
    let pst = std::fs::read(out.join("tracks/elev_test/elev_test_naip.pst")).unwrap();
    let tex = game_render::pst::Texture::parse(&pst).unwrap();
    assert_eq!((tex.width, tex.height), (64, 64));
    let at = |u: usize, v: usize| &tex.rgba[4 * (u + v * 64)..4 * (u + v * 64) + 3];
    // The map spans DEM columns 44..557 and rows 72..329: column 300 (x = 256) and row 200 (z = 128)
    // split the colours, which is the texture's middle.
    assert_eq!(at(63, 0), [220, 20, 20]);
    assert_eq!(at(0, 63), [20, 20, 220]);
    assert_eq!(at(0, 0), [128, 128, 128]);
    assert_eq!(at(63, 63), [128, 128, 128]);
    // And the port's paint puts the red at the north-east corner of the map (two texels in: its filter
    // wraps at the edge).
    let psm = String::from_utf8(
        std::fs::read(out.join("tracks/elev_test/terrainscreenspacemesh.psm")).unwrap(),
    )
    .unwrap();
    let s = game_render::splat::Splat::parse(&psm, |_| Some(tex.clone()));
    let (ex, ez) = (512.0 * f.scale, 256.0 * f.scale);
    assert_eq!(s.paint(ex - 60.0, ez - 60.0), [220, 20, 20]);
    assert_eq!(s.paint(60.0, 60.0), [20, 20, 220]);

    // mod.json says public domain and lists the gaps.
    let json = std::fs::read_to_string(out.join("mod.json")).unwrap();
    assert!(json.contains("\"permission\": \"public-domain\""));
    assert!(json.contains("\"game\": \"Real elevation\""));
    assert!(b.todo.iter().any(|t| t.contains("surface id is 3")));
}

#[test]
fn one_root_cell_builds_one_level_without_a_picture() {
    let d = dir("small");
    striped(&d.join("dem.tif"), 300, 300, 0.0, 300.0, 1.0, |i, j| {
        dunes(i, j, (0, 0))
    });
    let req = request(&d, vec![d.join("dem.tif")], vec![], (150.0, 150.0), (1, 1));
    let b = build_mod(&req).unwrap();
    assert_eq!(b.facts.samples, (257, 257));
    assert_eq!(b.files.len(), 3);
    let out = d.join("elev_test");
    write_mod(&out, &b).unwrap();
    let r = modcheck::check(&out, &mut modcheck::Discs::new(None));
    assert!(r.ok(), "{:?}", r.fail);
    // Writing again replaces the earlier build; a folder that is not a mod is never replaced.
    write_mod(&out, &b).unwrap();
    std::fs::create_dir_all(d.join("other")).unwrap();
    assert!(write_mod(&d.join("other"), &b).is_err());
}

#[test]
fn what_cannot_be_built_without_resampling_or_past_the_limits_is_refused() {
    let d = dir("refused");
    // Nodata inside the window.
    striped(&d.join("hole.tif"), 300, 300, 0.0, 300.0, 1.0, |i, j| {
        if (i, j) == (150, 140) {
            -999999.0
        } else {
            50.0
        }
    });
    let e = build_mod(&request(
        &d,
        vec![d.join("hole.tif")],
        vec![],
        (150.0, 150.0),
        (1, 1),
    ))
    .err()
    .unwrap();
    assert!(e.contains("no data"), "{e}");
    // The window off the DEM.
    let e = build_mod(&request(
        &d,
        vec![d.join("hole.tif")],
        vec![],
        (10.0, 150.0),
        (1, 1),
    ))
    .err()
    .unwrap();
    assert!(e.contains("no data"), "{e}");
    // A geographic grid.
    tiled(
        &d.join("geo.tif"),
        300,
        300,
        64,
        -115.0,
        33.0,
        &keys(4269, 2),
        |_, _| 50.0,
    );
    let e = build_mod(&request(
        &d,
        vec![d.join("geo.tif")],
        vec![],
        (-115.0, 33.0),
        (1, 1),
    ))
    .err()
    .unwrap();
    assert!(e.contains("geographic"), "{e}");
    // 17 root cells, and a bad id.
    let e = build_mod(&request(
        &d,
        vec![d.join("hole.tif")],
        vec![],
        (150.0, 150.0),
        (17, 1),
    ))
    .err()
    .unwrap();
    assert!(e.contains("1 to 16"), "{e}");
    let mut r = request(&d, vec![d.join("hole.tif")], vec![], (150.0, 150.0), (1, 1));
    r.id = "glamis".into();
    assert!(build_mod(&r).is_err());
}

#[test]
fn the_drape_takes_each_pixel_from_the_first_image_that_has_it() {
    let d = dir("drape");
    striped(&d.join("dem.tif"), 300, 300, 0.0, 300.0, 1.0, |i, j| {
        dunes(i, j, (0, 0))
    });
    // The first image is 0 (NAIP's fill) over its west half; the second is green everywhere.
    rgb(&d.join("a.tif"), 300, 300, 0.0, 300.0, 1.0, |i, _| {
        if i < 150 { [0, 0, 0] } else { [200, 180, 150] }
    });
    rgb(&d.join("b.tif"), 300, 300, 0.0, 300.0, 1.0, |_, _| {
        [40, 160, 40]
    });
    let mut req = request(
        &d,
        vec![d.join("dem.tif")],
        vec![d.join("a.tif")],
        (150.0, 150.0),
        (1, 1),
    );
    req.texture = 32;
    // Alone, the fill leaves the west half without a picture.
    let e = build_mod(&req).err().unwrap();
    assert!(e.contains("under no aerial image"), "{e}");
    req.naip.push(d.join("b.tif"));
    let b = build_mod(&req).unwrap();
    let (_, pst) = b.files.iter().find(|(p, _)| p.ends_with(".pst")).unwrap();
    let tex = game_render::pst::Texture::parse(pst).unwrap();
    let at = |u: usize, v: usize| tex.rgba[4 * (u + v * 32)..4 * (u + v * 32) + 3].to_vec();
    assert_eq!(at(2, 16), [40, 160, 40]);
    assert_eq!(at(29, 16), [200, 180, 150]);
}
