#!/bin/bash
set -e

# Define directories
PKG_DIR="./pkg"
NODE_DIR="${PKG_DIR}/node"
BROWSER_DIR="${PKG_DIR}/browser"
TYPES_DIR="./types"

# Use the Node.js build as the source for the initial package.json and README.md
SOURCE_DIR="$NODE_DIR"

# Copy package.json and README.md from the source to the pkg root, if they exist
if [ -f "${SOURCE_DIR}/package.json" ]; then
  cp "${SOURCE_DIR}/package.json" "${PKG_DIR}/package.json"
  echo "Copied package.json from ${SOURCE_DIR} to ${PKG_DIR}"
else
  echo "package.json not found in ${SOURCE_DIR}"
fi

if [ -f "${SOURCE_DIR}/README.md" ]; then
  cp "${SOURCE_DIR}/README.md" "${PKG_DIR}/README.md"
  echo "Copied README.md from ${SOURCE_DIR} to ${PKG_DIR}"
else
  echo "README.md not found in ${SOURCE_DIR}"
fi

# Remove package.json and README.md from both subdirectories
for dir in "$NODE_DIR" "$BROWSER_DIR"; do
  rm -f "${dir}/package.json" "${dir}/README.md"
  echo "Removed package.json and README.md from ${dir}"
done

# Replace cquisitor_lib.d.ts in each subdirectory with the one from TYPES_DIR
if [ -f "${TYPES_DIR}/cquisitor_lib.d.ts" ]; then
  for dir in "$NODE_DIR" "$BROWSER_DIR"; do
    cp "${TYPES_DIR}/cquisitor_lib.d.ts" "${dir}/cquisitor_lib.d.ts"
    echo "Replaced cquisitor_lib.d.ts in ${dir} with the one from ${TYPES_DIR}"
  done
else
  echo "cquisitor_lib.d.ts not found in ${TYPES_DIR}!"
  exit 1
fi

# Update the root package.json to add conditional exports and a browser field
if command -v jq >/dev/null 2>&1; then
  echo "Updating package.json with conditional exports and browser field."
  TMPFILE=$(mktemp)
  jq '
    # Set default main and types to the Node.js build
    .main = "./node/cquisitor_lib.js" |
    .types = "./node/cquisitor_lib.d.ts" |
    # Set the browser field to point to the browser build
    .browser = "./browser/cquisitor_lib.js" |
    # Update files array to include both builds
    .files = [
      "node/cquisitor_lib_bg.wasm",
      "node/cquisitor_lib.js",
      "node/cquisitor_lib.d.ts",
      "node/cquisitor_lib_bg.wasm.d.ts",
      "browser/cquisitor_lib_bg.wasm",
      "browser/cquisitor_lib.js",
      "browser/cquisitor_lib.d.ts",
      "browser/cquisitor_lib_bg.wasm.d.ts"
    ] |
    # Define conditional exports for Node.js and browser
    .exports = {
      ".": {
        "node": {
          "import": "./node/cquisitor_lib.js",
          "require": "./node/cquisitor_lib.js"
        },
        "browser": "./browser/cquisitor_lib.js",
        "default": "./node/cquisitor_lib.js"
      }
    }
  ' "${PKG_DIR}/package.json" > "$TMPFILE" && mv "$TMPFILE" "${PKG_DIR}/package.json"
  echo "package.json updated with conditional exports and browser field."
else
  echo "jq is not installed. Please install jq to update package.json fields."
fi

if command -v jq >/dev/null 2>&1; then
  echo "Updating package name in pkg/package.json with the name from root package.json."
  PACKAGE_NAME=$(jq -r '.name' package.json)
  TMPFILE=$(mktemp)
  jq --arg name "$PACKAGE_NAME" '.name = $name' "${PKG_DIR}/package.json" > "$TMPFILE" && mv "$TMPFILE" "${PKG_DIR}/package.json"
  echo "Package name updated to ${PACKAGE_NAME} in pkg/package.json."
else
  echo "jq is not installed. Please install jq to update package.json fields."
fi

cp .npmrc ./pkg/.npmrc

