# CLSAG vs FCMP++ bench suite

## Instructions

1. Install Monero build dependencies: https://github.com/monero-project/monero?tab=readme-ov-file#compiling-monero-from-source
2. Install Rust toolchain: https://rustup.rs/
3. Clone this repo: `git clone --recursive https://github.com/jeffro256/clsag_vs_fcmppp_bench; cd clsag_vs_fcmppp_bench`
4. Pull new commits and build benchmark binaries: `./build_bench.sh`
5. Actually run benchmarks: `./run_bench.sh`

For subsequent benchmarks runs, you will only need to run `./run_bench.sh`.
If there are new commits and/or you wish to rebuild the benchmark suite, you can run `./build_bench.sh` again.

The output of `./run_bench.sh` should look something like this:
```
    Finished `bench` profile [optimized] target(s) in 14.20s
     Running benches/benchmark.rs (target/release/deps/benchmark-4eb6fc4a2ba82e38)
4 C1 branch blinds and 3 C2 branch blinds took 2183ms to calculate
FCMP++ verify with N inputs/1
                        time:   [24.855 ms 24.932 ms 25.015 ms]
                        change: [+1.4802% +1.8054% +2.1849%] (p = 0.00 < 0.05)
                        Performance has regressed.
Found 2 outliers among 100 measurements (2.00%)
  2 (2.00%) high mild

$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$
test_sig_clsag<2, 2, 1> (1000 calls) - OK: 350 µs/call
test_sig_clsag<3, 2, 1> (1000 calls) - OK: 512 µs/call
test_sig_clsag<4, 2, 1> (1000 calls) - OK: 674 µs/call
test_sig_clsag<8, 2, 1> (1000 calls) - OK: 1317 µs/call
test_sig_clsag<16, 2, 1> (1000 calls) - OK: 2639 µs/call
test_sig_clsag<24, 2, 1> (1000 calls) - OK: 4000 µs/call
test_sig_clsag<32, 2, 1> (1000 calls) - OK: 5323 µs/call
test_sig_clsag<40, 2, 1> (1000 calls) - OK: 6733 µs/call
test_sig_clsag<48, 2, 1> (1000 calls) - OK: 8034 µs/call
test_sig_clsag<56, 2, 1> (1000 calls) - OK: 9390 µs/call
test_sig_clsag<64, 2, 1> (1000 calls) - OK: 10817 µs/call
test_sig_clsag<80, 2, 1> (1000 calls) - OK: 13770 µs/call
test_sig_clsag<96, 2, 1> (1000 calls) - OK: 16671 µs/call
test_sig_clsag<112, 2, 1> (1000 calls) - OK: 19466 µs/call
test_sig_clsag<128, 2, 1> (1000 calls) - OK: 22367 µs/call
test_sig_clsag<160, 2, 1> (1000 calls) - OK: 28591 µs/call
test_sig_clsag<192, 2, 1> (1000 calls) - OK: 35017 µs/call
test_sig_clsag<224, 2, 1> (1000 calls) - OK: 41295 µs/call
test_sig_clsag<256, 2, 1> (1000 calls) - OK: 48045 µs/call
Tests finished. Elapsed time: 279 sec
```
