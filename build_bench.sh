#!/usr/bin/bash

cd -- $(dirname -- $0)

# Update repo
git pull
git submodule update --init --force

# Update and compile monero branch
pushd monero
git submodule update --init --force
make -j`nproc`
popd

# Compile Rust benchmark
cargo build --benches --profile bench
