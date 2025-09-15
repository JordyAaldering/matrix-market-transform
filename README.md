# Matrix-Market Transform

Transform between row-major and column-major representations of the Matrix-Market file format (`.mtx`).

For usage, run `cargo run -- -h` or `matrix_market_transform -h`.

By default, `rayon` uses all available physical and logical cores, without pinning.
For improved performance, using only physical cores should be used.
This can be achieved by setting `RAYON_NUM_THREADS` to the number of physical cores on your system, and using the `tasket` command to pin the program to only those physical cores.

```
cargo build --release
RAYON_NUM_THREADS=8 taskset -c 0,2,4,6,8,10,12,14 ./target/release/matrix_market_transform -s row-major data/RM07R.mtx
```
