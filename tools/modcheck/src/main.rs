//! `modcheck`: prove built mod folders against the standard (§5). `modcheck --help` for usage.

use modcheck::{Discs, check, find_mods};
use std::path::PathBuf;
use std::process::ExitCode;

const USAGE: &str = "modcheck: prove mod folders against standard/OPENMX-TRACK-STANDARD.md §5

  modcheck [--iso DIR] [--quiet] PATH...

PATH is a mod folder (it has a mod.json) or a folder above some: out/mods, out/mods/tracks.
DIR holds the disc images a mod's source.boot is found in (default iso/, $DISCX_ISO overrides);
a disc mod whose disc is not there fails, since its copy cannot be proved.
Every mod prints `ok` or `FAIL`, then its fails and notes (--quiet: fails only). Exit 1 if any failed.";

fn main() -> ExitCode {
    let mut args: Vec<String> = std::env::args().skip(1).collect();
    if args.is_empty() || args.iter().any(|a| a == "-h" || a == "--help" || a == "help") {
        println!("{USAGE}");
        return ExitCode::SUCCESS;
    }
    let mut iso = std::env::var_os("DISCX_ISO").map(PathBuf::from).unwrap_or_else(|| PathBuf::from("iso"));
    if let Some(k) = args.iter().position(|a| a == "--iso") {
        args.remove(k);
        if k >= args.len() {
            eprintln!("modcheck: --iso needs a folder");
            return ExitCode::FAILURE;
        }
        iso = PathBuf::from(args.remove(k));
    }
    let quiet = args.iter().any(|a| a == "--quiet");
    args.retain(|a| a != "--quiet");

    let mut discs = Discs::new(Some(iso));
    let (mut ok, mut failed) = (0, 0);
    for root in &args {
        let mods = find_mods(std::path::Path::new(root));
        if mods.is_empty() {
            println!("modcheck: no mod folders under {root}");
        }
        for dir in mods {
            let r = check(&dir, &mut discs);
            println!("{} {}", if r.ok() { "ok  " } else { "FAIL" }, dir.display());
            for f in &r.fail {
                println!("    fail: {f}");
            }
            if !quiet {
                for n in &r.note {
                    println!("    note: {n}");
                }
            }
            if r.ok() { ok += 1 } else { failed += 1 }
        }
    }
    println!("modcheck: {ok} ok, {failed} failed");
    if failed > 0 { ExitCode::FAILURE } else { ExitCode::SUCCESS }
}
