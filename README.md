# 17355 - Program Analysis - Project Milestone
## Andres Rios - ariossta

I have managed to hook in to the compiler, extract the MIR and traverse the 
CFG of a program. This is what I promised in the proposal, so I'm currently 
on schedule. Now I have to start working on designing and implementing the 
actual analysis framework.

The code is included in the `src` folder. To execute it, install the latest
Rust nightly version (it needs to be nightly because I use compiler internals
that  cannot be used from the stable version), set the environment variable 
`RUST_SYSROOT` pointing to the nightly toolchain installation (it should look
like `/home/$USER/.rustup/toolchains/nightly-x86_64-unknown-linux-gnu` if 
you're on Linux), and then execute `cargo run` inside this folder. This will
compile the `example.rs` file, extract the MIR and print it to stdout.
