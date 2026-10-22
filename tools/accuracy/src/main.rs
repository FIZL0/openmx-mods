//! `accuracy`: a track mod against its source game running in PCSX2. `accuracy --help` for usage.

use accuracy::heights::{self, Oracle};
use accuracy::modfiles::TrackMod;
use accuracy::pose::Pose;
use accuracy::{look, ram, source};
use game_sim::world::Ground;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};

const USAGE: &str = "accuracy: a track mod against its source game running in PCSX2 (T7, notes/accuracy.md)

  accuracy setup --game mxvatv [--iso DIR]
      Writes extracted/<game>/kit.toml, the kit's config for the source game, with the disc found in
      DIR (default $DISCX_ISO, else iso/) by its boot ELF.

  accuracy heights --mod DIR --ram IMAGE [--points N] [--seed S] [--out DIR]
      The original's own terrain queries, run on IMAGE (32 MB of EE RAM of the game in a ride on the
      mod's map: `./kit state ram`, or `accuracy capture`), against openmx's port on the mod's .trn, at
      N points (default 2000) in each of the four ways the terrain object can be set. Writes
      DIR/heights.txt (default out/accuracy/<map>/). Exit 1 if anything differs.

  accuracy capture --out IMAGE [--kit KIT] [--slot 8]
      Saves the running game (kit copy $KIT_PCSX2) into a scratch slot and extracts its EE RAM.

  accuracy scene --mod DIR --ram IMAGE [--out DIR]
      The scene's [SceneInfo], [Fog] and [Environment] as the game holds them in IMAGE, against the
      mod's .scn, field by field (sceneram.rs). Writes DIR/scene.txt. Exit 1 if a field differs by more
      than an ulp.

  accuracy files --mod DIR --ram IMAGE [--out DIR]
      The resource names the game opened for the map (its name pool in IMAGE) against the mod's files
      (loaded.rs). Writes DIR/files.txt. Exit 1 if the game opened a file of the map the mod lacks.

  accuracy pose --ram IMAGE [--mod DIR]
      The viewports in IMAGE, and the camera pose of the one over the mod's terrain.

  accuracy poses --mod DIR [--out DIR] [--landmarks N] [--add NAME=x,y,z,yaw,pitch,fov]...
      Writes the map's standard poses into DIR/poses.txt (the scene's start, the centre looking four
      ways, an overview, the highest ground looking across), one looking at each of the map's N
      biggest landmarks (default 3, landmarks.rs), and any --add ones.

  accuracy shoot --mod DIR [--viewport ADDR] [--name POSE]... [--out DIR] [--kit KIT] [--settle MS]
  accuracy shoot --mod DIR [--viewport ADDR] --live NAME [--pad CMD]...
      Live, in the running game (kit copy $KIT_PCSX2): holds the ride camera, writes each pose of
      DIR/poses.txt (or the --name ones) into its viewport, waits (default 800 ms), takes `./kit shot`
      as DIR/POSE.original.png and checks the viewport kept the pose; lets the camera go at the end.
      --live keeps the game's own camera as it is (after the --pad commands) as pose NAME. Without
      --viewport the ride camera's viewport is found in a fresh capture (scratch slot 8).

  accuracy pair --mod DIR [--out DIR] [--openmx BIN] [--openmx-args TEMPLATE]
      openmx's shot from every pose that has none ({mod} {shot} {pose} in TEMPLATE, default
      \"--mod {mod} --shot {shot} --shot-pose {pose}\"), then DIR/POSE.pair.png side by side and
      DIR/index.md. Without MD2/MD10 in openmx the original's shots are indexed alone.

The source game comes from mod.json's source.boot (SLUS_211.04: MX vs. ATV Unleashed). KIT defaults to
../ps2-decomp-kit/kit; $KIT_CONFIG to extracted/<game>/kit.toml.";

struct Args(Vec<String>);

impl Args {
    fn take(&mut self, flag: &str) -> Option<String> {
        let i = self.0.iter().position(|a| a == flag)?;
        self.0.remove(i);
        (i < self.0.len()).then(|| self.0.remove(i))
    }
    fn all(&mut self, flag: &str) -> Vec<String> {
        std::iter::from_fn(|| self.take(flag)).collect()
    }
    fn need(&mut self, flag: &str) -> Result<String, String> {
        self.take(flag).ok_or_else(|| format!("{flag} is needed (--help)"))
    }
    fn done(&self) -> Result<(), String> {
        match self.0.first() {
            Some(a) => Err(format!("unknown argument {a} (--help)")),
            None => Ok(()),
        }
    }
}

fn main() -> ExitCode {
    let mut argv: Vec<String> = std::env::args().skip(1).collect();
    if argv.is_empty() || argv.iter().any(|a| a == "-h" || a == "--help" || a == "help") {
        println!("{USAGE}");
        return ExitCode::SUCCESS;
    }
    let cmd = argv.remove(0);
    let mut a = Args(argv);
    let run = match cmd.as_str() {
        "heights" => cmd_heights(&mut a),
        "capture" => cmd_capture(&mut a),
        "pose" => cmd_pose(&mut a),
        "scene" => cmd_scene(&mut a),
        "files" => cmd_files(&mut a),
        "setup" => cmd_setup(&mut a),
        "poses" => cmd_poses(&mut a),
        "shoot" => cmd_shoot(&mut a),
        "pair" => cmd_pair(&mut a),
        other => Err(format!("unknown command {other} (--help)")),
    };
    match run {
        Ok(true) => ExitCode::SUCCESS,
        Ok(false) => ExitCode::from(1),
        Err(e) => {
            eprintln!("accuracy {cmd}: {e}");
            ExitCode::from(2)
        }
    }
}

fn source_of(m: &TrackMod) -> Result<source::Source, String> {
    source::find(&m.boot).ok_or_else(|| format!("{}: no source game this tool knows has boot ELF {:?}", m.id, m.boot))
}

fn out_dir(a: &mut Args, m: &TrackMod) -> PathBuf {
    a.take("--out").map(PathBuf::from).unwrap_or_else(|| Path::new("out/accuracy").join(m.map.to_ascii_lowercase()))
}

/// The original's terrain object for the mod, in `ram`.
fn terrain_in(ram: &[u8], m: &TrackMod, src: &source::Source) -> Result<u32, String> {
    let (root, scale) = m.root_key();
    match ram::find_terrain(ram, src, &root, scale).as_slice() {
        [one] => Ok(*one),
        [] => Err(format!("no terrain object in the image holds {}'s tree: is it a ride on {}?", m.id, m.map)),
        many => Err(format!("{} terrain objects hold {}'s tree: {many:x?}", many.len(), m.id)),
    }
}

fn cmd_setup(a: &mut Args) -> Result<bool, String> {
    let game = a.need("--game")?;
    let iso_dir = a.take("--iso").or_else(|| std::env::var("DISCX_ISO").ok()).unwrap_or_else(|| "iso".into());
    a.done()?;
    let src = source::find(&game).ok_or_else(|| format!("{game}: not a source this tool knows"))?;
    let images = discx::images(Path::new(&iso_dir))?;
    let (iso, _) = images
        .into_iter()
        .find(|(_, boot)| boot.as_deref().is_some_and(|b| b.eq_ignore_ascii_case(src.boot)))
        .ok_or_else(|| format!("no disc in {iso_dir} boots {}", src.boot))?;
    let iso = std::fs::canonicalize(&iso).map_err(|e| format!("{}: {e}", iso.display()))?;
    let dir = std::env::current_dir().map_err(|e| e.to_string())?.join("extracted").join(src.name);
    std::fs::create_dir_all(&dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    let path = dir.join("kit.toml");
    std::fs::write(&path, source::kit_toml(&src, &iso, &dir)).map_err(|e| format!("{}: {e}", path.display()))?;
    println!("{}: {} on {}", path.display(), src.game, iso.display());
    Ok(true)
}

fn cmd_heights(a: &mut Args) -> Result<bool, String> {
    let m = TrackMod::load(Path::new(&a.need("--mod")?))?;
    let image = a.need("--ram")?;
    let n: usize = a.take("--points").map_or(Ok(2000), |s| s.parse().map_err(|e| format!("--points: {e}")))?;
    let seed: u64 = a.take("--seed").map_or(Ok(0x7acc_5eed), |s| s.parse().map_err(|e| format!("--seed: {e}")))?;
    let out = out_dir(a, &m);
    a.done()?;
    let src = source_of(&m)?;
    let ram = heights::load_ram(Path::new(&image))?;
    let at = terrain_in(&ram, &m, &src)?;
    let mut oracle = Oracle::new(ram, src, at);
    let terrain = m.parse();
    let ground = Ground::new(&terrain, &m.layout);
    let field = ground.field();
    let checks = heights::object_checks(&oracle, &src, &field);
    let (lo, hi) = heights::inner_box(&oracle, &src);
    let points = heights::points(n, seed, lo, hi, terrain.scale.0);
    let modes = heights::modes(oracle.own_mode());
    let as_is = heights::compare(&mut oracle, &field, source::Normals::Raw, &modes, &points, 0.0);
    let with_step = match src.normals {
        source::Normals::Raw => None,
        n if oracle.packed_normals() => {
            eprintln!("{}: the game is on its packed normal table, which the harness does not model", m.id);
            let _ = n;
            None
        }
        n => Some(heights::compare(&mut oracle, &field, n, &modes, &points, 0.0)),
    };
    let text = accuracy::heights_text(&m, &src, at, &checks, &as_is, with_step.as_ref());
    print!("{text}");
    std::fs::create_dir_all(&out).map_err(|e| format!("{}: {e}", out.display()))?;
    std::fs::write(out.join("heights.txt"), &text).map_err(|e| format!("{}: {e}", out.display()))?;
    // The verdict is the heights: the query's and the ride query's y at every point, and the object's
    // own numbers. What else differs is in the report (and notes/accuracy.md).
    Ok(as_is.differ("height") + as_is.differ("ride height") == 0 && checks.iter().all(|c| c.1))
}

fn kit_path(a: &mut Args) -> PathBuf {
    a.take("--kit").map(PathBuf::from).unwrap_or_else(|| PathBuf::from("../ps2-decomp-kit/kit"))
}

/// The kit, with `KIT_CONFIG` defaulted to the source game's config under `extracted/`.
fn kit(kit: &Path, src: Option<&source::Source>) -> Command {
    let mut c = Command::new(kit);
    if std::env::var_os("KIT_CONFIG").is_none()
        && let Some(s) = src
    {
        c.env("KIT_CONFIG", Path::new("extracted").join(s.name).join("kit.toml"));
    }
    c
}

fn run(mut c: Command) -> Result<String, String> {
    let out = c.output().map_err(|e| format!("{c:?}: {e}"))?;
    let text = format!("{}{}", String::from_utf8_lossy(&out.stdout), String::from_utf8_lossy(&out.stderr));
    if out.status.success() { Ok(text) } else { Err(format!("{c:?} failed: {}", text.trim())) }
}

fn cmd_capture(a: &mut Args) -> Result<bool, String> {
    let out = PathBuf::from(a.need("--out")?);
    let k = kit_path(a);
    let slot = a.take("--slot").unwrap_or_else(|| "8".into());
    a.done()?;
    print!("{}", capture(&k, &slot, &out)?);
    Ok(true)
}

/// Save the running copy into `slot` and extract its EE RAM to `out`.
fn capture(k: &Path, slot: &str, out: &Path) -> Result<String, String> {
    let k = k.to_owned();
    // PCSX2 writes the state after `save` returns: wait for the slot's file to change, then for its
    // size to settle. The file is <config>/savestates/<serial> (<crc>).<slot>.p2s.
    let status = run({
        let mut c = kit(&k, None);
        c.args(["pcsx2", "status"]);
        c
    })?;
    let config = status
        .lines()
        .find_map(|l| l.strip_prefix("config").map(|r| PathBuf::from(r.trim())))
        .ok_or("`kit pcsx2 status` names no config folder: is the copy running?")?;
    let states = config.join("savestates");
    let suffix = format!(".{slot:0>2}.p2s");
    let file = || {
        std::fs::read_dir(&states).ok()?.flatten().map(|e| e.path()).find(|p| p.to_string_lossy().ends_with(&suffix))
    };
    let stamp = |p: &Option<PathBuf>| p.as_ref().and_then(|p| std::fs::metadata(p).ok()).map(|m| (m.modified().ok(), m.len()));
    let before = stamp(&file());
    run({
        let mut c = kit(&k, None);
        c.args(["pine", "save", slot]);
        c
    })?;
    let mut last = before;
    for _ in 0..100 {
        std::thread::sleep(std::time::Duration::from_millis(300));
        let now = stamp(&file());
        if now.is_some() && now != before && now == last {
            break;
        }
        last = now;
    }
    if last == before {
        return Err(format!("no new state in {} after the save", states.display()));
    }
    run({
        let mut c = kit(&k, None);
        c.args(["state", "ram", slot, "--out"]).arg(out);
        c
    })
}

fn cmd_pose(a: &mut Args) -> Result<bool, String> {
    let image = a.need("--ram")?;
    let m = a.take("--mod").map(|d| TrackMod::load(Path::new(&d))).transpose()?;
    a.done()?;
    let ram = heights::load_ram(Path::new(&image))?;
    let src = match &m {
        Some(m) => source_of(m)?,
        None => source::MXVATV,
    };
    let vp = src.viewport.ok_or_else(|| format!("{}: no viewport layout known", src.game))?;
    let found = ram::find_viewports(&ram, &vp);
    for &at in &found {
        let p = Pose::from_viewport(&ram, 0, at, &vp)?;
        println!("viewport {at:#x}: eye {:?} forward {:?} fov {}", p.eye, p.forward, p.fov);
    }
    if let Some(m) = &m {
        let t = terrain_in(&ram, m, &src)?;
        let oracle = Oracle::new(ram.clone(), src, t);
        let at = ram::pick_viewport(&ram, &vp, &found, oracle.quad(src.object.min), oracle.quad(src.object.max))?;
        let p = Pose::from_viewport(&ram, 0, at, &vp)?;
        println!("{}'s camera: viewport {at:#x}, --shot-pose {}", m.id, p.md10());
    }
    Ok(true)
}

/// The pose in the live game, read over PINE.
fn live_pose(k: &Path, src: &source::Source, at: u32, scratch: &Path) -> Result<Pose, String> {
    let vp = src.viewport.ok_or("no viewport layout")?;
    run({
        let mut c = kit(k, Some(src));
        c.args(["pine", "block", &format!("{at:x}"), "0x200", "--out"]).arg(scratch);
        c
    })?;
    let bytes = std::fs::read(scratch).map_err(|e| format!("{}: {e}", scratch.display()))?;
    Pose::from_viewport(&bytes, at, at, &vp)
}

fn pine_write(k: &Path, src: &source::Source, at: u32, value: String, kind: &str) -> Result<(), String> {
    let mut c = kit(k, Some(src));
    c.args(["pine", "write", &format!("{at:#x}"), &value, "--as", kind]);
    run(c).map(|_| ())
}

/// Hold the ride camera (or let it go again): its update slot to the stub, or back.
fn hold_camera(k: &Path, src: &source::Source, hold: bool) -> Result<(), String> {
    let cam = src.camera.ok_or_else(|| format!("{}: no way to hold the camera is known", src.game))?;
    let to = if hold { cam.stub } else { cam.update };
    pine_write(k, src, cam.update_slot, format!("{to:#x}"), "u32")
}

/// Write a pose into the viewport: forward and up, then the eye, then the fov.
fn place(k: &Path, src: &source::Source, vp_at: u32, p: &Pose) -> Result<(), String> {
    let vp = src.viewport.ok_or("no viewport layout")?;
    for (off, v) in [(vp.forward, p.forward), (vp.up, p.up), (vp.eye, p.eye)] {
        for (i, c) in v.iter().enumerate() {
            pine_write(k, src, vp_at + off + 4 * i as u32, format!("{c:?}"), "f32")?;
        }
    }
    pine_write(k, src, vp_at + vp.fov, format!("{:?}", p.fov), "f32")
}

fn cmd_poses(a: &mut Args) -> Result<bool, String> {
    let m = TrackMod::load(Path::new(&a.need("--mod")?))?;
    let out = out_dir(a, &m);
    let extra = a.all("--add");
    let n: usize = a.take("--landmarks").map_or(Ok(3), |s| s.parse().map_err(|e| format!("--landmarks: {e}")))?;
    a.done()?;
    for (name, p) in look::standard_poses(&m) {
        look::write_pose(&out, &name, &p)?;
    }
    let marks = accuracy::landmarks::of_mod(&m, n)?;
    for (name, p) in accuracy::landmarks::poses(&m, &marks) {
        look::write_pose(&out, &name, &p)?;
    }
    for e in extra {
        let (name, md10) = e.split_once('=').ok_or_else(|| format!("--add {e}: NAME=x,y,z,yaw,pitch,fov"))?;
        look::write_pose(&out, name, &Pose::parse_md10(md10)?)?;
    }
    for (name, p) in look::read_poses(&out)? {
        println!("{name}: --shot-pose {}", p.md10());
    }
    Ok(true)
}

fn cmd_shoot(a: &mut Args) -> Result<bool, String> {
    let m = TrackMod::load(Path::new(&a.need("--mod")?))?;
    let vp_at = a.take("--viewport");
    let out = out_dir(a, &m);
    let k = kit_path(a);
    let names = a.all("--name");
    let live = a.take("--live");
    let pads = a.all("--pad");
    let settle: u64 = a.take("--settle").map_or(Ok(800), |s| s.parse().map_err(|e| format!("--settle: {e}")))?;
    a.done()?;
    let src = source_of(&m)?;
    let vp = src.viewport.ok_or_else(|| format!("{}: no viewport layout known", src.game))?;
    std::fs::create_dir_all(&out).map_err(|e| format!("{}: {e}", out.display()))?;
    let scratch = out.join(".viewport.bin");
    // The viewport is on the game's heap, somewhere else every session: found in a fresh capture
    // unless given.
    let vp_at = match vp_at {
        Some(v) => u32::from_str_radix(v.trim_start_matches("0x"), 16).map_err(|e| format!("--viewport: {e}"))?,
        None => {
            let image = out.join(".capture.bin");
            capture(&k, "8", &image)?;
            let ram = heights::load_ram(&image);
            let _ = std::fs::remove_file(&image);
            let ram = ram?;
            let t = terrain_in(&ram, &m, &src)?;
            let oracle = Oracle::new(ram.clone(), src, t);
            let found = ram::find_viewports(&ram, &vp);
            let at = ram::pick_viewport(&ram, &vp, &found, oracle.quad(src.object.min), oracle.quad(src.object.max))?;
            println!("the ride camera's viewport is at {at:#x}");
            at
        }
    };
    if !pads.is_empty() {
        let mut c = kit(&k, Some(&src));
        c.args(["pad", "send"]).args(&pads);
        run(c)?;
    }
    let wait = || std::thread::sleep(std::time::Duration::from_millis(settle));
    // The live camera, held where it is: the game's own view, kept as a pose of its own.
    if let Some(name) = live {
        wait();
        hold_camera(&k, &src, true)?;
        let shot = (|| {
            wait();
            let p = live_pose(&k, &src, vp_at, &scratch)?;
            let mut c = kit(&k, Some(&src));
            c.arg("shot").arg(look::original_png(&out, &name));
            run(c)?;
            look::write_pose(&out, &name, &p)?;
            println!("{name}: the game's own camera, --shot-pose {}", p.md10());
            Ok::<(), String>(())
        })();
        hold_camera(&k, &src, false)?;
        let _ = std::fs::remove_file(&scratch);
        return shot.map(|()| true);
    }
    let all = look::read_poses(&out)?;
    let todo: Vec<(String, Pose)> = if names.is_empty() {
        all
    } else {
        names
            .iter()
            .map(|n| all.iter().find(|(m, _)| m == n).cloned().ok_or_else(|| format!("{n}: not in {}", out.join(look::POSES).display())))
            .collect::<Result<_, _>>()?
    };
    if todo.is_empty() {
        return Err(format!("no poses in {} (accuracy poses, or --live NAME)", out.join(look::POSES).display()));
    }
    hold_camera(&k, &src, true)?;
    let mut ok = true;
    for (name, p) in &todo {
        let r = (|| {
            place(&k, &src, vp_at, p)?;
            wait();
            let mut c = kit(&k, Some(&src));
            c.arg("shot").arg(look::original_png(&out, name));
            run(c)?;
            // What the viewport held through the shot: the game normalises forward and up itself, so
            // the read-back is compared to a thousandth.
            let back = live_pose(&k, &src, vp_at, &scratch)?;
            if !back.close(p, 1e-3) {
                return Err(format!("the viewport did not keep the pose: wrote {p:?}, read {back:?}"));
            }
            println!("{name}: {} --shot-pose {}", look::original_png(&out, name).display(), p.md10());
            Ok::<(), String>(())
        })();
        if let Err(e) = r {
            eprintln!("{name}: {e}");
            let _ = std::fs::remove_file(look::original_png(&out, name));
            ok = false;
        }
    }
    hold_camera(&k, &src, false)?;
    let _ = std::fs::remove_file(&scratch);
    Ok(ok)
}

fn cmd_pair(a: &mut Args) -> Result<bool, String> {
    let m = TrackMod::load(Path::new(&a.need("--mod")?))?;
    let out = out_dir(a, &m);
    let openmx = PathBuf::from(a.take("--openmx").unwrap_or_else(|| "openmx/target/release/openmx".into()));
    let template = a.take("--openmx-args").unwrap_or_else(|| "--mod {mod} --shot {shot} --shot-pose {pose}".into());
    a.done()?;
    let src = source_of(&m)?;
    let ready = look::openmx_ready(&openmx);
    let mut ok = true;
    for r in look::rows(&out)? {
        if !r.original {
            continue;
        }
        let shot = look::openmx_png(&out, &r.name);
        if !r.openmx
            && let Ok(()) = ready
        {
            let args: Vec<String> = template
                .split_whitespace()
                .map(|w| {
                    w.replace("{mod}", &m.folder.to_string_lossy()).replace("{shot}", &shot.to_string_lossy()).replace("{pose}", &r.pose.md10())
                })
                .collect();
            let st = Command::new(&openmx).args(&args).status().map_err(|e| format!("{}: {e}", openmx.display()))?;
            if !st.success() || !shot.exists() {
                eprintln!("{}: openmx gave no shot ({st})", r.name);
                ok = false;
            }
        }
        if shot.exists() {
            look::compose(&look::original_png(&out, &r.name), &shot, &look::pair_png(&out, &r.name))?;
        }
    }
    let waiting = ready.as_ref().err().map(|e| format!("not taken: {e}"));
    let index = look::index(&out, &m.map, src.game, waiting.as_deref())?;
    std::fs::write(out.join("index.md"), &index).map_err(|e| format!("{}: {e}", out.display()))?;
    print!("{index}");
    Ok(ok)
}

fn cmd_scene(a: &mut Args) -> Result<bool, String> {
    let m = TrackMod::load(Path::new(&a.need("--mod")?))?;
    let image = a.need("--ram")?;
    let out = out_dir(a, &m);
    a.done()?;
    let src = source_of(&m)?;
    if src.boot != source::MXVATV.boot {
        return Err(format!("{}: the scene's readers are known on MX vs. ATV Unleashed only", src.game));
    }
    let ram = heights::load_ram(Path::new(&image))?;
    // The image must be of this map: its terrain object holds the mod's tree.
    terrain_in(&ram, &m, &src)?;
    let json = std::fs::read_to_string(m.folder.join("mod.json")).map_err(|e| e.to_string())?;
    let json = modcheck::json::Json::parse(&json)?;
    let scn = json.get("lookups").and_then(|l| l.get("scene")).and_then(|s| s.str()).ok_or("mod.json names no scene")?;
    let text = std::fs::read(m.folder.join(scn)).map_err(|e| format!("{scn}: {e}"))?;
    let fields = accuracy::sceneram::compare(&ram, &String::from_utf8_lossy(&text))?;
    let report = accuracy::sceneram::text(&m.id, &fields);
    print!("{report}");
    std::fs::create_dir_all(&out).map_err(|e| format!("{}: {e}", out.display()))?;
    std::fs::write(out.join("scene.txt"), &report).map_err(|e| format!("{}: {e}", out.display()))?;
    Ok(fields.iter().all(|f| f.ulps() <= 1))
}

fn cmd_files(a: &mut Args) -> Result<bool, String> {
    use accuracy::loaded::{self, Have};
    let m = TrackMod::load(Path::new(&a.need("--mod")?))?;
    let image = a.need("--ram")?;
    let out = out_dir(a, &m);
    a.done()?;
    let ram = heights::load_ram(Path::new(&image))?;
    let names = loaded::pool(&ram, &m.map);
    if names.is_empty() {
        return Err(format!("no name pool with env\\{0}\\{0}.scn in the image: is it a ride on {0}?", m.map));
    }
    let (paths, files) = loaded::mod_files(&m.folder);
    let rows = loaded::audit(&names, &paths, &files);
    let count = |h: Have| rows.iter().filter(|r| r.1 == h).count();
    let mut s = format!(
        "{}: the game opened {} names, {} of them the map's: {} at their path in the mod, {} by file name only, {} not in the mod\n",
        m.id,
        names.len(),
        rows.len(),
        count(Have::Path),
        count(Have::Base),
        count(Have::No)
    );
    for (n, h) in &rows {
        if *h != Have::Path {
            s.push_str(&format!("  {:<6} {n}\n", if *h == Have::No { "NOT" } else { "name" }));
        }
    }
    print!("{s}");
    std::fs::create_dir_all(&out).map_err(|e| format!("{}: {e}", out.display()))?;
    std::fs::write(out.join("files.txt"), &s).map_err(|e| format!("{}: {e}", out.display()))?;
    Ok(count(Have::No) == 0)
}
