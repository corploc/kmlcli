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

# run TUI smoke test — produces screenshots in tests/smoke/ for visual inspection
smoke-test: build
    @rm -rf tests/smoke
    @mkdir -p tests/smoke
    @rm -f /tmp/kmlcli_perf.log
    vhs tests/smoke.tape
    @test ! -f /tmp/kmlcli_perf.log || (echo "FAIL: /tmp/kmlcli_perf.log was written — T1 regressed" && exit 1)
    @echo "Screenshots written to tests/smoke/"
    @ls tests/smoke/

# clean build artifacts
clean:
    cargo clean
