# list recipes (default)
default:
    @just --list

# build release binary
build:
    cargo build --release

# run all checks (fmt, clippy, test)
check:
    cargo fmt --check
    cargo clippy --all-targets -- -D warnings
    cargo test

# format code
fmt:
    cargo fmt

# install locally from source
install:
    cargo install --path .

# regenerate docs/demo.gif from demo.tape
demo: build
    @mkdir -p docs
    vhs demo.tape

# clean build artifacts
clean:
    cargo clean
