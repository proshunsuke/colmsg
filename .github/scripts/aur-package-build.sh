#!/usr/bin/env bash
set -euo pipefail

if [[ $# -lt 3 || $# -gt 4 ]]; then
  echo "Usage: $0 <aur-directory> <version> <source-spec> [builder-uid]" >&2
  exit 2
fi

aur_dir=$1
version=$2
source_spec=$3
builder_uid=${4:-}

cd "$aur_dir"

sed -i "s/^pkgver=.*/pkgver=$version/" PKGBUILD
sed -i "s|^source=.*$|source=(\"$source_spec\")|" PKGBUILD
sed -i "s/^sha256sums=.*/sha256sums=('SKIP')/" PKGBUILD

# makepkg's default C/C++ LTO can break AWS-LC linking; keep Cargo's Rust LTO.
if ! grep -qF 'options+=( !lto )' PKGBUILD; then
  printf '\n# Disable makepkg C/C++ LTO; Cargo release LTO remains enabled.\noptions+=( !lto )\n' >> PKGBUILD
fi

bash -n PKGBUILD

if ! id builder >/dev/null 2>&1; then
  if [[ -n "$builder_uid" ]]; then
    useradd -m -u "$builder_uid" -g root builder
  else
    useradd -m builder
  fi
fi

if ! grep -qF 'builder ALL=(ALL) NOPASSWD: ALL' /etc/sudoers; then
  echo 'builder ALL=(ALL) NOPASSWD: ALL' >> /etc/sudoers
fi

builder_group=$(id -gn builder)
find . -name .git -prune -o -type l -prune -o -print0 | xargs -0 chown "builder:$builder_group"

echo "=== Updated PKGBUILD ==="
cat PKGBUILD
su builder -s /bin/bash -c "cd \"$PWD\" && makepkg --printsrcinfo > .SRCINFO"
echo "=== Generated .SRCINFO ==="
cat .SRCINFO
namcap PKGBUILD || true

su builder -s /bin/bash -c "cd \"$PWD\" && makepkg -sf --noconfirm"
ls -lh *.pkg.tar.*
namcap *.pkg.tar.* || true
