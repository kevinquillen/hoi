#!/bin/bash
set -e

# Install cross if not installed
if ! command -v cross &> /dev/null; then
    cargo install cross
fi

# Define targets
TARGETS=(
    # Windows
    "x86_64-pc-windows-gnu"     # Windows x86_64
    "aarch64-pc-windows-msvc"   # Windows ARM64
    "i686-pc-windows-gnu"       # Windows x86
    
    # macOS
    "x86_64-apple-darwin"       # macOS x86_64
    "aarch64-apple-darwin"      # macOS ARM64
    
    # Linux
    "x86_64-unknown-linux-gnu"  # Linux x86_64
    "aarch64-unknown-linux-gnu" # Linux ARM64
    "i686-unknown-linux-gnu"    # Linux x86
)

# Create output directory
mkdir -p target/builds

# Build for each target
for TARGET in "${TARGETS[@]}"; do
    echo "Building for $TARGET..."
    cross build --release --target "$TARGET" || echo "Failed to build for $TARGET"
    
    # Create output filename with platform and arch info
    PLATFORM=$(echo "$TARGET" | cut -d'-' -f2)
    ARCH=$(echo "$TARGET" | cut -d'-' -f1)
    
    if [[ "$TARGET" == *"windows"* ]]; then
        EXT=".exe"
    else
        EXT=""
    fi
    
    # Copy binary to output directory if build succeeded
    if [ -f "target/$TARGET/release/hoi$EXT" ]; then
        cp "target/$TARGET/release/hoi$EXT" "target/builds/hoi-$PLATFORM-$ARCH$EXT"
        echo "✓ Successfully built for $TARGET"
    fi
done

echo "Done! Binaries are in target/builds/"