#!/usr/bin/env bash
# The test gate: `agent-runner done` and every merge run it.
set -euo pipefail
cd "$(dirname "$0")"

# No game data in git: nothing under the ignored folders, and no file over 1 MB.
bad=$(git ls-files -- iso extracted out | head -5)
if [ -n "$bad" ]; then
    echo "check: game data is tracked: $bad" >&2
    exit 1
fi
big=$(git ls-files -z | xargs -0 -r du -k -- 2>/dev/null | awk '$1 > 1024 {print $2}' | head -5)
if [ -n "$big" ]; then
    echo "check: files over 1 MB are tracked (game data?): $big" >&2
    exit 1
fi

if [ -f Cargo.toml ]; then
    if [ ! -d openmx/crates/game-sim ]; then
        echo "check: openmx is not linked: ln -s ../openmx openmx (see README)" >&2
        exit 1
    fi
    out=$(cargo test --workspace --quiet 2>&1) || { echo "$out" | tail -40; echo "check: FAILED" >&2; exit 1; }
    echo "$out" | grep -E "^test result" | awk '{p+=$4; f+=$6} END {print "check: ok, " p " passed, " f " failed"}'
else
    echo "check: ok (no Rust workspace yet)"
fi
