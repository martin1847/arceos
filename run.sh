cd payload/hello_app/
cargo build --target riscv64gc-unknown-none-elf --release
rust-objcopy --binary-architecture=riscv64 --strip-all -O binary ../../target/riscv64gc-unknown-none-elf/release/hello_app ../hello_abi.bin
cd ..
./up_apps.bin.sh hello_abi.bin
cd ..
make ${1:-run}