#!/bin/sh
# Builds inputmodule-control into the plugin, so the repository works as a
# local Claude Code marketplace. With an output directory, also copies the
# finished marketplace there, ready to publish as its own git repository.
#
#   plugin/build.sh [OUTPUT_DIR]
#
# It builds for this machine's architecture only. With podman or docker, it
# builds in a Debian 12 container, so the binary runs with glibc 2.36 or newer,
# like the one committed in the plugin. Without either, or with NATIVE=1, it
# builds here, and the binary needs this machine's glibc or newer. Either way
# it needs libudev.
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
target=$arch-unknown-linux-gnu
engine=
if [ -z "${NATIVE:-}" ]; then
    engine=$(command -v podman || command -v docker || true)
fi
if [ -n "$engine" ]; then
    # Its own target folder, so it doesn't mix with builds for this machine.
    build=target/plugin-bookworm
    mkdir -p "$build"
    "$engine" run --rm -v "$PWD:/src:ro,Z" -v "$PWD/$build:/target:Z" \
        -e CARGO_TARGET_DIR=/target -w /src docker.io/library/rust:1-bookworm \
        sh -c "apt-get update -qq >/dev/null &&
            apt-get install -y -qq libudev-dev pkg-config >/dev/null &&
            cargo build --release --locked -p inputmodule-control --target $target"
else
    echo "Building for this machine's glibc. Install podman or docker for a build that runs on older ones." >&2
    build=target
    cargo build --release --locked -p inputmodule-control --target "$target"
fi
# Rename over the old one, which the display may be running from.
tmp="$libexec/.inputmodule-control-$arch.tmp"
cp "$build/$target/release/inputmodule-control" "$tmp"
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
