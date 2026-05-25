# Contributing

Contributions are welcome. Here's how to get started.

## Setup

```bash
git clone git@github.com:corploc/kmlcli.git
cd kmlcli
cargo build
cargo test
```

## Workflow

1. Open an issue first if it's a non-trivial change — saves wasted work
2. Fork, branch from `main`
3. Use conventional commits (`feat:`, `fix:`, `docs:`, `chore:`, `refactor:`, `test:`) — release-please depends on it
4. Run before pushing:
   ```bash
   cargo fmt
   cargo clippy -- -D warnings
   cargo test
   ```
5. Open a PR with a clear description

## Reporting bugs

Use the bug issue template. Include:
- KML/KMZ file that triggers the bug (or a minimal reproducer)
- Terminal emulator + OS
- `kmlcli --version`
- Steps + expected vs actual

## Releases

Handled automatically by release-please on merge to `main`. Maintainers only.

## License

By contributing you agree your work is licensed under MIT OR Apache-2.0.
