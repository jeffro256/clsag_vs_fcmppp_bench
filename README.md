# CLSAG vs FCMP++ bench suite

## Instructions

1. Clone repo: `git clone --recursive https://github.com/jeffro256/clsag_vs_fcmppp_bench`
2. Pulling new commits and building benchmarks: `./build_bench.sh`
3. Actually running benchmarks: `./run_bench.sh`

You will need all the dependencies that are normally needed to build the Monero core repo:
https://github.com/monero-project/monero?tab=readme-ov-file#compiling-monero-from-source

You will also need a modern (>= 1.69) Rust compiler and cargo installtion.

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
