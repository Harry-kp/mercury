#!/bin/bash

# Mercury Release Automation Script
# Usage: ./scripts/release.sh [patch|minor|major]

set -e

# ANSI Colors
GREEN='\033[0;32m'
BLUE='\033[0;34m'
RED='\033[0;31m'
NC='\033[0m'

echo -e "${BLUE}🚀 Starting Release Process...${NC}"

# 1. Check Git Status
if [[ -n $(git status -s) ]]; then
    echo -e "${RED}❌ Error: Git working directory is not clean. Commit or stash changes first.${NC}"
    exit 1
fi

# 2. Determine Bump Type
BUMP_TYPE=$1
if [[ -z "$BUMP_TYPE" ]]; then
    echo "Select release type:"
    select type in "patch" "minor" "major"; do
        BUMP_TYPE=$type
        break
    done
fi

# 3. Get Current Version
CURRENT_VERSION=$(grep '^version =' Cargo.toml | cut -d '"' -f 2)
echo -e "Current Version: ${GREEN}$CURRENT_VERSION${NC}"

# 4. Calculate New Version
# Split version into array (semver)
IFS='.' read -r -a parts <<< "${CURRENT_VERSION//-beta/}" # remove -beta for calculation if present
major=${parts[0]}
minor=${parts[1]}
patch=${parts[2]}

case $BUMP_TYPE in
    major) major=$((major + 1)); minor=0; patch=0 ;;
    minor) minor=$((minor + 1)); patch=0 ;;
    patch) patch=$((patch + 1)) ;;
    *) echo -e "${RED}Invalid type${NC}"; exit 1 ;;
esac

NEW_VERSION="$major.$minor.$patch"
# Optional: Ask to append -beta
read -p "Append '-beta' suffix? (y/N) " -n 1 -r
echo
if [[ $REPLY =~ ^[Yy]$ ]]; then
    NEW_VERSION="${NEW_VERSION}-beta"
fi

echo -e "Target Version:  ${GREEN}$NEW_VERSION${NC}"
read -p "Continue? (y/N) " -n 1 -r
echo
if [[ ! $REPLY =~ ^[Yy]$ ]]; then
    echo "Aborted."
    exit 1
fi

# 5. Update Cargo.toml
echo "📝 Updating Cargo.toml..."
# generic sed for mac/linux compatibility
sed -i.bak "s/^version = \"$CURRENT_VERSION\"/version = \"$NEW_VERSION\"/" Cargo.toml
rm Cargo.toml.bak

# 6. Update CHANGELOG.md
echo "📝 Updating CHANGELOG.md..."
DATE=$(date +%Y-%m-%d)
HEADER="## [$NEW_VERSION] - $DATE"

# Replace [Unreleased] with New Version Header + New Unreleased Section
# We assume the changelog starts with:
# ## [Unreleased]
# ### Added...
TERM="## [Unreleased]"
REPLACEMENT="## [Unreleased]\n\n$HEADER"

# Use python for safer multi-line replacement than sed
python3 -c "
import sys
content = sys.stdin.read()
new_content = content.replace('## [Unreleased]', '$REPLACEMENT')
with open('CHANGELOG.md', 'w') as f:
    f.write(new_content)
" < CHANGELOG.md

echo -e "${GREEN}✅ Files updated.${NC}"

# 7. Git Commit & Tag
echo "📦 Committing and Tagging..."
git add Cargo.toml CHANGELOG.md
git commit -m "chore: release v$NEW_VERSION"
git tag -a "v$NEW_VERSION" -m "Release v$NEW_VERSION"

echo -e "${GREEN}✅ Tag v$NEW_VERSION created.${NC}"

# 8. Push Prompt
echo -e "${BLUE}Ready to push to GitHub? This will trigger the release build.${NC}"
read -p "Push now? (y/N) " -n 1 -r
echo
if [[ $REPLY =~ ^[Yy]$ ]]; then
    git push origin master
    git push origin "v$NEW_VERSION"
    echo -e "${GREEN}🚀 Released! Check GitHub Actions for build progress.${NC}"
else
    echo "Done (local only). Remember to push later: git push origin v$NEW_VERSION"
fi
