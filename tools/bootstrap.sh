#!/bin/sh
set -eu
rustup toolchain install 1.94.1 --profile minimal --component clippy --component rustfmt --component llvm-tools-preview
cargo fetch --locked
git config core.hooksPath .githooks
