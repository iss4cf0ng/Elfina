set -euo pipefail

# Config
STAGE="elfina"
ARCHIVE="elfina-linux.tar.gz"

# Build
echo "=== Building elfina ==="
make clean
make

# Check
if [ ! -f "bin/elfina" ]; then
    echo "ERROR: bin/elfina not found — build failed."
    exit 1
fi

# Stage
echo ""
echo "=== Staging ${STAGE}/ ==="
rm -rf "${STAGE}"
mkdir -p "${STAGE}"

cp bin/elfina "${STAGE}/elfina"
echo "  + elfina (x86-64)"

if [ -f "bin/elfina32" ]; then
    cp bin/elfina32 "${STAGE}/elfina32"
    echo "  + elfina32 (x86)"
else
    echo "  - elfina32 not found (install gcc-multilib to include it)"
fi

[ -f "README.md" ] && cp README.md "${STAGE}/README.md" && echo "  + README.md"

# Packing
echo ""
echo "=== Packing ${ARCHIVE} ==="
tar -czf "${ARCHIVE}" "${STAGE}"
rm -rf "${STAGE}"

# Summary
echo ""
echo "Done!"
echo "  ${ARCHIVE}  ($(du -sh "${ARCHIVE}" | cut -f1))"
echo ""
echo "Contents:"
tar -tzf "${ARCHIVE}"