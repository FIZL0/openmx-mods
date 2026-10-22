//! `bpb`: BPB (Untamed's binary INI) to INI text. `bpb help` for usage.

use std::path::{Path, PathBuf};
use std::process::ExitCode;

const USAGE: &str = r#"bpb: MX vs. ATV Untamed's binary INI (.scn.bpb, .toy.bpb, .phy.bpb, ...) to INI text

  bpb FILE.bpb [OUT]     decode one file to OUT (default: stdout)
  bpb -r DIR OUTDIR      decode every .bpb under DIR into OUTDIR, same paths, `.bpb` dropped

A section or key whose stored hash is not its name's is said on stderr, and the exit status is 1."#;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match run(&args) {
        Ok(true) => ExitCode::SUCCESS,
        Ok(false) => ExitCode::FAILURE,
        Err(e) => {
            eprintln!("bpb: {e}");
            ExitCode::FAILURE
        }
    }
}

fn run(args: &[String]) -> Result<bool, String> {
    match args {
        [h] if h == "help" || h == "-h" || h == "--help" => {
            println!("{USAGE}");
            Ok(true)
        }
        [r, dir, out] if r == "-r" => tree(Path::new(dir), Path::new(out)),
        [file] => {
            let (ini, ok) = decode(Path::new(file))?;
            use std::io::Write;
            match std::io::stdout().write_all(&ini) {
                // `bpb FILE | head` stops reading early; that is not a failure.
                Err(e) if e.kind() == std::io::ErrorKind::BrokenPipe => {}
                r => r.map_err(|e| format!("stdout: {e}"))?,
            }
            Ok(ok)
        }
        [file, out] => {
            let (ini, ok) = decode(Path::new(file))?;
            std::fs::write(out, ini).map_err(|e| format!("{out}: {e}"))?;
            Ok(ok)
        }
        _ => Err(format!("bad arguments\n\n{USAGE}")),
    }
}

/// The INI, and whether every hash matched its name.
fn decode(path: &Path) -> Result<(Vec<u8>, bool), String> {
    let bytes = std::fs::read(path).map_err(|e| format!("{}: {e}", path.display()))?;
    let bpb = bpb::parse(&bytes).map_err(|e| format!("{}: {e}", path.display()))?;
    let bad = bpb.bad_hashes();
    for b in &bad {
        eprintln!("bpb: {}: hash {b}", path.display());
    }
    Ok((bpb.to_ini(), bad.is_empty()))
}

fn tree(dir: &Path, out: &Path) -> Result<bool, String> {
    let mut files = Vec::new();
    walk(dir, &mut files)?;
    files.sort();
    let (mut ok, mut n) = (true, 0);
    for f in &files {
        let rel = f.strip_prefix(dir).expect("under the root");
        let dest = out.join(rel).with_extension("");
        let (ini, good) = decode(f)?;
        if let Some(parent) = dest.parent() {
            std::fs::create_dir_all(parent).map_err(|e| format!("{}: {e}", parent.display()))?;
        }
        std::fs::write(&dest, ini).map_err(|e| format!("{}: {e}", dest.display()))?;
        ok &= good;
        n += 1;
    }
    eprintln!("bpb: {n} files into {}", out.display());
    Ok(ok)
}

fn walk(dir: &Path, out: &mut Vec<PathBuf>) -> Result<(), String> {
    for e in std::fs::read_dir(dir).map_err(|e| format!("{}: {e}", dir.display()))? {
        let p = e.map_err(|e| format!("{}: {e}", dir.display()))?.path();
        if p.is_dir() {
            walk(&p, out)?;
        } else if p.extension().is_some_and(|x| x.eq_ignore_ascii_case("bpb")) {
            out.push(p);
        }
    }
    Ok(())
}
