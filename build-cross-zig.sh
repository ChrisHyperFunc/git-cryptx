#!/bin/bash

# Clear the target directory
rm -rf target/release-cross/*
mkdir -p target/release-cross

build_and_package() {
    TARGET=$1
    OUTPUT_NAME="git-cryptx-${TARGET}"

    if cargo zigbuild --release --target "$TARGET"; then
        if [[ "$TARGET" == "x86_64-pc-windows-gnu" ]]; then
            # Use zip format for Windows
            zip -r "target/release-cross/${OUTPUT_NAME}.zip" -j "target/$TARGET/release/git-cryptx.exe"
            echo "Packaging successful: ${OUTPUT_NAME}.zip"
        else
            # Use tar.gz format for other platforms
            tar -czf "target/release-cross/${OUTPUT_NAME}.tar.gz" -C "target/$TARGET/release/" git-cryptx
            echo "Packaging successful: ${OUTPUT_NAME}.tar.gz"
        fi
    else
        echo "Compilation failed: $TARGET"
    fi
}

# macOS
build_and_package "aarch64-apple-darwin"
build_and_package "x86_64-apple-darwin"

# Windows
build_and_package "x86_64-pc-windows-gnu"

# Linux
build_and_package "x86_64-unknown-linux-musl"

rm -f .intentionally-empty-file.o