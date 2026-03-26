#!/usr/bin/env bash
#
# Stages platform binaries into npm package directories and publishes all packages.
# Usage: publish-npm.sh <version> <artifacts-dir>
#
set -euo pipefail

VERSION="$1"
ARTIFACTS_DIR="$2"
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
NPM_DIR="$ROOT_DIR/npm"

# Map: target triple -> npm package directory
declare -A TARGET_MAP=(
  ["aarch64-apple-darwin"]="cli-darwin-arm64"
  ["x86_64-apple-darwin"]="cli-darwin-x64"
  ["x86_64-unknown-linux-gnu"]="cli-linux-x64"
  ["aarch64-unknown-linux-gnu"]="cli-linux-arm64"
)

echo "Publishing openproof v${VERSION} to npm..."

# Stage binaries from tarballs into npm package dirs
for target in "${!TARGET_MAP[@]}"; do
  pkg_dir="${TARGET_MAP[$target]}"
  tarball="${ARTIFACTS_DIR}/openproof-v${VERSION}-${target}.tar.gz"

  if [ ! -f "$tarball" ]; then
    echo "Error: missing tarball ${tarball}" >&2
    exit 1
  fi

  echo "Staging ${target} -> npm/${pkg_dir}/"
  tmpdir="$(mktemp -d)"
  tar xzf "$tarball" -C "$tmpdir"
  cp "$tmpdir/openproof-v${VERSION}-${target}/openproof" "$NPM_DIR/$pkg_dir/bin/openproof"
  chmod +x "$NPM_DIR/$pkg_dir/bin/openproof"
  rm -rf "$tmpdir"
done

# Update version in all package.json files
for pkg_dir in cli-darwin-arm64 cli-darwin-x64 cli-linux-x64 cli-linux-arm64 openproof; do
  pkg_json="$NPM_DIR/$pkg_dir/package.json"
  # Use node for reliable JSON editing
  node -e "
    const fs = require('fs');
    const pkg = JSON.parse(fs.readFileSync('$pkg_json', 'utf8'));
    pkg.version = '$VERSION';
    if (pkg.optionalDependencies) {
      for (const key of Object.keys(pkg.optionalDependencies)) {
        pkg.optionalDependencies[key] = '$VERSION';
      }
    }
    fs.writeFileSync('$pkg_json', JSON.stringify(pkg, null, 2) + '\n');
  "
  echo "Updated ${pkg_dir}/package.json to v${VERSION}"
done

# Publish platform packages first
for pkg_dir in cli-darwin-arm64 cli-darwin-x64 cli-linux-x64 cli-linux-arm64; do
  echo "Publishing @openproof/${pkg_dir}..."
  cd "$NPM_DIR/$pkg_dir"
  npm publish --access public
done

# Publish main package last (so optional deps are already available)
echo "Publishing openproof..."
cd "$NPM_DIR/openproof"
npm publish --access public

echo "Published openproof v${VERSION} to npm."
