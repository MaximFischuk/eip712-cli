#!/usr/bin/env bash
# Usage: ./scripts/update-formula.sh <version> [formula_path]
# Example: ./scripts/update-formula.sh 1.0.0
# Example: ./scripts/update-formula.sh 1.0.0 ../homebrew-eip712-cli/Formula/eip712-cli.rb
#
# Downloads sha256sums.txt from the GitHub release and updates the
# Homebrew formula with the correct version and checksums.

set -euo pipefail

VERSION="${1:?Usage: $0 <version> [formula_path]}"
FORMULA="${2:-Formula/eip712-cli.rb}"
REPO="MaximFischuk/eip712-cli"
BASE_URL="https://github.com/${REPO}/releases/download/v${VERSION}"

TMPDIR=$(mktemp -d)
trap 'rm -rf "$TMPDIR"' EXIT

echo "Updating formula to version ${VERSION}..."

# Download checksums from the release (no need to download full binaries)
echo "  Downloading sha256sums.txt..."
curl -fsSL -o "${TMPDIR}/sha256sums.txt" "${BASE_URL}/sha256sums.txt"

SHA_DARWIN_ARM64=$(grep "aarch64-apple-darwin" "${TMPDIR}/sha256sums.txt" | awk '{print $1}')
SHA_DARWIN_X86=$(grep "x86_64-apple-darwin" "${TMPDIR}/sha256sums.txt" | awk '{print $1}')
SHA_LINUX_X86=$(grep "x86_64-unknown-linux-gnu" "${TMPDIR}/sha256sums.txt" | awk '{print $1}')
SHA_LINUX_ARM64=$(grep "aarch64-unknown-linux-gnu" "${TMPDIR}/sha256sums.txt" | awk '{print $1}')

echo "  aarch64-apple-darwin:      ${SHA_DARWIN_ARM64}"
echo "  x86_64-apple-darwin:       ${SHA_DARWIN_X86}"
echo "  x86_64-unknown-linux-gnu:  ${SHA_LINUX_X86}"
echo "  aarch64-unknown-linux-gnu: ${SHA_LINUX_ARM64}"

sed -i.bak "s|version \".*\"|version \"${VERSION}\"|" "$FORMULA"
sed -i.bak "s|sha256 \"[a-f0-9]*\" # aarch64-apple-darwin|sha256 \"${SHA_DARWIN_ARM64}\" # aarch64-apple-darwin|" "$FORMULA"
sed -i.bak "s|sha256 \"[a-f0-9]*\" # x86_64-apple-darwin|sha256 \"${SHA_DARWIN_X86}\" # x86_64-apple-darwin|" "$FORMULA"
sed -i.bak "s|sha256 \"[a-f0-9]*\" # x86_64-unknown-linux-gnu|sha256 \"${SHA_LINUX_X86}\" # x86_64-unknown-linux-gnu|" "$FORMULA"
sed -i.bak "s|sha256 \"[a-f0-9]*\" # aarch64-unknown-linux-gnu|sha256 \"${SHA_LINUX_ARM64}\" # aarch64-unknown-linux-gnu|" "$FORMULA"

rm -f "${FORMULA}.bak"

echo "Formula updated successfully."
echo "Don't forget to commit and push the changes."
