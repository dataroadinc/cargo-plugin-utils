#!/bin/bash
set -euo pipefail

# Repository migration mapping: repo_name -> source_organization
get_source_org() {
  case "$1" in
    "cargo-version-info"|"dotenvage"|"cargo-fmt-toml"|"cargo-propagate-features"|"cargo-plugin-utils")
      echo "agnos-ai"
      ;;
    "ekg-rs")
      echo "EKGF"
      ;;
    *)
      echo "unknown"
      ;;
  esac
}

REPOS=(
  "cargo-version-info"
  "dotenvage"
  "cargo-fmt-toml"
  "cargo-propagate-features"
  "cargo-plugin-utils"
  "ekg-rs"
)

NEW_ORG="dataroadinc"
WORK_DIR="/Users/jgeluk/Work"

echo "Starting migration to $NEW_ORG organization"

# Check if GitHub CLI is available
if ! command -v gh &> /dev/null; then
  echo "ERROR: GitHub CLI (gh) not found. Please install it to enable automated transfers."
  echo "Install: brew install gh (or see https://cli.github.com/)"
  exit 1
fi

# Check GitHub CLI authentication
if ! gh auth status &>/dev/null; then
  echo "WARNING: GitHub CLI not authenticated. Please run: gh auth login"
  exit 1
fi

for repo in "${REPOS[@]}"; do
  OLD_ORG=$(get_source_org "$repo")
  echo ""
  echo "Processing $repo (from $OLD_ORG to $NEW_ORG)..."
  cd "$WORK_DIR/$repo" || { echo "ERROR: Failed to cd to $repo"; exit 1; }

  # 1. Transfer GitHub repository (if not already transferred)
  echo "  Transferring GitHub repository..."
  if gh repo view "$NEW_ORG/$repo" &>/dev/null; then
    echo "  Repository already in $NEW_ORG"
  else
    # Check if repo exists in old org before attempting transfer
    if gh repo view "$OLD_ORG/$repo" &>/dev/null; then
      if gh api -X POST "repos/$OLD_ORG/$repo/transfer" -f "new_owner=$NEW_ORG" &>/dev/null; then
        echo "  Repository transfer initiated"
        echo "  Waiting 5 seconds for transfer to process..."
        sleep 5
        # Verify transfer succeeded
        if gh repo view "$NEW_ORG/$repo" &>/dev/null; then
          echo "  Transfer confirmed successful"
        else
          echo "  WARNING: Transfer may still be processing. Verify manually if needed."
        fi
      else
        echo "  ERROR: Transfer API call failed. Check permissions and try again."
        echo "  Required: Admin access to $OLD_ORG/$repo and create repo permission in $NEW_ORG"
      fi
    else
      echo "  WARNING: Repository not found in $OLD_ORG. May already be transferred or renamed."
    fi
  fi

  # 2. Update file contents (replace old org with new org)
  echo "  Updating file contents..."
  find . -type f \( -name "Cargo.toml" -o -name "cog.toml" -o -name "*.md" -o -name "*.yml" -o -name "*.yaml" \) \
    -not -path "./.git/*" \
    -not -path "./target/*" \
    -not -path "./CHANGELOG.md" \
    -exec sed -i '' "s|github.com/$OLD_ORG|github.com/$NEW_ORG|g" {} +

  # 3. Update git remote
  echo "  Updating git remote..."
  git remote set-url origin "https://github.com/$NEW_ORG/$repo.git" || true

  # 4. Verify no old references remain (excluding CHANGELOG.md which is historical)
  echo "  Verifying changes..."
  if grep -r "github.com/$OLD_ORG" --exclude-dir=.git --exclude-dir=target --exclude="CHANGELOG.md" . 2>/dev/null; then
    echo "  WARNING: Some $OLD_ORG references may remain (excluding CHANGELOG.md)"
  else
    echo "  No $OLD_ORG references found (excluding CHANGELOG.md)"
  fi

  # 5. Verify new references exist
  if grep -r "$NEW_ORG" --exclude-dir=.git --exclude-dir=target . 2>/dev/null | head -1 > /dev/null; then
    echo "  $NEW_ORG references confirmed"
  else
    echo "  ERROR: No $NEW_ORG references found - something went wrong"
  fi

  # 6. Test build (quick check)
  echo "  Testing build..."
  if cargo check --quiet 2>/dev/null; then
    echo "  Build successful"
  else
    echo "  WARNING: Build check skipped or failed (non-critical)"
  fi

  echo "  Completed $repo"
done

# 7. Verify GitHub repository locations
echo ""
echo "Verifying GitHub repository locations..."
if command -v gh &> /dev/null; then
  for repo in "${REPOS[@]}"; do
    if gh repo view "$NEW_ORG/$repo" &>/dev/null; then
      echo "  $repo is in $NEW_ORG"
    else
      echo "  WARNING: $repo not found in $NEW_ORG (may need manual transfer)"
    fi
  done
else
  echo "  WARNING: GitHub CLI not found - skipping repo verification"
fi

# 8. Generate crates.io update report
echo ""
echo "Generating crates.io update report..."
cat > /tmp/crates-io-updates.md <<EOF
# crates.io Metadata Updates Required

The following crates need their repository URLs updated on crates.io:

EOF

for repo in "${REPOS[@]}"; do
  crate_name=$(basename "$repo")
  echo "- **$crate_name**" >> /tmp/crates-io-updates.md
  echo "  - Current: https://crates.io/crates/$crate_name" >> /tmp/crates-io-updates.md
  echo "  - Update repository to: https://github.com/$NEW_ORG/$repo" >> /tmp/crates-io-updates.md
  echo "  - Edit URL: https://crates.io/crates/$crate_name/settings" >> /tmp/crates-io-updates.md
  echo "" >> /tmp/crates-io-updates.md
done

echo "  Report saved to /tmp/crates-io-updates.md"
echo ""
echo "Migration complete!"
echo ""
echo "Next steps:"
echo "  1. Review changes: git diff in each repository"
echo "  2. If repos not yet transferred, transfer them on GitHub"
echo "  3. Update crates.io metadata (see /tmp/crates-io-updates.md)"
echo "  4. Commit and push changes"