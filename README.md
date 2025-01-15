# CLSAG vs FCMP++ bench suite

This benchmark suite compares the time it takes to verify a CLSAG signature versus a FCMP++ signature in Monero. These times do not include verifying amount commitments balance, range proofs, or other transaction semantic rules. And by "FCMP++ signature" here, I mean the membership proof AND the spend authorization / linking proof together, since CLSAG fills all these roles. Hopefully, the results of these benchmarks can inform future consensus protocol decisions for Monero.

## Benching

1. Install Monero build dependencies: https://github.com/monero-project/monero?tab=readme-ov-file#compiling-monero-from-source
2. Install Rust toolchain: https://rustup.rs/
3. Install Gnuplot (optional): https://riptutorial.com/gnuplot/example/11275/installation-or-setup
4. Clone this repo: `git clone --recursive https://github.com/jeffro256/clsag_vs_fcmppp_bench; cd clsag_vs_fcmppp_bench`
5. Pull new commits and build benchmark binaries: `./build_bench.sh`
6. Actually run benchmarks: `./run_bench.sh`

For subsequent benchmarks runs, you will only need to run `./run_bench.sh`.
If there are new commits to the benchmark repo and/or you wish to rebuild the benchmark suite, you can run `./build_bench.sh` again.

The output of `./run_bench.sh` should look something like this:
```
...
<LINUX DISTRO INFO>
...
$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$
...
<CPU INFO>
...
$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$
    Finished `bench` profile [optimized] target(s) in 0.06s
     Running benches/benchmark.rs (target/release/deps/benchmark-599c020ecf8457c3)
Gnuplot not found, using plotters backend
4 C1 branch blinds and 3 C2 branch blinds took 3461ms to calculate
8 C1 branch blinds and 6 C2 branch blinds took 6936ms to calculate
16 C1 branch blinds and 12 C2 branch blinds took 13992ms to calculate
32 C1 branch blinds and 24 C2 branch blinds took 28854ms to calculate
64 C1 branch blinds and 48 C2 branch blinds took 56403ms to calculate
FCMP++ verify with N inputs/1
                        time:   [33.601 ms 33.610 ms 33.622 ms]
                        change: [-1.3519% -1.2486% -1.1556%] (p = 0.00 < 0.05)
                        Performance has improved.
Found 7 outliers among 100 measurements (7.00%)
  5 (5.00%) high mild
  2 (2.00%) high severe
FCMP++ verify with N inputs/2
                        time:   [54.794 ms 54.837 ms 54.886 ms]
Found 4 outliers among 100 measurements (4.00%)
  1 (1.00%) high mild
  3 (3.00%) high severe
FCMP++ verify with N inputs/4
                        time:   [98.127 ms 98.154 ms 98.184 ms]
Found 12 outliers among 100 measurements (12.00%)
  9 (9.00%) high mild
  3 (3.00%) high severe
FCMP++ verify with N inputs/8
                        time:   [183.64 ms 183.68 ms 183.73 ms]
Found 16 outliers among 100 measurements (16.00%)
  2 (2.00%) low mild
  9 (9.00%) high mild
  5 (5.00%) high severe
Benchmarking FCMP++ verify with N inputs/16: Warming up for 3.0000 s
Warning: Unable to complete 100 samples in 30.0s. You may wish to increase target time to 37.7s, or reduce sample count to 70.
FCMP++ verify with N inputs/16
                        time:   [375.32 ms 375.99 ms 376.64 ms]

$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$
test_sig_clsag<2, 2, 1> (1000 calls) - OK: 496 µs/call
test_sig_clsag<3, 2, 1> (1000 calls) - OK: 722 µs/call
test_sig_clsag<4, 2, 1> (1000 calls) - OK: 955 µs/call
test_sig_clsag<8, 2, 1> (1000 calls) - OK: 1864 µs/call
test_sig_clsag<16, 2, 1> (1000 calls) - OK: 3717 µs/call
test_sig_clsag<24, 2, 1> (1000 calls) - OK: 5618 µs/call
test_sig_clsag<32, 2, 1> (1000 calls) - OK: 7525 µs/call
test_sig_clsag<40, 2, 1> (1000 calls) - OK: 9474 µs/call
test_sig_clsag<48, 2, 1> (1000 calls) - OK: 11376 µs/call
test_sig_clsag<56, 2, 1> (1000 calls) - OK: 13318 µs/call
test_sig_clsag<64, 2, 1> (1000 calls) - OK: 15275 µs/call
test_sig_clsag<80, 2, 1> (1000 calls) - OK: 19260 µs/call
test_sig_clsag<96, 2, 1> (1000 calls) - OK: 23298 µs/call
test_sig_clsag<112, 2, 1> (1000 calls) - OK: 27311 µs/call
test_sig_clsag<128, 2, 1> (1000 calls) - OK: 31492 µs/call
test_sig_clsag<160, 2, 1> (1000 calls) - OK: 39973 µs/call
test_sig_clsag<192, 2, 1> (1000 calls) - OK: 48971 µs/call
test_sig_clsag<224, 2, 1> (1000 calls) - OK: 57857 µs/call
test_sig_clsag<256, 2, 1> (1000 calls) - OK: 67081 µs/call
Tests finished. Elapsed time: 391 sec
```

## Analyzing

The FCMP++ benchmarks results are in the section between lines of `$` characters, starting with the ```Finished `bench` profile [optimized] target(s) in ...``` message. These timing results are a function of the number of inputs. If a transaction is spending 4 TXOs, then the `FCMP++ verify with N inputs/4` result is the one relevant here, etc. You can find nicely formatted graphs and an HTML report for the FCMP++ results at `target/criterion/report/index.html`. You should observe that FCMP++s benefit from "batching": the more proofs verified in a "batch", the faster the verification for each individual proof is. Thus, the time to verify a 16-input FCMP++ transaction is not exactly 16 times slower than verifying a 1-input transaction.

The CLSAG results are in the last section of the output, after the last line of `$`. By constrast, these results are a function of the number of *ring members*, not inputs. As of the time of this writing, January 2025, Monero's ring size is 16. This benchmark section runs results for CLSAG signature of many different ring sizes, ranging from 2 to 256. Since CLSAGs do not batch well, the number of transactions inputs were not varied in this test. One can extropolate the verification time for a transaction with N CLSAG inputs to be more or less N times slower than 1 CLSAG input.
