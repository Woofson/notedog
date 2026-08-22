#!/usr/bin/env bash
set -e

echo "🚀 Preparing NoteDog AUR Release (v0.5.0)..."

REPO_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
TMP_DIR=$(mktemp -d)

echo "📦 Cloning AUR repository..."
git clone ssh://aur@aur.archlinux.org/notedog.git "$TMP_DIR"

echo "📄 Copying PKGBUILD and .SRCINFO..."
cp "$REPO_DIR/PKGBUILD" "$REPO_DIR/.SRCINFO" "$TMP_DIR/"

cd "$TMP_DIR"
git branch -m master 2>/dev/null || true
git add PKGBUILD .SRCINFO
git commit -m "Update notedog to 0.5.0" || true
git push -u origin master

echo "🎉 NoteDog v0.5.0 successfully published to AUR!"
rm -rf "$TMP_DIR"
