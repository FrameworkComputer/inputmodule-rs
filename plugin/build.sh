#!/bin/sh
# Builds inputmodule-control into the plugin, so the repository works as a
# local Claude Code marketplace. With an output directory, also copies the
# finished marketplace there, ready to publish as its own git repository.
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
# Rename over the old one, which the display may be running from.
tmp="$libexec/.inputmodule-control-$arch.tmp"
cp "target/$target/release/inputmodule-control" "$tmp"
# The release profile keeps debug info, which the plugin doesn't need.
if command -v strip >/dev/null 2>&1; then
    strip "$tmp"
fi
mv -f "$tmp" "$libexec/inputmodule-control-$arch"
chmod +x "$libexec"/inputmodule-control-*
ls -l "$libexec"

[ -n "$out" ] || exit 0
rm -rf "$out/ledmatrix"
cp -R plugin/ledmatrix "$out"/
mkdir -p "$out/.claude-plugin"
# The plugin is at the top of the published copy.
sed 's|"\./plugin/ledmatrix"|"./ledmatrix"|' .claude-plugin/marketplace.json \
    >"$out/.claude-plugin/marketplace.json"
cp plugin/README.md LICENSE "$out"/
cp LICENSE "$out/ledmatrix/"
echo "marketplace copied to $out"
