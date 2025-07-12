# Get the current version
VERSION=$(rg --no-filename '^version = "(.*)"' ./Cargo.toml --replace '$1')

# Increment the patch version
MAJOR=$(echo $VERSION | cut -d. -f1)
MINOR=$(echo $VERSION | cut -d. -f2)
PATCH=$(echo $VERSION | cut -d. -f3)
PATCH=$((PATCH + 1))
VERSION="$MAJOR.$MINOR.$PATCH"

# Update the version in Cargo.toml
sed -i "s/^version = \".*\"/version = \"$VERSION\"/" ./Cargo.toml
# cargo generate-lockfile
cargo update

# Commit the change
git add ./Cargo.toml
git commit -S -m "Bump version to v$VERSION"

# Tag
git tag -s "v$VERSION" -m "Version v$VERSION"

# Display
# echo "Bump version to v$VERSION"
