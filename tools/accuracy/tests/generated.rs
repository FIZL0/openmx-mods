//! tools/accuracy on generated data: terrains from trnwrite, RAM images and viewports laid out by hand.
//! Nothing here needs a disc or an emulator; `tests/lago.rs` is the one map run against the original.

use accuracy::heights::{self, Mode, Original, Over127, Ride, port_ride};
use accuracy::look;
use accuracy::modfiles::TrackMod;
use accuracy::pose::Pose;
use accuracy::ram;
use accuracy::source::{MXVATV, Normals};
use game_sim::contact_search::{Ground as Hit, Heightfield, TerrainField, ground};
use game_sim::world::Ground;
use ps2_float::Vf;
use std::path::{Path, PathBuf};
use trnwrite::scene::SceneText;
use trnwrite::{Grid, Options};

/// A fresh folder under the system's temp directory, removed when dropped.
struct Temp(PathBuf);

impl Temp {
    fn new(tag: &str) -> Temp {
        use std::sync::atomic::{AtomicU32, Ordering};
        static N: AtomicU32 = AtomicU32::new(0);
        let p = std::env::temp_dir().join(format!("accuracy-{tag}-{}-{}", std::process::id(), N.fetch_add(1, Ordering::Relaxed)));
        let _ = std::fs::remove_dir_all(&p);
        std::fs::create_dir_all(&p).unwrap();
        Temp(p)
    }
}

impl Drop for Temp {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

/// Rolling hills 300 samples a side at 4.5 ft, surfaces in stripes; `bump` raises one sample.
fn terrain_bytes(bump: Option<(u32, u32)>) -> (Vec<u8>, trnwrite::Trn) {
    let (w, d) = (300u32, 300u32);
    let mut feet = Vec::with_capacity((w * d) as usize);
    for z in 0..d {
        for x in 0..w {
            let (fx, fz) = (x as f32, z as f32);
            let mut h = 120.0 + 40.0 * (fx / 23.0).sin() * (fz / 31.0).cos() + 0.15 * fx;
            if bump == Some((x, z)) {
                h += 9.0;
            }
            feet.push(h);
        }
    }
    let (mut grid, hs) = Grid::from_feet(w, d, &feet, 4.5).unwrap();
    grid.surfaces = (0..w * d).map(|i| ((i % w) / 7 + (i / w) / 5) as u8 % 5).collect();
    let trn = trnwrite::build(&grid, &Options::new(4.5, hs)).unwrap();
    let (bytes, _) = trnwrite::write(&trn).unwrap();
    (bytes, trn)
}

/// A mod folder holding a generated terrain and its scene, as an importer would leave it.
fn generated_mod(dir: &Path, bump: Option<(u32, u32)>) -> TrackMod {
    let (bytes, trn) = terrain_bytes(bump);
    std::fs::create_dir_all(dir.join("Env/gen")).unwrap();
    std::fs::write(dir.join("Env/gen/gen.ps2.trn"), &bytes).unwrap();
    let scene = SceneText::for_terrain(&trn, "gen", [600.0, 200.0, 650.0], [0.6, 0.0, 0.8]);
    std::fs::write(dir.join("Env/gen/gen.scn"), scene.to_text()).unwrap();
    std::fs::write(
        dir.join("mod.json"),
        r#"{"standard": 1, "kind": "track", "id": "gen_hills", "name": "Hills", "map": "Gen",
            "source": {"game": "MX vs. ATV Unleashed", "boot": "SLUS_211.04"},
            "lookups": {"scene": "Env/gen/gen.scn", "terrain": "Env/gen/gen.ps2.trn"}}"#,
    )
    .unwrap();
    TrackMod::load(dir).unwrap()
}

/// A stand-in original: openmx's own query over a terrain of its choosing, with its normals raw or over
/// 127, the way a source game's cell reader would hand them.
struct Fake<'a> {
    field: TerrainField<'a>,
    normals: Normals,
}

impl Fake<'_> {
    fn with(&self, mode: Mode) -> Box<dyn Heightfield + '_> {
        let mut f = TerrainField { terrain: self.field.terrain, ..self.field };
        f.coarse = mode.coarse;
        f.split_one_way = mode.split_one_way;
        match self.normals {
            Normals::Raw => Box::new(f),
            Normals::Over127 { .. } => Box::new(Over127 { field: f }),
        }
    }
}

impl Original for Fake<'_> {
    fn query(&mut self, mode: Mode, at: Vf) -> Hit {
        ground(self.with(mode).as_ref(), at)
    }
    fn ride(&mut self, mode: Mode, at: Vf) -> Option<Ride> {
        Some(port_ride(self.with(mode).as_ref(), at))
    }
}

const OWN: Mode = Mode { coarse: false, split_one_way: true };

/// The generated grid's 300 samples, in feet: past them the tree repeats the last row.
const SIDE: f32 = 299.0 * 4.5;

fn run(original: &TrackMod, original_normals: Normals, port: &TrackMod, port_normals: Normals, n: usize) -> heights::Report {
    run_in(original, original_normals, port, port_normals, n, [0.01, 0.01], [SIDE, SIDE])
}

fn run_in(original: &TrackMod, original_normals: Normals, port: &TrackMod, port_normals: Normals, n: usize, lo: [f32; 2], hi: [f32; 2]) -> heights::Report {
    let (to, tp) = (original.parse(), port.parse());
    let (go, gp) = (Ground::new(&to, &original.layout), Ground::new(&tp, &port.layout));
    let mut fake = Fake { field: go.field(), normals: original_normals };
    let field = gp.field();
    let points = heights::points(n, 7, lo, hi, 4.5);
    heights::compare(&mut fake, &field, port_normals, &heights::modes(OWN), &points, 0.0)
}

#[test]
fn a_copied_terrain_is_equal_at_every_point_and_field() {
    let t = Temp::new("same");
    let m = generated_mod(&t.0, None);
    let r = run(&m, Normals::Raw, &m, Normals::Raw, 1200);
    assert_eq!(r.modes.len(), 4);
    assert_eq!(r.total_points(), 4800);
    assert_eq!(r.total_mismatches(), 0, "{:?}", r.mismatches.first());
    assert!(r.modes.iter().all(|(_, t)| t.cells == t.points && t.rides == t.points));
}

#[test]
fn normals_over_127_leave_the_heights_and_are_matched_by_the_step() {
    let t = Temp::new("over127");
    let m = generated_mod(&t.0, None);
    let step = Normals::Over127 { packed: 0 };
    let as_is = run(&m, step, &m, Normals::Raw, 1000);
    assert_eq!(as_is.differ("height") + as_is.differ("ride height"), 0);
    assert_eq!(as_is.differ("four surfaces") + as_is.differ("weights") + as_is.differ("ride surface"), 0);
    assert!(as_is.differ("normal") > 0, "normalising from bytes and from bytes/127 rounds apart somewhere");
    let with_step = run(&m, step, &m, step, 1000);
    assert_eq!(with_step.total_mismatches(), 0, "{:?}", with_step.mismatches.first());
}

#[test]
fn a_changed_sample_is_found() {
    let (a, b) = (Temp::new("orig"), Temp::new("bumped"));
    let original = generated_mod(&a.0, None);
    // The bump is at sample (150, 140); the points are drawn from the ten samples around it.
    let port = generated_mod(&b.0, Some((150, 140)));
    let r = run_in(&original, Normals::Raw, &port, Normals::Raw, 2000, [145.0 * 4.5, 135.0 * 4.5], [155.0 * 4.5, 145.0 * 4.5]);
    assert!(r.differ("height") > 0 && r.differ("ride height") > 0);
    for x in &r.mismatches {
        let (sx, sz) = (x.at[0] / 4.5, x.at[1] / 4.5);
        assert!((147.0..=152.0).contains(&sx) && (137.0..=142.0).contains(&sz), "a difference far from the bump: {x:?}");
    }
    let src = MXVATV;
    let text = accuracy::heights_text(&port, &src, 0x1000, &[], &r, None);
    assert!(text.lines().next().unwrap().contains("heights DIFFER"), "{text}");
}

#[test]
fn points_cover_the_box_the_same_way_every_time() {
    let p = heights::points(2000, 42, [0.01, 0.01], [5759.99, 5759.99], 4.5);
    assert_eq!(p.len(), 2000);
    assert_eq!(p, heights::points(2000, 42, [0.01, 0.01], [5759.99, 5759.99], 4.5));
    assert!(p.iter().all(|q| q.iter().all(|v| (0.01..=5759.99).contains(v))));
    let on_sample = p.iter().filter(|q| q[0] % 4.5 == 0.0).count();
    assert!(on_sample >= 400, "{on_sample} of 2000 on a sample column");
    let (lo, hi) = p.iter().fold((f32::MAX, f32::MIN), |(l, h), q| (l.min(q[0]), h.max(q[0])));
    assert!(lo < 100.0 && hi > 5600.0, "the points reach both edges: {lo}..{hi}");
}

#[test]
fn the_terrain_object_is_found_by_its_tree() {
    let t = Temp::new("find");
    let m = generated_mod(&t.0, None);
    let (root, scale) = m.root_key();
    let o = MXVATV.object;
    let mut ram = vec![0u8; 1 << 20];
    let node = 0x8000u32;
    ram[(node + 0x0c) as usize..(node + 0x1c) as usize].copy_from_slice(&root);
    let put = |ram: &mut Vec<u8>, at: u32, v: u32| ram[at as usize..at as usize + 4].copy_from_slice(&v.to_le_bytes());
    // The object, and a decoy pointing at the same node whose scale is not the file's.
    put(&mut ram, 0x2000 + o.tree, node);
    put(&mut ram, 0x2000 + o.height_scale, scale);
    put(&mut ram, 0x5000 + o.tree, node);
    put(&mut ram, 0x5000 + o.height_scale, 2.0f32.to_bits());
    assert_eq!(ram::find_terrain(&ram, &MXVATV, &root, scale), vec![0x2000]);
    // A node that is not the root's: nothing.
    ram[(node + 0x10) as usize] ^= 1;
    assert!(ram::find_terrain(&ram, &MXVATV, &root, scale).is_empty());
}

fn put_quad(ram: &mut [u8], at: u32, v: [f32; 4]) {
    for (k, c) in v.iter().enumerate() {
        let a = at as usize + 4 * k;
        ram[a..a + 4].copy_from_slice(&c.to_bits().to_le_bytes());
    }
}

#[test]
fn the_viewport_over_the_terrain_is_the_camera() {
    let vp = MXVATV.viewport.unwrap();
    let mut ram = vec![0u8; 1 << 20];
    let lay = |ram: &mut Vec<u8>, at: u32, eye: [f32; 4], copy: bool| {
        put_quad(ram, at + vp.eye, eye);
        put_quad(ram, at + vp.eye_copy, if copy { eye } else { [0.0; 4] });
        put_quad(ram, at + vp.forward, [0.6, 0.0, 0.8, 1.0]);
        put_quad(ram, at + vp.up, [0.0, 1.0, 0.0, 1.0]);
        put_quad(ram, at + vp.fov, [85.0, 0.0, 0.0, 0.0]);
    };
    lay(&mut ram, 0x4000, [100.0, 50.0, 200.0, 1.0], true);
    lay(&mut ram, 0x9000, [-500.0, 50.0, 200.0, 1.0], true); // a viewport off the map (a menu's)
    lay(&mut ram, 0xc000, [100.0, 50.0, 200.0, 1.0], false); // no second copy: not a viewport
    let found = ram::find_viewports(&ram, &vp);
    assert_eq!(found, vec![0x4000, 0x9000]);
    let at = ram::pick_viewport(&ram, &vp, &found, [0.0; 4], [1000.0, 0.0, 1000.0, 1.0]).unwrap();
    assert_eq!(at, 0x4000);
    let p = Pose::from_viewport(&ram, 0, at, &vp).unwrap();
    assert_eq!(p.eye, [100.0, 50.0, 200.0]);
    // The same pose out of a block read over PINE, which starts at the viewport.
    let block = ram[0x4000..0x4200].to_vec();
    assert_eq!(Pose::from_viewport(&block, 0x4000, 0x4000, &vp).unwrap(), p);
}

#[test]
fn a_pose_goes_to_openmx_and_back() {
    let p = Pose::from_angles([10.0, 20.0, 30.0], 120.0, -15.0, 85.0);
    assert!((p.yaw() - 120.0).abs() < 1e-4 && (p.pitch() + 15.0).abs() < 1e-4);
    let up_dot_forward: f32 = (0..3).map(|k| p.up[k] * p.forward[k]).sum();
    assert!(up_dot_forward.abs() < 1e-6 && p.up[1] > 0.0);
    let q = Pose::parse_md10(&p.md10()).unwrap();
    assert!(q.close(&p, 1e-5), "{q:?} {p:?}");
    let (name, r) = Pose::parse_line(&p.line("ridge")).unwrap();
    assert_eq!((name.as_str(), r), ("ridge", p));
    assert!(Pose::parse_md10("1,2,3").is_err());
}

#[test]
fn the_pose_list_and_its_index() {
    let t = Temp::new("look");
    let m = generated_mod(&t.0.join("mod"), None);
    let out = t.0.join("out");
    let poses = look::standard_poses(&m);
    let names: Vec<&str> = poses.iter().map(|(n, _)| n.as_str()).collect();
    assert_eq!(names, ["start", "centre-n", "centre-e", "centre-s", "centre-w", "overview", "ridge"]);
    let terrain = m.parse();
    let ground = Ground::new(&terrain, &m.layout);
    for (n, p) in &poses {
        let h = ground.at(ps2_float::F(p.eye[0]), ps2_float::F(p.eye[2])).map(|h| h.y.0);
        assert!(h.is_some_and(|h| p.eye[1] > h), "{n}: the eye is under the ground or off the map: {p:?}");
    }
    for (n, p) in &poses {
        look::write_pose(&out, n, p).unwrap();
    }
    // Writing a pose again replaces it.
    let moved = Pose::from_angles([1.0, 2.0, 3.0], 0.0, 0.0, 85.0);
    look::write_pose(&out, "ridge", &moved).unwrap();
    let back = look::read_poses(&out).unwrap();
    assert_eq!(back.len(), 7);
    assert_eq!(back.last().unwrap(), &("ridge".to_owned(), moved));
    std::fs::write(look::original_png(&out, "start"), b"not really a png").unwrap();
    let index = look::index(&out, "Gen", "MX vs. ATV Unleashed", Some("not taken: no MD10")).unwrap();
    assert!(index.contains("| start | `") && index.contains("[yes](start.original.png) | no | no |"), "{index}");
    assert!(index.contains("| ridge | `1,2,3,0,0,85` | no | no | no |"), "{index}");
    assert!(index.contains("not taken: no MD10"));
    assert!(look::write_pose(&out, "two words", &moved).is_err());
}

#[test]
fn a_pair_is_the_two_shots_side_by_side() {
    let have_magick = std::process::Command::new("magick").arg("-version").output().is_ok_and(|o| o.status.success());
    if !have_magick {
        eprintln!("skipped: ImageMagick's magick is not installed");
        return;
    }
    let t = Temp::new("pair");
    let (a, b, out) = (t.0.join("a.ppm"), t.0.join("b.ppm"), t.0.join("pair.ppm"));
    // 4×2 red and 8×4 blue: the right one is scaled to the left one's height.
    let ppm = |w: u32, h: u32, rgb: [u8; 3]| {
        let mut v = format!("P6\n{w} {h}\n255\n").into_bytes();
        for _ in 0..w * h {
            v.extend_from_slice(&rgb);
        }
        v
    };
    std::fs::write(&a, ppm(4, 2, [255, 0, 0])).unwrap();
    std::fs::write(&b, ppm(8, 4, [0, 0, 255])).unwrap();
    look::compose(&a, &b, &out).unwrap();
    let got = std::fs::read(&out).unwrap();
    assert!(got.starts_with(b"P6\n8 2\n255\n"), "{:?}", String::from_utf8_lossy(&got[..12.min(got.len())]));
    let px = &got[11..];
    assert_eq!(&px[0..3], &[255, 0, 0]);
    assert_eq!(&px[3 * 7..3 * 8], &[0, 0, 255]);
}

#[test]
fn the_kit_config_names_the_disc_and_the_gameplay_module() {
    let t = accuracy::source::kit_toml(&MXVATV, Path::new("/discs/mxvatv.iso"), Path::new("/repo/extracted/mxvatv"));
    for want in [
        "serial  = \"SLUS-21104\"",
        "iso     = \"/discs/mxvatv.iso\"",
        "elf  = \"SLUS_211.04\"",
        "file = \"GAME_REL.BIN\"\nload = \"0x45c580\"",
        "context = \"/repo/extracted/mxvatv/context\"",
        "project = \"/repo/extracted/mxvatv/ghidra\"",
    ] {
        assert!(t.contains(want), "{want:?} not in\n{t}");
    }
    let mxu = accuracy::source::kit_toml(&accuracy::source::MXU, Path::new("/d.iso"), Path::new("/x"));
    assert!(mxu.contains("SLUS-20767") && !mxu.contains("[[overlay]]"));
}
