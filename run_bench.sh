#!/usr/bin/bash

cd -- $(dirname -- $0)

cargo bench

echo '$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$'

XMR_PERFTEST="./monero/build/release/tests/performance_tests/performance_tests"

"$XMR_PERFTEST" --filter=test_sig_clsag*
