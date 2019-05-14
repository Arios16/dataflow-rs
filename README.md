# 17355 - Program Analysis - Project
## Andres Rios - ariossta

The code for the library is included in the `src` folder. 

A runnable example is provided in `examples/precise_sign_analysis`. To execute
it, install the latest Rust nightly version (it needs to be nightly because 
compiler internals cannot be used from the stable version), set the environment
variable `RUST_SYSROOT` pointing to the nightly toolchain installation (it 
should look like `/home/$USER/.rustup/toolchains/nightly-x86_64-unknown-linux-gnu`
if you're on Linux), and then execute `cargo run -- example.rs` inside 
`examples/precise_sign_analysis` folder. This will run the analysis on the 
`examples/precise_sign_analysis/example.rs` code.
