#!/usr/bin/env bash
# Update packaging/aur/rcloud-bin/PKGBUILD to a released version.
# Fetches the published .sha256 sidecars, rewrites pkgver / pkgrel / sha256sums,
# and regenerates .SRCINFO (when makepkg is available).
#
# Usage: update-aur-bin.sh <version-without-v>   e.g. update-aur-bin.sh 0.5.0
set -euo pipefail

version="${1:?usage: update-aur-bin.sh <version>}"
version="${version#v}"

repo="MauroGonzalez51/rust-rcloud"
base="https://github.com/${repo}/releases/download/v${version}"
dir="$(cd "$(dirname "${BASH_SOURCE[0]}")/../aur/rcloud-bin" && pwd)"
pkgbuild="${dir}/PKGBUILD"

fetch_sha() {
    # $1 = rust target triple. Prints the bare hex digest.
    local triple="$1"
    curl -fsSL "${base}/rcloud-${triple}-unknown-linux-gnu.tar.xz.sha256" \
        | awk '{print $1}'
}

sha_x86_64="$(fetch_sha x86_64)"
sha_aarch64="$(fetch_sha aarch64)"

[[ "${sha_x86_64}"  =~ ^[0-9a-f]{64}$ ]] || { echo "bad x86_64 sha: ${sha_x86_64}" >&2; exit 1; }
[[ "${sha_aarch64}" =~ ^[0-9a-f]{64}$ ]] || { echo "bad aarch64 sha: ${sha_aarch64}" >&2; exit 1; }

sed -i \
    -e "s/^pkgver=.*/pkgver=${version}/" \
    -e "s/^pkgrel=.*/pkgrel=1/" \
    -e "s/^sha256sums_x86_64=.*/sha256sums_x86_64=('${sha_x86_64}')/" \
    -e "s/^sha256sums_aarch64=.*/sha256sums_aarch64=('${sha_aarch64}')/" \
    "${pkgbuild}"

echo "Updated ${pkgbuild} -> ${version}"
echo "  x86_64:  ${sha_x86_64}"
echo "  aarch64: ${sha_aarch64}"

if command -v makepkg >/dev/null 2>&1; then
    (cd "${dir}" && makepkg --printsrcinfo > .SRCINFO)
    echo "Regenerated ${dir}/.SRCINFO"
else
    echo "makepkg not found; skipping .SRCINFO regeneration" >&2
fi
