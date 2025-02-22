#!/bin/bash

# Check if version parameter is provided
if [ -z "$1" ]; then
    echo "Error: Please provide a version tag (e.g., v0.2.0)"
    echo "Usage: $0 <version>"
    exit 1
fi

# Assign version from parameter
VERSION=$1

# Validate version format (e.g., vX.Y.Z)
if ! [[ $VERSION =~ ^v[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
    echo "Error: Version must follow the format vX.Y.Z (e.g., v0.2.0)"
    exit 1
fi

# Strip 'v' from version for Cargo.toml
CARGO_VERSION=${VERSION#v}

# Ensure we're on develop branch
CURRENT_BRANCH=$(git rev-parse --abbrev-ref HEAD)
if [ "$CURRENT_BRANCH" != "develop" ]; then
    echo "Error: Must be on 'develop' branch to start release. Current branch: $CURRENT_BRANCH"
    exit 1
fi

# Ensure working directory is clean
if [ -n "$(git status --porcelain)" ]; then
    echo "Error: Working directory is not clean. Please commit or stash changes."
    exit 1
fi

# Check if Cargo.toml exists
if [ ! -f "Cargo.toml" ]; then
    echo "Error: Cargo.toml not found in current directory"
    exit 1
fi

# Update version in Cargo.toml
echo "Updating Cargo.toml version to $CARGO_VERSION..."
sed -i '' "s/version = \"[0-9]*\.[0-9]*\.[0-9]*\"/version = \"$CARGO_VERSION\"/" Cargo.toml

# Verify the change
if ! grep -q "version = \"$CARGO_VERSION\"" Cargo.toml; then
    echo "Error: Failed to update version in Cargo.toml"
    exit 1
fi

# Update Cargo.lock by running a cargo command (if Cargo.lock exists and is tracked)
if [ -f "Cargo.lock" ] && git ls-files --error-unmatch Cargo.lock >/dev/null 2>&1; then
    echo "Updating Cargo.lock..."
    cargo check  # This regenerates Cargo.lock based on the new version
fi

# Commit both Cargo.toml and Cargo.lock (if modified)
echo "Committing Cargo.toml and Cargo.lock (if updated)..."
git add Cargo.toml
if [ -f "Cargo.lock" ] && [ -n "$(git status --porcelain Cargo.lock)" ]; then
    git add Cargo.lock
fi
git commit -m "Bump version to $CARGO_VERSION for release"

# Update local branches
git fetch origin

# Checkout main and merge develop into main
echo "Merging develop into main..."
git checkout main
git merge --no-ff develop -m "Merge branch 'develop' into main for release $VERSION"

# Tag the release
echo "Creating tag $VERSION..."
git tag -a "$VERSION" -m "Release $VERSION"

# Push main and tags to remote
echo "Pushing main and tags to origin..."
git push origin main
git push origin "$VERSION"

# Switch back to develop
echo "Switching back to develop..."
git checkout develop

echo "Release $VERSION completed successfully!"