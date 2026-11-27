#!/bin/sh
# SPDX-License-Identifier: GPL-3.0-or-later
set -eu
sdk=$(cd "$1" && pwd)
mkdir -p "$2"
out=$(cd "$2" && pwd)
root=$(cd "$(dirname "$0")/../.." && pwd)
triple=aarch64-unknown-linux-gnu
gcc="$sdk/bin/aarch64-meego-linux-gnu-gcc"

export CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER="$gcc"
export TARGET_CC="$gcc"
export TARGET_CXX="$sdk/bin/aarch64-meego-linux-gnu-g++"
export TARGET_AR="$sdk/bin/aarch64-meego-linux-gnu-ar"
export QT_INCLUDE_PATH="$sdk/sysroot/usr/include/qt5"
export QT_LIBRARY_PATH="$sdk/sysroot/usr/lib64"
export SAILFISHAPP_INCLUDE_PATH="$sdk/sysroot/usr/include/sailfishapp"
export CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_RUSTFLAGS="-C link-arg=-Wl,--as-needed -C link-arg=-Wl,-z,relro,-z,now"
export CARGO_TARGET_DIR="$out/target"
export CARGO_INCREMENTAL=0

cd "$root"
rustup target add "$triple" > /dev/null
cargo build --release --locked --target "$triple" -p harbour-nanunana --features sailfish
cp -f "$CARGO_TARGET_DIR/$triple/release/harbour-nanunana" "$out/harbour-nanunana"
ls -l "$out/harbour-nanunana"
