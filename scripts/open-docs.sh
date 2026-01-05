#!/usr/bin/env bash
set -e

DOC_PARENT="$(pwd)/target/thumbv8m.main-none-eabihf/doc"
DOC_PATH="$DOC_PARENT/device_kit/index.html"

if [ ! -f "$DOC_PATH" ]; then
  echo "Error: Documentation not found at $DOC_PATH"
  echo "Run 'cargo docdoc' first to build the docs"
  exit 1
fi

# Copy entire doc directory (includes all dependencies) to Windows temp
TEMP_WINDOWS=$(cmd.exe /c "echo %TEMP%" 2>/dev/null | tr -d '\r')
TEMP_DIR=$(wslpath -u "$TEMP_WINDOWS")
TEMP_DOCS="$TEMP_DIR/device_kit_docs"

rm -rf "$TEMP_DOCS"
cp -r "$DOC_PARENT" "$TEMP_DOCS"

# Convert temp path to Windows path and open
TEMP_WIN_PATH=$(wslpath -w "$TEMP_DOCS/device_kit/index.html")
powershell.exe -NoProfile -Command "Invoke-Item '$TEMP_WIN_PATH'" &
