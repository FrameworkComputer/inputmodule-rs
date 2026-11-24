#!/bin/sh
# Builds inputmodule-control into the plugin, so plugin/ works as a local
# Claude Code marketplace. With an output directory, also copies the finished
# marketplace there, ready to publish as its own git repository.
#
#   plugin/build.sh [OUTPUT_DIR]
#
# It builds for this machine's architecture only. The binary needs libudev and
# this machine's glibc or newer.
set -eu
out=
if [ $# -gt 0 ]; then
    mkdir -p "$1"
    out=$(cd "$1" && pwd -P)
fi
cd "$(dirname "$0")/.."
libexec=plugin/ledmatrix/libexec
mkdir -p "$libexec"

[ "$(uname -s)" = Linux ] || { echo "The plugin only runs on Linux" >&2; exit 1; }
arch=$(uname -m)
# The workspace builds for the firmware's target by default.
target=$(rustc -vV | sed -n 's/^host: //p')
cargo build --release -p inputmodule-control --target "$target"
cp "target/$target/release/inputmodule-control" "$libexec/inputmodule-control-$arch"
# The release profile keeps debug info, which the plugin doesn't need.
if command -v strip >/dev/null 2>&1; then
    strip "$libexec/inputmodule-control-$arch"
fi
chmod +x "$libexec"/inputmodule-control-*
ls -l "$libexec"

[ -n "$out" ] || exit 0
rm -rf "$out/ledmatrix"
cp -R plugin/.claude-plugin plugin/ledmatrix "$out"/
cp plugin/README.md LICENSE "$out"/
cp LICENSE "$out/ledmatrix/"
echo "marketplace copied to $out"
