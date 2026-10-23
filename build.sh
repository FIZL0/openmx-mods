#!/usr/bin/env bash
# Build every mod the discs in iso/ (and the tools that need no disc) can give, into out/mods/, and
# with --check prove each one with openmx's readers (standard/OPENMX-TRACK-STANDARD.md §5).
#
#   ./build.sh                  import and package
#   ./build.sh --check          import, package, then check everything in out/mods/
#   ./build.sh --check-only     check out/mods/ as it is
#   options: --iso DIR (default iso/, or $DISCX_ISO)   --out DIR (default out/)
#            --install DIR (default: openmx/, where the port's disc and settings.txt are)   --no-install
#
# Every mod in out/mods/ is then copied into the game's own mods folder, INSTALL/mods/<dest>/<id>/, the
# folder openmx reads beside its disc and settings.txt (each id replaced whole; other mods there are left
# alone). `mods` in this repo is a link to out/mods/.
#
# Discs are found by their boot ELF (SYSTEM.CNF), never by file name: `discx discs`.
#
# How a tool is wired in: one line in STEPS below. A step runs only if its crate exists
# (tools/<crate>/Cargo.toml) and, for a disc step, only if that disc is in iso/. Each step is run as
#
#   <crate> [--disc IMAGE] --out STAGE [--mods OUT/mods]
#
# and writes one folder per mod, STAGE/<id>/ with its mod.json. build.sh then moves each folder to
# OUT/mods/<dest>/<id>/, replacing the old one whole, so nothing stale is left in it. --disc is given
# to disc steps, --mods (the mods built so far) to `after` steps. A step that fails is reported, its
# output is not packaged, and the run goes on; build.sh exits 1 at the end.
set -euo pipefail
cd "$(dirname "$0")"

# when   needs (boot ELF or -)  crate          dest     the task that writes the crate
STEPS='
disc     SLUS_211.04            import-mxvatv  tracks   M2
disc     SLUS_217.01            import-mxvatv  tracks   M4
disc     SLUS_211.04            import-music   music    A1
disc     SLUS_217.01            import-music   music    A1
disc     SLUS_211.04            import-tricks  tricks   M19
disc     SLUS_217.01            import-tricks  tricks   M19
data     -                      elevation      tracks   M1
after    -                      stitch         tracks   C1
'
# MX Unleashed (SLUS_207.67) has no step: its tracks are already the port's own, and a mod of one is
# only ever a world's tile (§1), which stitch copies from the disc itself.

check=0 import=1 install=openmx
iso=${DISCX_ISO:-iso}
out=out
while [ $# -gt 0 ]; do
    case $1 in
        --check) check=1 ;;
        --check-only) check=1 import=0 ;;
        --iso) iso=${2:?--iso needs a folder}; shift ;;
        --out) out=${2:?--out needs a folder}; shift ;;
        --install) install=${2:?--install needs a folder}; shift ;;
        --no-install) install="" ;;
        -h|--help) sed -n '2,8p' "$0" | sed 's/^# \{0,1\}//'; exit 0 ;;
        *) echo "build: unknown argument $1 (--help)" >&2; exit 2 ;;
    esac
    shift
done

# Data steps find a disc they borrow from themselves (elevation's skies, M17): the same folder.
export DISCX_ISO=$iso

if [ ! -d openmx/crates/game-sim ]; then
    echo "build: openmx is not linked: ln -s ../openmx openmx (see README)" >&2
    exit 1
fi

say() { echo "build: $*"; }
failed=0

# The binaries: discx and modcheck, and every step crate that exists.
crates=(discx modcheck)
while read -r when needs crate dest task; do
    [ -n "${when:-}" ] || continue
    if [ -f "tools/$crate/Cargo.toml" ] && [[ " ${crates[*]} " != *" $crate "* ]]; then
        crates+=("$crate")
    fi
done <<< "$STEPS"
# openmx's own warnings are not ours to read on every run: cargo's output only when it fails.
if ! log=$(cargo build --release $(printf -- '-p %s ' "${crates[@]}") 2>&1); then
    echo "$log" | tail -40 >&2
    echo "build: cargo build failed" >&2
    exit 1
fi
target=$(cargo metadata --format-version 1 --no-deps | sed -n 's/.*"target_directory":"\([^"]*\)".*/\1/p')
bin=$target/release

# The discs present, as "BOOT<tab>path".
discs=""
if [ -d "$iso" ]; then
    discs=$("$bin/discx" discs "$iso" | awk -F'\t' '{print $1 "\t" $3}')
fi
disc_for() { printf '%s\n' "$discs" | awk -F'\t' -v b="$1" 'toupper($1) == toupper(b) {print $2; exit}'; }

if [ $import = 1 ]; then
    if [ -z "$discs" ]; then
        say "no disc images in $iso/"
    else
        printf '%s\n' "$discs" | while IFS=$'\t' read -r boot path; do
            say "disc $boot: $path"
        done
    fi
    stage_root=$out/.stage
    rm -rf "$stage_root"
    n=0
    while read -r when needs crate dest task; do
        [ -n "${when:-}" ] || continue
        n=$((n + 1))
        what=$crate
        if [ "$needs" != - ]; then what="$crate ($needs)"; fi
        if [ ! -f "tools/$crate/Cargo.toml" ]; then
            say "skip $what: tools/$crate does not exist yet ($task)"
            continue
        fi
        args=()
        if [ "$when" = disc ]; then
            image=$(disc_for "$needs")
            if [ -z "$image" ]; then
                say "skip $what: no disc with boot ELF $needs in $iso/"
                continue
            fi
            args+=(--disc "$image")
        fi
        if [ "$when" = after ]; then args+=(--mods "$out/mods"); fi
        stage=$stage_root/$n-$crate
        mkdir -p "$stage"
        say "run $what"
        if ! "$bin/$crate" "${args[@]}" --out "$stage"; then
            say "FAIL $what: the importer failed; nothing of it packaged"
            failed=1
            continue
        fi
        mkdir -p "$out/mods/$dest"
        got=0
        for d in "$stage"/*/; do
            [ -d "$d" ] || continue
            id=$(basename "$d")
            if [ ! -f "$d/mod.json" ]; then
                say "FAIL $what: $id has no mod.json; not packaged"
                failed=1
                continue
            fi
            rm -rf "${out:?}/mods/$dest/$id"
            mv "$d" "$out/mods/$dest/$id"
            got=$((got + 1))
        done
        say "$what: $got mods into $out/mods/$dest/"
    done <<< "$STEPS"
    rm -rf "$stage_root"
fi

if [ $check = 1 ]; then
    if [ -d "$out/mods" ]; then
        "$bin/modcheck" --iso "$iso" "$out/mods" || failed=1
    else
        say "nothing to check: no $out/mods/"
    fi
fi
# Into the game's mods folder: openmx's store is the folder holding its disc (or settings.txt).
if [ -n "$install" ] && [ -d "$out/mods" ]; then
    if compgen -G "$install/*.iso" >/dev/null || [ -f "$install/settings.txt" ]; then
        dir=$(cd "$install" && pwd -P)/mods
        got=0
        for d in "$out"/mods/*/*/; do
            [ -f "$d/mod.json" ] || continue
            rel=${d#"$out/mods/"}
            rel=${rel%/}
            mkdir -p "$dir/$(dirname "$rel")"
            rm -rf "${dir:?}/$rel"
            cp -a --reflink=auto "$d" "$dir/$rel"
            got=$((got + 1))
        done
        say "installed $got mods into $dir/"
    else
        say "not installed: $install/ holds no disc or settings.txt (--install DIR, the folder openmx runs from)"
    fi
fi
exit $failed
