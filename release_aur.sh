#!/usr/bin/env bash
set -e

REPO_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PKGVER=$(grep '^pkgver=' "$REPO_DIR/PKGBUILD" | cut -d= -f2)

echo "🚀 Preparing NoteDog AUR Release (v$PKGVER)..."

TMP_DIR=$(mktemp -d)

echo "📦 Cloning AUR repository..."
git clone ssh://aur@aur.archlinux.org/notedog.git "$TMP_DIR"

echo "📄 Copying PKGBUILD and .SRCINFO..."
cp "$REPO_DIR/PKGBUILD" "$REPO_DIR/.SRCINFO" "$TMP_DIR/"

cd "$TMP_DIR"
git branch -m master 2>/dev/null || true
git add PKGBUILD .SRCINFO
git commit -m "Update notedog to $PKGVER" || true
git push -u origin master

echo "🎉 NoteDog v$PKGVER successfully published to AUR!"
rm -rf "$TMP_DIR"
