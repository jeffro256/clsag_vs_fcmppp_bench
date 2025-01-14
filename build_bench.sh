#!/usr/bin/bash

cd -- $(dirname -- $0)

git submodule sync --recursive

pushd monero && make -j`nproc` && popd && cargo build --benches --profile bench && echo "done building ;)" && exit 0

echo 'build failed ;('
exit 1
