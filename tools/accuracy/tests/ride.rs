//! The per-map openmx run (T10, `accuracy ride`) on generated data: a mod folder written here, and logs
//! in openmx's own words. Nothing here needs a disc or openmx; notes/accuracy.md has the real run.

use accuracy::look;
use accuracy::modfiles::TrackMod;
use accuracy::pose::Pose;
use accuracy::ride::{self, Log, Shell};
use std::path::{Path, PathBuf};
use trnwrite::scene::SceneText;
use trnwrite::{Grid, Options};

/// A fresh folder under the system's temp directory, removed when dropped.
struct Temp(PathBuf);

impl Temp {
    fn new(tag: &str) -> Temp {
        use std::sync::atomic::{AtomicU32, Ordering};
        static N: AtomicU32 = AtomicU32::new(0);
        let p = std::env::temp_dir().join(format!("accuracy-ride-{tag}-{}-{}", std::process::id(), N.fetch_add(1, Ordering::Relaxed)));
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

/// `<mods>/tracks/gen_hills/`: a two-level terrain of rolling hills (512 samples at 4.5 ft, so the scene
/// says 2 x 2), its scene with the start at `start`, and a mod.json with `extra` fields.
fn generated_mod(mods: &Path, start: [f32; 3], extra: &str) -> PathBuf {
    let dir = mods.join("tracks/gen_hills");
    let (w, d) = (512u32, 512u32);
    let feet: Vec<f32> = (0..w * d)
        .map(|i| {
            let (x, z) = ((i % w) as f32, (i / w) as f32);
            120.0 + 40.0 * (x / 23.0).sin() * (z / 31.0).cos() + 0.15 * x
        })
        .collect();
    let (grid, hs) = Grid::from_feet(w, d, &feet, 4.5).unwrap();
    let trn = trnwrite::build(&grid, &Options::new(4.5, hs)).unwrap();
    let (bytes, _) = trnwrite::write(&trn).unwrap();
    std::fs::create_dir_all(dir.join("Env/gen")).unwrap();
    // The scene names the terrain without the platform tag, as MX vs. ATV's do: the shell adds `.ps2`.
    std::fs::write(dir.join("Env/gen/gen.ps2.trn"), &bytes).unwrap();
    let mut scene = SceneText::for_terrain(&trn, "gen", start, [0.6, 0.0, 0.8]);
    scene.terrain_file = "gen.trn".into();
    std::fs::write(dir.join("Env/gen/gen.scn"), scene.to_text()).unwrap();
    std::fs::write(
        dir.join("mod.json"),
        format!(
            r#"{{"standard": 1, "kind": "track", "id": "gen_hills", "name": "Hills", "map": "Gen",
            "source": {{"game": "MX vs. ATV Unleashed", "boot": "SLUS_211.04"}},
            "lookups": {{"scene": "Env/gen/gen.scn", "terrain": "Env/gen/gen.ps2.trn"}}{extra}}}"#
        ),
    )
    .unwrap();
    dir
}

/// What openmx printed for Lago (T10's run), cut to the lines the run reads and a few it does not.
const LAGO: &str = "mods: mxvatv_nat05 (Lago) from /x/out/mods/tracks/mxvatv_nat05
stunts: 57 on the disc, 24 this rider can do
National05:HighPlains: 271 props, 243 collision shapes, ground 5756 x 5756 feet in 1280 x 1280 samples
solid: 243 of 245 collision meshes (2)
  Anaheim_pole.col: no such member
  Anaheim_pole.col: no such member
start: 2113, 447, 3015 facing 110 degrees, 11 camera presets
the dirt under the start line is at 446.7 feet
map: 5760 x 5760 feet to its edge, shot back from 150 feet short of it; radar 864 feet across
wreck: reason 13 — L3 and R3 together to get back on
  1s 61.4 fps, 242309 triangles a frame,   1.8 mph
ride hash 0x6ca0cd0be1d392c1
end: 2147, 442, 3001
ridden for 6.0s over 180 frames, ending at 2147.41, 441.91, 3000.78
";

#[test]
fn the_ride_command_line_built_from_a_mod() {
    let pose = Pose::from_angles([1700.0, 560.0, 1700.0], 45.0, -25.0, 85.0);
    let folder = Path::new("/my mods/tracks/mxvatv_nat05");
    let args = ride::ride_args(folder, Path::new("/out/ride-overview.openmx.png"), &pose, 180);
    let word = |flag: &str| args.iter().position(|a| a == flag).map(|i| args[i + 1].as_str());
    assert_eq!(word("--mods"), Some("/my mods"));
    assert_eq!(word("--track"), Some("mxvatv_nat05"));
    // The ride goes on after the shot: the frames are the ride's, not the pair's two.
    assert_eq!(word("--frames"), Some("180"));
    assert_eq!(word("--shot"), Some("/out/ride-overview.openmx.png"));
    assert!(Pose::parse_md10(word("--shot-pose").unwrap()).unwrap().close(&pose, 1e-3));
    assert_eq!(word("--graphics"), Some("original"));
    assert!(args.contains(&"--headless".to_owned()));
    assert!(args.iter().all(|a| !a.contains('{')), "{args:?}");
    // The same flags as the pairs' template, the frames apart.
    assert_eq!(ride::RIDE_ARGS.replace("{frames}", "2"), look::OPENMX_ARGS);
}

#[test]
fn a_ride_log_reads() {
    let log = ride::read_log(LAGO, "");
    assert_eq!(log.loaded.as_deref(), Some("mxvatv_nat05"));
    assert_eq!(log.start, Some([2113.0, 447.0, 3015.0]));
    assert_eq!(log.dirt, Some(446.7));
    assert_eq!(log.edge, Some((5760.0, 5760.0)));
    assert_eq!(log.frames, Some(180));
    assert_eq!(log.end, Some([2147.41, 441.91, 3000.78]));
    assert_eq!(log.wrecks, [ride::CANNONBALL]);
    assert_eq!(log.unsolid, ["Anaheim_pole.col"]);
    assert!(log.refused.is_empty());
    // A left-out mod and anything on stderr are refusals.
    let log = ride::read_log("mods: tracks/gen_x left out: no .scn\n", "error: no track gen_x\n");
    assert_eq!(log.loaded, None);
    assert_eq!(log.refused, ["mods: tracks/gen_x left out: no .scn", "error: no track gen_x"]);
}

#[test]
fn loads_wants_the_mod_taken_and_every_missing_mesh_in_todo() {
    let log = ride::read_log(LAGO, "");
    let (ok, why) = ride::loads_verdict("mxvatv_nat05", &log, true, "Col\\Anaheim_pole.col: not on the disc");
    assert!(ok, "{why}");
    assert!(why.contains("Anaheim_pole.col not loaded, in todo"), "{why}");
    let (ok, why) = ride::loads_verdict("mxvatv_nat05", &log, true, "");
    assert!(!ok && why.contains("Anaheim_pole.col not loaded and not in todo"), "{why}");
    assert!(!ride::loads_verdict("mxvatv_nat04", &log, true, "Anaheim_pole.col").0);
    assert!(!ride::loads_verdict("mxvatv_nat05", &log, false, "Anaheim_pole.col").0);
    let refused = ride::read_log(LAGO, "thread 'main' panicked\n");
    assert!(!ride::loads_verdict("mxvatv_nat05", &refused, true, "Anaheim_pole.col").0);
}

#[test]
fn the_shell_loads_a_generated_mod_as_openmx_does() {
    let t = Temp::new("load");
    let dir = generated_mod(&t.0, [600.0, 200.0, 650.0], "");
    let shell = Shell::load(&dir).unwrap();
    let m = TrackMod::load(&dir).unwrap();
    assert_eq!(shell.scn, "Env\\gen\\gen.scn");
    // `gen.trn` found as `gen.ps2.trn`, the same bytes the harness reads.
    assert_eq!(shell.terrain, m.terrain);
    assert_eq!(shell.start(), [600.0, 200.0, 650.0]);
    assert_eq!(shell.edge(), (2304.0, 2304.0));
    // The same ground everywhere but the last sample strip, where openmx's ride query has none.
    let (ok, why) = ride::heights_verdict(&shell, &m, None, 500, 7).unwrap();
    assert!(ok, "{why}");
    let a = ride::shell_against_harness(&shell, &m, &[[100.0, 100.0], [1000.5, 2000.25], [2299.5, 500.0], [500.0, 2299.5]]);
    assert_eq!((a.differ, a.strip), (0, 2), "{a:?}");

    // mod.json's spawn takes the scene's place.
    let t2 = Temp::new("spawn");
    let dir = generated_mod(&t2.0, [600.0, 200.0, 650.0], r#", "spawn": {"from": "test", "position": [700.5, 180, 720], "direction": [0, 0, 1]}"#);
    assert_eq!(Shell::load(&dir).unwrap().start(), [700.5, 180.0, 720.0]);

    // A folder with two scenes is refused, as openmx refuses it.
    std::fs::write(dir.join("Env/gen/other.scn"), b"[SceneInfo]\r\n").unwrap();
    assert!(Shell::load(&dir).err().unwrap().contains("2 .scn"));
}

#[test]
fn start_and_edge_against_a_ride() {
    let t = Temp::new("start");
    let dir = generated_mod(&t.0, [600.0, 200.0, 650.0], "");
    let shell = Shell::load(&dir).unwrap();
    let m = TrackMod::load(&dir).unwrap();
    let terrain = shell.parse();
    let mut ground = game_sim::world::Ground::new(&terrain, &shell.scene.terrain);
    let under = ride::ground_y(&mut ground, 600.0, 650.0);
    let at_end = ride::ground_y(&mut ground, 640.0, 700.0);
    let ridden = |dirt: f32, end_y: f32, wrecks: Vec<u32>, edge: (f32, f32)| Log {
        loaded: Some("gen_hills".into()),
        dirt: Some(dirt),
        edge: Some(edge),
        frames: Some(180),
        end: Some([640.0, end_y, 700.0]),
        wrecks,
        ..Default::default()
    };
    let good = ridden((under * 10.0).round() / 10.0, at_end + 0.4, vec![], (2304.0, 2304.0));
    let (ok, why) = ride::start_verdict(&shell, &good);
    assert!(ok, "{why}");
    assert!(why.contains("ft over the dirt") || why.contains("under the dirt"), "{why}");
    // Fallen through, a wreck, or a dirt that is not the shell ground's: not on the ground.
    assert!(!ride::start_verdict(&shell, &ridden(under, at_end - 40.0, vec![], (2304.0, 2304.0))).0);
    let (ok, why) = ride::start_verdict(&shell, &ridden(under, at_end, vec![11], (2304.0, 2304.0)));
    assert!(!ok && why.contains("landed on the rear"), "{why}");
    assert!(!ride::start_verdict(&shell, &ridden(under + 3.0, at_end, vec![], (2304.0, 2304.0))).0);
    assert!(!ride::start_verdict(&shell, &Log::default()).0);

    let (ok, why) = ride::edge_verdict(&shell, &m, &good);
    assert!(ok, "{why}");
    assert!(!ride::edge_verdict(&shell, &m, &ridden(under, at_end, vec![], (5760.0, 5760.0))).0);
    let (ok, why) = ride::edge_verdict(&shell, &m, &ridden(under, at_end, vec![ride::CANNONBALL], (2304.0, 2304.0)));
    assert!(!ok && why.contains("thrown off the map"), "{why}");
}

#[test]
fn the_toy_with_collision_is_the_shot() {
    let t = Temp::new("toy");
    let dir = generated_mod(&t.0, [600.0, 200.0, 650.0], "");
    std::fs::write(
        dir.join("Env/gen/gen.toy"),
        "[animatedtoys]\r\nNumberOfanimatedtoys=2\r\n\r\n[AnimatedToy1]\r\nName=Flag\r\nPosition=1,2,3\r\nHasCollision=FALSE\r\n\r\n\
         [AnimatedToy2]\r\nName=Nat07_WM_Blade1\r\nPosition=2698.491943,478.733826,2303.025879\r\nMCFName=Nat07_WM_Blade.mcf\r\nHasCollision=TRUE\r\n\r\n[Triggers]\r\n",
    )
    .unwrap();
    assert_eq!(ride::collision_toy(&dir), Some(("Nat07_WM_Blade1".to_owned(), [2698.491943, 478.733826, 2303.025879])));
    std::fs::remove_file(dir.join("Env/gen/gen.toy")).unwrap();
    assert_eq!(ride::collision_toy(&dir), None);
}
