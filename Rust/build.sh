#!/usr/bin/env bash
# build.sh — compile elfina (x86-64) and elfina32 (x86)
#            both fully static, no external dependencies
#
# Usage:
#   ./build.sh           build both, pack into elfina-linux.tar.gz
#   ./build.sh --no-pack build both, skip packaging
#
# Requirements:
#   rustup
#   musl-tools  →  sudo apt install musl-tools

set -euo pipefail

TARGET64="x86_64-unknown-linux-musl"   # static x86-64, no glibc
TARGET32="i686-unknown-linux-musl"     # static x86,    no glibc

ARCHIVE="elfina-linux.tar.gz"
STAGE="elfina"
NO_PACK=0

# ------------------------------------------------------------------ #
# Parse arguments                                                      #
# ------------------------------------------------------------------ #
for arg in "$@"; do
    case "$arg" in
        --no-pack) NO_PACK=1 ;;
        *) echo "Unknown argument: $arg"; exit 1 ;;
    esac
done

# ------------------------------------------------------------------ #
# Sanity checks                                                        #
# ------------------------------------------------------------------ #
if ! command -v cargo &>/dev/null; then
    echo "ERROR: cargo not found. Install from https://rustup.rs"
    exit 1
fi

if ! command -v musl-gcc &>/dev/null; then
    echo "ERROR: musl-gcc not found."
    echo "Install with: sudo apt install musl-tools"
    exit 1
fi

echo "Rust  : $(rustc --version)"
echo "Cargo : $(cargo --version)"
echo ""

# ------------------------------------------------------------------ #
# Add rustup targets if not already present                            #
# ------------------------------------------------------------------ #
for TARGET in "$TARGET64" "$TARGET32"; do
    if ! rustup target list --installed | grep -q "$TARGET"; then
        echo "Adding rustup target: $TARGET"
        rustup target add "$TARGET"
    fi
done

# ------------------------------------------------------------------ #
# Build x86-64 — fully static via musl                                #
# Result: single binary, no libc dependency, runs on any x86-64 Linux #
# ------------------------------------------------------------------ #
echo "=== Building elfina (x86-64, static) ==="
cargo build --release --target "$TARGET64"
echo ""

# ------------------------------------------------------------------ #
# Build x86 — fully static via musl                                   #
# Result: single binary, no libc dependency, runs on any x86 Linux    #
# ------------------------------------------------------------------ #
echo "=== Building elfina32 (x86, static) ==="
cargo build --release --target "$TARGET32"
echo ""

# ------------------------------------------------------------------ #
# Collect binaries into bin/                                           #
# ------------------------------------------------------------------ #
mkdir -p bin
cp "target/$TARGET64/release/elfina" bin/elfina
cp "target/$TARGET32/release/elfina" bin/elfina32

echo "=== Output ==="
file bin/elfina bin/elfina32
echo ""
echo "Dependencies:"
ldd bin/elfina   || echo "  elfina:   statically linked (no deps)"
ldd bin/elfina32 || echo "  elfina32: statically linked (no deps)"
echo ""

# ------------------------------------------------------------------ #
# Pack into tar.gz                                                     #
# ------------------------------------------------------------------ #
if [ "$NO_PACK" -eq 1 ]; then
    echo "Skipping packaging (--no-pack)"
    exit 0
fi

echo "=== Packing ${ARCHIVE} ==="
rm -rf "${STAGE}"
mkdir -p "${STAGE}"

cp bin/elfina   "${STAGE}/elfina"
cp bin/elfina32 "${STAGE}/elfina32"
[ -f "README.md" ] && cp README.md "${STAGE}/README.md"

tar -czf "${ARCHIVE}" "${STAGE}"
rm -rf "${STAGE}"

echo ""
echo "Done!"
echo "  ${ARCHIVE}  ($(du -sh "${ARCHIVE}" | cut -f1))"
echo ""
echo "Contents:"
tar -tzf "${ARCHIVE}"