#!/usr/bin/env bash
# Usage: ./release.sh VERSION
#   e.g. ./release.sh 1.2.3   or   ./release.sh v1.2.3
#
# Updates the version in all three places, commits, tags, and pushes.
# The GitHub Actions release workflow then builds and publishes installers.

set -euo pipefail

VERSION="${1:?Usage: ./release.sh VERSION (e.g. 1.2.3)}"
VERSION="${VERSION#v}"   # strip leading 'v' if provided

if ! [[ "$VERSION" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
  echo "Error: VERSION must be MAJOR.MINOR.PATCH (got: '$VERSION')" >&2
  exit 1
fi

TAG="v${VERSION}"

# Refuse to re-tag an existing tag
if git tag --list "$TAG" | grep -q .; then
  echo "Error: tag $TAG already exists" >&2
  exit 1
fi

echo "==> Bumping version to $VERSION"

# 1. package.json
python3 - <<PYEOF
import json
with open('package.json') as f:
    d = json.load(f)
d['version'] = '${VERSION}'
with open('package.json', 'w') as f:
    json.dump(d, f, indent=2)
    f.write('\n')
print('  package.json')
PYEOF

# 2. src-tauri/Cargo.toml  (replace only the [package] version on line 3)
python3 - <<PYEOF
import re
with open('src-tauri/Cargo.toml') as f:
    text = f.read()
# Replace the first occurrence of version = "..." (the [package] version)
text = re.sub(r'^version = "[^"]*"', 'version = "${VERSION}"', text, count=1, flags=re.MULTILINE)
with open('src-tauri/Cargo.toml', 'w') as f:
    f.write(text)
print('  src-tauri/Cargo.toml')
PYEOF

# 3. src-tauri/tauri.conf.json
python3 - <<PYEOF
import json
with open('src-tauri/tauri.conf.json') as f:
    d = json.load(f)
d['version'] = '${VERSION}'
with open('src-tauri/tauri.conf.json', 'w') as f:
    json.dump(d, f, indent=2)
    f.write('\n')
print('  src-tauri/tauri.conf.json')
PYEOF

echo ""
echo "==> Changes:"
git diff --stat

echo ""
read -rp "Commit, tag $TAG, and push? [y/N] " confirm
if [[ "$confirm" != "y" && "$confirm" != "Y" ]]; then
  echo "Aborted. Changes left unstaged."
  exit 0
fi

git add package.json src-tauri/Cargo.toml src-tauri/tauri.conf.json
git commit -m "chore: release ${TAG}"
git tag "${TAG}"
git push
git push origin "${TAG}"

echo ""
echo "Done! Tag ${TAG} pushed."
echo "GitHub Actions release workflow is now building installers."
